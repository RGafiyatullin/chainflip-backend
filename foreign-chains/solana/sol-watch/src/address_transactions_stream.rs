//! Works fine until if there is no slot with multiple transactions :\

use std::{borrow::Borrow, collections::VecDeque, sync::atomic::AtomicBool, time::Duration};

use futures::{stream, Stream, TryStreamExt};
use sol_prim::{Address, Signature, SlotNumber};
use sol_rpc::{calls::GetSignaturesForAddress, traits::CallApi};

const DEFAULT_MAX_PAGE_SIZE: usize = 100;
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(5);

pub struct AddressSignatures<Api, K> {
	call_api: Api,
	address: Address,
	starting_with_slot: Option<SlotNumber>,
	after_transaction: Option<Signature>,
	max_page_size: usize,
	poll_interval: Duration,

	state: State,
	kill_switch: K,
}

impl<Api, K> AddressSignatures<Api, K> {
	pub fn new(call_api: Api, address: Address, kill_switch: K) -> Self {
		Self {
			call_api,
			address,
			starting_with_slot: None,
			after_transaction: None,
			max_page_size: DEFAULT_MAX_PAGE_SIZE,
			poll_interval: DEFAULT_POLL_INTERVAL,

			state: State::GetHistory(Duration::ZERO, None),
			kill_switch,
		}
	}

	pub fn starting_with_slot(self, slot: SlotNumber) -> Self {
		Self { starting_with_slot: Some(slot), ..self }
	}
	pub fn after_transaction(self, tx_id: Signature) -> Self {
		Self { after_transaction: Some(tx_id), ..self }
	}
	pub fn max_page_size(self, max_page_size: usize) -> Self {
		Self { max_page_size, ..self }
	}
	pub fn poll_interval(self, poll_interval: Duration) -> Self {
		Self { poll_interval, ..self }
	}
}

impl<Api, K> AddressSignatures<Api, K>
where
	Api: CallApi,
	K: Borrow<AtomicBool>,
{
	pub fn into_stream(mut self) -> impl Stream<Item = Result<Signature, Api::Error>> {
		self.state = State::GetHistory(Duration::ZERO, self.after_transaction);
		stream::try_unfold(self, Self::unfold).try_filter_map(|opt| async move { Ok(opt) })
	}
}

enum State {
	GetHistory(Duration, Option<Signature>),
	Drain(VecDeque<Signature>, Option<Signature>),
}

impl<Api, K> AddressSignatures<Api, K>
where
	Api: CallApi,
	K: Borrow<AtomicBool>,
{
	async fn unfold(mut self) -> Result<Option<(Option<Signature>, Self)>, Api::Error> {
		if self.kill_switch.borrow().load(std::sync::atomic::Ordering::Relaxed) {
			return Ok(None)
		}

		let out = match self.state {
			State::GetHistory(sleep, last_signature) => {
				tokio::time::sleep(sleep).await;

				let mut history = VecDeque::new();
				get_transaction_history(
					&self.call_api,
					&mut history,
					self.address,
					self.starting_with_slot,
					last_signature,
					self.max_page_size,
				)
				.await?;
				let last_signature = history.front().copied().or(last_signature);
				self.state = State::Drain(history, last_signature);

				Some((None, self))
			},
			State::Drain(mut queue, last_signature) =>
				if let Some(signature) = queue.pop_back() {
					self.state = State::Drain(queue, last_signature);
					Some((Some(signature), self))
				} else {
					self.state = State::GetHistory(self.poll_interval, last_signature);
					Some((None, self))
				},
		};
		Ok(out)
	}
}

async fn get_transaction_history<Api>(
	call_api: Api,
	output: &mut impl Extend<Signature>,

	address: Address,
	starting_with_slot: Option<SlotNumber>,
	after_tx: Option<Signature>,

	max_page_size: usize,
) -> Result<(), Api::Error>
where
	Api: CallApi,
{
	let mut before_tx = None;

	loop {
		let (page_size, reference_signature) = get_single_page(
			&call_api,
			output,
			address,
			starting_with_slot,
			after_tx,
			before_tx,
			max_page_size,
		)
		.await?;

		before_tx = reference_signature;

		if page_size != max_page_size {
			break Ok(())
		}
	}
}

async fn get_single_page<Api>(
	call_api: Api,
	output: &mut impl Extend<Signature>,

	address: Address,
	starting_with_slot: Option<SlotNumber>,
	after_tx: Option<Signature>,
	before_tx: Option<Signature>,

	max_page_size: usize,
) -> Result<(usize, Option<Signature>), Api::Error>
where
	Api: CallApi,
{
	let request = GetSignaturesForAddress {
		before: before_tx,
		until: after_tx,
		limit: Some(max_page_size),
		..GetSignaturesForAddress::for_address(address)
	};
	let page = call_api.call(request).await?;

	// TODO: make sure the page is actually sorted by slot-number.

	let mut row_count = 0;
	let mut reference_signature = None;
	let signatures = page
		.into_iter()
		.take_while(|e| starting_with_slot.map(|s| s <= e.slot).unwrap_or(true))
		.map(|e| {
			row_count += 1;
			reference_signature = Some(e.signature);

			e.signature
		});

	output.extend(signatures);

	Ok((row_count, reference_signature))
}

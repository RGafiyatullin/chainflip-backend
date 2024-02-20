//! A stream of transactions that "touch" the specified address.
//! 
//! Works fine if no two transactions have the same slot-number.
//! I think the following is implemented incorrectly: [handling `before` argument](https://github.com/solana-labs/solana/blob/master/ledger/src/blockstore.rs#L2845)
//! 
//! The argument `before` if specified — stands for the "most recent" exclusive boundary of the search,
//! i.e. this transaction and anything that happens later — should be excluded.
//! The code does the following:
//! - looks up the `slot`, that the `before`-transaction belongs to;
//! - takes all the tx-ids from that slot in reverse order (the newer — at the front, the older — at the back of the list);
//! - finds the exact position of the `before`-transaction in this list;
//! - truncates the list, effectively:
//!   - throwing away the older transactions;
//!   - keeping the newer transactions.
//!


use std::{collections::VecDeque, time::Duration};

use futures::{stream, Stream, TryStreamExt};
use sol_prim::{Address, Signature, SlotNumber};
use sol_rpc::{calls::GetSignaturesForAddress, traits::CallApi};

const DEFAULT_MAX_PAGE_SIZE: usize = 1000;
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(5);

pub struct AddressSignatures<Api> {
	call_api: Api,
	address: Address,
	starting_with_slot: Option<SlotNumber>,
	after_transaction: Option<Signature>,
	max_page_size: usize,
	poll_interval: Duration,

	state: State,
}

impl<Api> AddressSignatures<Api> {
	pub fn new(call_api: Api, address: Address) -> Self {
		Self {
			call_api,
			address,
			starting_with_slot: None,
			after_transaction: None,
			max_page_size: DEFAULT_MAX_PAGE_SIZE,
			poll_interval: DEFAULT_POLL_INTERVAL,

			state: State::GetHistory(Duration::ZERO, None),
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

impl<Api> AddressSignatures<Api>
where
	Api: CallApi,
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

impl<Api> AddressSignatures<Api>
where
	Api: CallApi,
{
	async fn unfold(mut self) -> Result<Option<(Option<Signature>, Self)>, Api::Error> {
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

		before_tx = reference_signature.or(before_tx);

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

	let mut page = call_api.call(request).await?;

	// page.sort_unstable_by(|lo, hi| {
	// 	(lo.slot, &lo.signature).cmp(&(hi.slot, &hi.signature)).reverse()
	// });
	// page.sort_by_key(|e| std::cmp::Reverse(e.slot));

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

type AnyError = Box<dyn std::error::Error + Send + Sync + 'static>;

use futures::TryStreamExt;
use sol_prim::{Address, SlotNumber};
use sol_rpc::{calls::GetTransaction, traits::CallApi};
use sol_watch::{
	address_transactions_stream::AddressSignatures, deduplicate_stream::DeduplicateStreamExt,
};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
	#[structopt(long, short, env = "API_URL", default_value = "https://api.devnet.solana.com:443")]
	call_api: String,

	#[structopt(long, short, default_value = "1000")]
	page_size: usize,

	#[structopt(long, short, default_value = "0")]
	slot: SlotNumber,

	#[structopt(long, short, default_value = "1000")]
	dedup_backlog: usize,

	#[structopt()]
	address: Address,
}

#[tokio::main]
async fn main() -> Result<(), AnyError> {
	let args: Args = StructOpt::from_args();

	let call_api = sol_rpc::retrying::Retrying::new(
		jsonrpsee::http_client::HttpClientBuilder::default().build(args.call_api.as_str())?,
		sol_rpc::retrying::Delays::default(),
	);

	AddressSignatures::new(&call_api, args.address)
		.starting_with_slot(args.slot)
		.max_page_size(args.page_size)
		.into_stream()
		.deduplicate(
			args.dedup_backlog,
			|result| result.as_ref().ok().copied(),
			|tx_id, _| eprintln!("! {}", tx_id),
		)
		.and_then(|tx_id| {
			let call_api = &call_api;
			async move {
				let response = call_api.call(GetTransaction::for_signature(tx_id)).await?;
				Ok((response.slot, tx_id, response.balances(&args.address)))
			}
		})
		.try_for_each(|(slot, tx_id, balance)| async move {
			Ok(eprintln!("- [{}] {}: {:?}", slot, tx_id, balance))
		})
		.await?;

	Ok(())
}

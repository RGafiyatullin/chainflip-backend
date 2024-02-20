type AnyError = Box<dyn std::error::Error + Send + Sync + 'static>;

use futures::{stream, StreamExt, TryStreamExt};
use sol_prim::Address;
use sol_watch::address_transactions_stream::AddressSignatures;

#[tokio::main]
async fn main() -> Result<(), AnyError> {
	let addresses: Vec<Address> =
		std::env::args().skip(1).map(|s| s.parse()).collect::<Result<_, _>>()?;

	eprintln!("Addresses:");
	for a in &addresses {
		eprintln!(" - {}", a);
	}

	let call_api = jsonrpsee::http_client::HttpClientBuilder::default()
		.build("https://api.devnet.solana.com:443")?;
	stream::select_all(
		addresses
			.into_iter()
			.map(|a| AddressSignatures::new(&call_api, a).into_stream().boxed()),
	)
	.try_for_each(|s| async move { Ok(eprintln!("{}", s)) })
	.await?;

	Ok(())
}

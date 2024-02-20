use std::{collections::HashSet, time::Duration};

use futures::stream::{Stream, StreamExt};
use state_chain_runtime::{BitcoinInstance, SolanaInstance};
use tokio_stream::wrappers::IntervalStream;

use cf_chains::sol::SolAddress;
use cf_primitives::ChannelId;

use crate::state_chain_observer::client::{
	chain_api::ChainApi, extrinsic_api::signed::SignedExtrinsicApi, storage_api::StorageApi,
};

const SC_BLOCK_TIME: Duration = Duration::from_secs(6);

#[derive(Debug, Clone)]
pub struct DepositAddressesUpdate {
	pub added: Vec<(ChannelId, SolAddress)>,
	pub removed: Vec<(ChannelId, SolAddress)>,
}

pub fn deposit_addresses_updates<StateChainClient>(
	state_chain_client: &StateChainClient,
) -> impl Stream<Item = DepositAddressesUpdate> + '_
where
	StateChainClient: StorageApi + ChainApi + SignedExtrinsicApi + 'static + Send + Sync,
{
	IntervalStream::new(tokio::time::interval(SC_BLOCK_TIME))
		.then(|_| {
			let sc_latest_finalized_block = state_chain_client.latest_finalized_block();
			// tracing::warn!(
			// 	"SC_LATEST_FINALIZED_BLOCK: {:?}/{:?}",
			// 	sc_latest_finalized_block.number,
			// 	sc_latest_finalized_block.hash
			// );
			state_chain_client.storage_map_values::<pallet_cf_ingress_egress::DepositChannelLookup<
				state_chain_runtime::Runtime,
				SolanaInstance,
			>>(sc_latest_finalized_block.hash)
		})
		.filter_map(|result| async move {
			match result {
				Ok(deposit_addresses) => Some(deposit_addresses),
				Err(reason) => {
					tracing::warn!("Error fetching deposit-addresses: {}", reason);
					None
				},
			}
		})
		.map(|current_vec| {
			// if !current_vec.is_empty() {
			// 	tracing::warn!("DEPOSIT_ADDRESS_LOOKUP: {:#?}", current_vec);
			// }
			current_vec
				.into_iter()
				.map(|entry| {
					let chan = entry.deposit_channel;
					(chan.channel_id, chan.address)
				})
				.collect::<HashSet<_>>()
		})
		.scan(HashSet::new(), |prev_set, current_set| {
			let removed = prev_set.difference(&current_set).copied().collect::<Vec<_>>();
			let added = current_set.difference(&prev_set).copied().collect::<Vec<_>>();

			*prev_set = current_set;

			async move { Some(DepositAddressesUpdate { added, removed }) }
		})
}

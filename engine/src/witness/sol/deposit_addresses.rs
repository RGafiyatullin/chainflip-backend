use std::{collections::HashSet, time::Duration};

use futures::{
	stream::{self, Stream, StreamExt},
	TryFutureExt, TryStreamExt,
};
use state_chain_runtime::SolanaInstance;
use tokio_stream::wrappers::IntervalStream;

use cf_chains::sol::{DepositChannelState, SolAddress};
use cf_primitives::ChannelId;

use crate::state_chain_observer::client::{
	chain_api::ChainApi, extrinsic_api::signed::SignedExtrinsicApi, storage_api::StorageApi,
};

const SC_BLOCK_TIME: Duration = Duration::from_secs(6);

#[derive(Debug, Clone)]
pub struct DepositAddressesUpdate<
	Added = (ChannelId, SolAddress, DepositChannelState),
	Removed = (ChannelId, SolAddress),
> {
	pub added: Vec<Added>,
	pub removed: Vec<Removed>,
}

pub fn deposit_addresses_updates<StateChainClient>(
	state_chain_client: &StateChainClient,
) -> impl Stream<Item = Result<DepositAddressesUpdate, anyhow::Error>> + '_
where
	StateChainClient: StorageApi + ChainApi + SignedExtrinsicApi + 'static + Send + Sync,
{
	IntervalStream::new(tokio::time::interval(SC_BLOCK_TIME))
		.then(|_| {
			let sc_latest_finalized_block = state_chain_client.latest_finalized_block();
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
		.then(|update| populate_update_with_channel_state(state_chain_client, update))
}

async fn populate_update_with_channel_state<StateChainClient>(
	state_chain_client: &StateChainClient,
	update: DepositAddressesUpdate<(ChannelId, SolAddress)>,
) -> Result<DepositAddressesUpdate<(ChannelId, SolAddress, DepositChannelState)>, anyhow::Error>
where
	StateChainClient: StorageApi + ChainApi + SignedExtrinsicApi + 'static + Send + Sync,
{
	let sc_latest_finalized_blockhash = state_chain_client.latest_finalized_block().hash;
	let added = stream::iter(update.added)
		.then(|(channel_id, address)| async move {
			state_chain_client
				.storage_map_entry::<pallet_cf_ingress_egress::DepositChannelPool<
					state_chain_runtime::Runtime,
					SolanaInstance,
				>>(sc_latest_finalized_blockhash, &channel_id)
				.map_ok(move |state| {
					(channel_id, address, state.map(|c| c.state).unwrap_or_default())
				})
				.await
		})
		.try_collect::<Vec<_>>()
		.await?;
	Ok(DepositAddressesUpdate { added, removed: update.removed })
}

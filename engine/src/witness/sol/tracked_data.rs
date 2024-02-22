use cf_chains::{sol::SolTrackedData, ChainState, Solana};
use sol_rpc::calls::{GetExistingBlocks, GetSlot};

use super::{Result, SolanaApi};

const SLOT_GRANULARITY: u64 = 25;

pub async fn collect_tracked_data<C: SolanaApi>(sol_client: C) -> Result<ChainState<Solana>> {
	let latest_slot = sol_client.call(GetSlot::default()).await?;
	let min_slot = latest_slot - (latest_slot % SLOT_GRANULARITY);
	let existing_slots = sol_client.call(GetExistingBlocks::range(min_slot, latest_slot)).await?;
	let reported_slot = existing_slots
		.first()
		.copied()
		.ok_or_else(|| anyhow::anyhow!("Come on! At least the `latest_slot` must exist!"))?;

	let chain_state = ChainState {
		block_height: reported_slot,
		tracked_data: SolTrackedData { ingress_fee: None, egress_fee: None },
	};

	Ok(chain_state)
}

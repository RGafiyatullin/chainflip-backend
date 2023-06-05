use crate::{BitcoinInstance, EthereumInstance, PolkadotInstance, Runtime, RuntimeCall};
use codec::{Decode, Encode};
use pallet_cf_witnesser::WitnessDataExtraction;
use sp_std::{mem, prelude::*};

fn select_median<T: Encode + Decode>(data: &mut [Vec<u8>]) -> Option<T> {
	if data.is_empty() {
		return None
	}

	let len = data.len();
	let median_index = if len % 2 == 0 { (len - 1) / 2 } else { len / 2 };
	// Encoding is order-preserving so we can sort the raw encoded bytes and then decode
	// just the result.
	let (_, median_bytes, _) = data.select_nth_unstable(median_index);

	match Decode::decode(&mut &median_bytes[..]) {
		Ok(median) => Some(median),
		Err(e) => {
			log::error!("Failed to decode median priority fee: {:?}", e);
			None
		},
	}
}

impl WitnessDataExtraction for RuntimeCall {
	fn extract(&mut self) -> Option<Vec<u8>> {
		match self {
			RuntimeCall::EthereumChainTracking(pallet_cf_chain_tracking::Call::<
				Runtime,
				EthereumInstance,
			>::update_chain_state {
				ref mut state,
			}) => {
				let priority_fee = mem::take(&mut state.priority_fee);
				Some(priority_fee.encode())
			},
			RuntimeCall::BitcoinChainTracking(pallet_cf_chain_tracking::Call::<
				Runtime,
				BitcoinInstance,
			>::update_chain_state {
				ref mut state,
			}) => {
				let fee_info = mem::take(&mut state.btc_fee_info);
				Some(fee_info.encode())
			},
			RuntimeCall::PolkadotChainTracking(pallet_cf_chain_tracking::Call::<
				Runtime,
				PolkadotInstance,
			>::update_chain_state {
				ref mut state,
			}) => {
				let fee_info = mem::take(&mut state.median_tip);
				Some(fee_info.encode())
			},
			_ => None,
		}
	}

	fn combine_and_inject(&mut self, data: &mut [Vec<u8>]) {
		match self {
			RuntimeCall::EthereumChainTracking(pallet_cf_chain_tracking::Call::<
				Runtime,
				EthereumInstance,
			>::update_chain_state {
				state,
			}) =>
				if let Some(median) = select_median(data) {
					state.priority_fee = median;
				},
			RuntimeCall::BitcoinChainTracking(pallet_cf_chain_tracking::Call::<
				Runtime,
				BitcoinInstance,
			>::update_chain_state {
				state,
			}) => {
				if let Some(median) = select_median(data) {
					state.btc_fee_info = median;
				};
			},
			RuntimeCall::PolkadotChainTracking(pallet_cf_chain_tracking::Call::<
				Runtime,
				PolkadotInstance,
			>::update_chain_state {
				state,
			}) => {
				if let Some(median) = select_median(data) {
					state.median_tip = median;
				};
			},
			_ => {
				log::warn!("No witness data injection for call {:?}", self);
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{RuntimeOrigin, Validator, Witnesser};
	use cf_chains::{
		btc::{BitcoinFeeInfo, BitcoinTrackedData},
		dot::PolkadotTrackedData,
		eth::EthereumTrackedData,
		Bitcoin, Chain, Ethereum, Polkadot,
	};
	use cf_primitives::{AccountRole, ForeignChain};
	use cf_traits::EpochInfo;
	use frame_support::{assert_ok, traits::Get, Hashable};
	use pallet_cf_witnesser::CallHash;
	use sp_std::{collections::btree_set::BTreeSet, iter};

	const BLOCK_HEIGHT: u64 = 1_000;
	const BASE_FEE: u128 = 40;

	fn chain_tracking_call_with_fee<C: Chain + Get<ForeignChain>>(fee: u32) -> RuntimeCall {
		match <C as Get<ForeignChain>>::get() {
			ForeignChain::Ethereum =>
				RuntimeCall::EthereumChainTracking(pallet_cf_chain_tracking::Call::<
					Runtime,
					EthereumInstance,
				>::update_chain_state {
					state: EthereumTrackedData {
						block_height: BLOCK_HEIGHT,
						base_fee: BASE_FEE,
						priority_fee: fee.into(),
					},
				}),
			ForeignChain::Bitcoin =>
				RuntimeCall::BitcoinChainTracking(pallet_cf_chain_tracking::Call::<
					Runtime,
					BitcoinInstance,
				>::update_chain_state {
					state: BitcoinTrackedData {
						block_height: BLOCK_HEIGHT,
						btc_fee_info: BitcoinFeeInfo::new(fee.into()),
					},
				}),
			ForeignChain::Polkadot =>
				RuntimeCall::PolkadotChainTracking(pallet_cf_chain_tracking::Call::<
					Runtime,
					PolkadotInstance,
				>::update_chain_state {
					state: PolkadotTrackedData {
						block_height: BLOCK_HEIGHT as u32,
						median_tip: fee.into(),
					},
				}),
		}
	}

	#[test]
	fn test_medians_all_chains() {
		test_medians::<Ethereum>();
		test_medians::<Bitcoin>();
		test_medians::<Polkadot>();
	}

	#[track_caller]
	fn test_medians<C: Chain + Get<ForeignChain>>() {
		test_priority_fee_median::<C>(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 5);
		test_priority_fee_median::<C>(&[6, 4, 5, 10, 1, 7, 8, 9, 2, 3], 5);
		test_priority_fee_median::<C>(&[1, 2, 3, 4, 6, 6, 7, 8, 9, 10], 6);
		test_priority_fee_median::<C>(&[0, 0, 1, 1, 2, 3, 3, 4, 6], 2);
		test_priority_fee_median::<C>(&[1, 1, 1], 1);
	}

	fn test_priority_fee_median<T: Chain + Get<ForeignChain>>(fees: &[u32], expected_median: u32) {
		let mut calls =
			fees.iter().copied().map(chain_tracking_call_with_fee::<T>).collect::<Vec<_>>();

		let mut extracted_data =
			calls.iter_mut().map(|call| call.extract().unwrap()).collect::<Vec<_>>();

		let call_hashes = calls.iter().map(|call| CallHash(call.blake2_256())).collect::<Vec<_>>();
		assert!(
			iter::zip(call_hashes.iter(), call_hashes.iter().skip(1)).all(|(a, b)| a == b),
			"Call hashes should all be equal after extraction."
		);

		let mut threshold_call = calls.last().unwrap().clone();
		threshold_call.combine_and_inject(&mut extracted_data[..]);

		assert_eq!(threshold_call, chain_tracking_call_with_fee::<T>(expected_median));
	}

	#[test]
	fn test_priority_fee_witnessing() {
		frame_support::sp_io::TestExternalities::new_empty().execute_with(|| {
			assert!(
				pallet_cf_chain_tracking::ChainState::<Runtime, EthereumInstance>::get().is_none()
			);

			let calls = [1u32, 100, 12, 10, 9, 11].map(chain_tracking_call_with_fee::<Ethereum>);

			let authorities =
				(0..calls.len()).map(|i| [i as u8; 32].into()).collect::<BTreeSet<_>>();
			let current_epoch = 1;
			pallet_cf_validator::CurrentEpoch::<Runtime>::put(current_epoch);
			pallet_cf_validator::HistoricalAuthorities::<Runtime>::insert(
				current_epoch,
				&authorities,
			);

			for (index, authority_id) in authorities.into_iter().enumerate() {
				pallet_cf_account_roles::AccountRoles::<Runtime>::insert(
					&authority_id,
					AccountRole::Validator,
				);
				pallet_cf_validator::AuthorityIndex::<Runtime>::insert(
					<Validator as EpochInfo>::epoch_index(),
					&authority_id,
					index as u32,
				);
				assert_ok!(Witnesser::witness_at_epoch(
					RuntimeOrigin::signed(authority_id),
					Box::new(calls[index].clone()),
					current_epoch
				));
			}

			assert_eq!(
				pallet_cf_chain_tracking::ChainState::<Runtime, EthereumInstance>::get().unwrap(),
				EthereumTrackedData {
					block_height: BLOCK_HEIGHT,
					base_fee: BASE_FEE,
					priority_fee: 10
				}
			);
		})
	}
}

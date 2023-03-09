pub mod batch_fetch_and_transfer;

use super::{scriptpubkey_from_address, Bitcoin, BitcoinNetwork, BitcoinOutput, BtcAmount, Utxo};
use crate::*;
use frame_support::{CloneNoBound, DebugNoBound, EqNoBound, Never, PartialEqNoBound};
use sp_std::marker::PhantomData;

#[derive(CloneNoBound, DebugNoBound, PartialEqNoBound, EqNoBound, Encode, Decode, TypeInfo)]
#[scale_info(skip_type_params(Environment))]
pub enum BitcoinApi<Environment: 'static> {
	BatchFetchAndTransfer(batch_fetch_and_transfer::BatchFetchAndTransfer),
	//RotateVaultProxy(rotate_vault_proxy::RotateVaultProxy),
	//CreateAnonymousVault(create_anonymous_vault::CreateAnonymousVault),
	//ChangeGovKey(set_gov_key_with_agg_key::ChangeGovKey),
	#[doc(hidden)]
	#[codec(skip)]
	_Phantom(PhantomData<Environment>, Never),
}

impl<E> AllBatch<Bitcoin> for BitcoinApi<E>
where
	E: ChainEnvironment<<Bitcoin as Chain>::ChainAmount, Vec<Utxo>>
		+ ChainEnvironment<(), BitcoinNetwork>,
{
	fn new_unsigned(
		_fetch_params: Vec<FetchAssetParams<Bitcoin>>,
		transfer_params: Vec<TransferAssetParams<Bitcoin>>,
	) -> Result<Self, ()> {
		let bitcoin_network = <E as ChainEnvironment<(), BitcoinNetwork>>::lookup(())
			.expect("Since the lookup function always returns a some");
		let mut total_output_amount = 0;
		let mut btc_outputs = vec![];
		for transfer_param in transfer_params {
			btc_outputs.push(BitcoinOutput {
				amount: transfer_param.clone().amount.try_into().expect("Since this output comes from the AMM and if AMM math works correctly, this should be a valid bitcoin amount which should be less than u64::max"),
				script_pubkey: scriptpubkey_from_address(
					sp_std::str::from_utf8(&transfer_param.to[..]).map_err(|_| ())?,
					bitcoin_network.clone(),
				).map_err(|_|())?,
			});
			total_output_amount += transfer_param.amount;
		}
		let selected_input_utxos =
			<E as ChainEnvironment<BtcAmount, Vec<Utxo>>>::lookup(total_output_amount).ok_or(())?;

		Ok(Self::BatchFetchAndTransfer(
			batch_fetch_and_transfer::BatchFetchAndTransfer::new_unsigned(
				selected_input_utxos,
				btc_outputs,
			),
		))
	}
}

impl<E> From<batch_fetch_and_transfer::BatchFetchAndTransfer> for BitcoinApi<E> {
	fn from(tx: batch_fetch_and_transfer::BatchFetchAndTransfer) -> Self {
		Self::BatchFetchAndTransfer(tx)
	}
}

impl<E> ApiCall<Bitcoin> for BitcoinApi<E> {
	fn threshold_signature_payload(&self) -> <Bitcoin as ChainCrypto>::Payload {
		match self {
			BitcoinApi::BatchFetchAndTransfer(tx) => tx.threshold_signature_payload(),

			BitcoinApi::_Phantom(..) => unreachable!(),
		}
	}

	fn signed(self, threshold_signature: &<Bitcoin as ChainCrypto>::ThresholdSignature) -> Self {
		match self {
			BitcoinApi::BatchFetchAndTransfer(call) => call.signed(threshold_signature).into(),

			BitcoinApi::_Phantom(..) => unreachable!(),
		}
	}

	fn chain_encoded(&self) -> Vec<u8> {
		match self {
			BitcoinApi::BatchFetchAndTransfer(call) => call.chain_encoded(),

			BitcoinApi::_Phantom(..) => unreachable!(),
		}
	}

	fn is_signed(&self) -> bool {
		match self {
			BitcoinApi::BatchFetchAndTransfer(call) => call.is_signed(),

			BitcoinApi::_Phantom(..) => unreachable!(),
		}
	}
}

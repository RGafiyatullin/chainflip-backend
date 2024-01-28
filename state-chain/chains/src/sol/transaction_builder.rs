use cf_primitives::chains::Solana;

use crate::TransactionBuilder;

use super::api::SolanaApi;

pub struct SolanaTransactionBuilder;

impl<Env> TransactionBuilder<Solana, SolanaApi<Env>> for SolanaTransactionBuilder {
	fn build_transaction(_signed_call: &SolanaApi<Env>) -> <Solana as crate::Chain>::Transaction {
		unimplemented!()
	}
	fn calculate_gas_limit(_call: &SolanaApi<Env>) -> Option<ethereum_types::U256> {
		unimplemented!()
	}
	fn refresh_unsigned_data(_tx: &mut <Solana as crate::Chain>::Transaction) {
		unimplemented!()
	}
	fn requires_signature_refresh(
		_call: &SolanaApi<Env>,
		_payload: &<<Solana as crate::Chain>::ChainCrypto as crate::ChainCrypto>::Payload,
	) -> bool {
		unimplemented!()
	}
}

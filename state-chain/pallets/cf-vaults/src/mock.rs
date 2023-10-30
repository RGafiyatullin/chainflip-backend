#![cfg(test)]

use std::cell::RefCell;

use super::*;
use crate as pallet_cf_vaults;
use cf_chains::{
	mocks::{MockEthereum, MockEthereumChainCrypto},
	ApiCall, SetAggKeyWithAggKeyError,
};
use cf_primitives::{BroadcastId, ThresholdSignatureRequestId};
use cf_traits::{
	impl_mock_callback, impl_mock_chainflip, mocks::block_height_provider::BlockHeightProvider,
};
use frame_support::{
	construct_runtime, parameter_types, traits::UnfilteredDispatchable, StorageHasher,
};
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};

thread_local! {
	pub static SET_AGG_KEY_WITH_AGG_KEY_REQUIRED: RefCell<bool> = RefCell::new(true);
}

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub struct Test {
		System: frame_system,
		VaultsPallet: pallet_cf_vaults,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<5>;
}

impl_mock_chainflip!(Test);
impl_mock_callback!(RuntimeOrigin);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct MockSetAggKeyWithAggKey {
	old_key: <<MockEthereum as Chain>::ChainCrypto as ChainCrypto>::AggKey,
	new_key: <<MockEthereum as Chain>::ChainCrypto as ChainCrypto>::AggKey,
}

impl MockSetAggKeyWithAggKey {
	pub fn set_required(required: bool) {
		SET_AGG_KEY_WITH_AGG_KEY_REQUIRED.with(|cell| {
			*cell.borrow_mut() = required;
		});
	}
}

impl SetAggKeyWithAggKey<MockEthereumChainCrypto> for MockSetAggKeyWithAggKey {
	fn new_unsigned(
		old_key: Option<<<MockEthereum as Chain>::ChainCrypto as ChainCrypto>::AggKey>,
		new_key: <<MockEthereum as Chain>::ChainCrypto as ChainCrypto>::AggKey,
	) -> Result<Self, SetAggKeyWithAggKeyError> {
		if !SET_AGG_KEY_WITH_AGG_KEY_REQUIRED.with(|cell| *cell.borrow()) {
			return Err(SetAggKeyWithAggKeyError::NotRequired)
		}

		Ok(Self { old_key: old_key.ok_or(SetAggKeyWithAggKeyError::Failed)?, new_key })
	}
}

impl ApiCall<MockEthereumChainCrypto> for MockSetAggKeyWithAggKey {
	fn threshold_signature_payload(
		&self,
	) -> <<MockEthereum as Chain>::ChainCrypto as ChainCrypto>::Payload {
		unimplemented!()
	}

	fn signed(
		self,
		_threshold_signature: &<<MockEthereum as Chain>::ChainCrypto as ChainCrypto>::ThresholdSignature,
	) -> Self {
		unimplemented!()
	}

	fn chain_encoded(&self) -> Vec<u8> {
		unimplemented!()
	}

	fn is_signed(&self) -> bool {
		unimplemented!()
	}

	fn transaction_out_id(
		&self,
	) -> <<MockEthereum as Chain>::ChainCrypto as ChainCrypto>::TransactionOutId {
		todo!()
	}
}

pub struct MockBroadcaster;

impl MockBroadcaster {
	pub fn send_broadcast() {
		storage::hashed::put(&<Twox64Concat as StorageHasher>::hash, b"MockBroadcaster", &());
	}

	pub fn broadcast_sent() -> bool {
		storage::hashed::exists(&<Twox64Concat as StorageHasher>::hash, b"MockBroadcaster")
	}
}

impl Broadcaster<MockEthereum> for MockBroadcaster {
	type ApiCall = MockSetAggKeyWithAggKey;
	type Callback = MockCallback;

	fn threshold_sign_and_broadcast(
		_api_call: Self::ApiCall,
	) -> (BroadcastId, ThresholdSignatureRequestId) {
		Self::send_broadcast();
		(1, 2)
	}

	fn threshold_sign_and_broadcast_with_callback(
		_api_call: Self::ApiCall,
		_callback: Self::Callback,
	) -> (BroadcastId, ThresholdSignatureRequestId) {
		unimplemented!()
	}
}

parameter_types! {
	pub const KeygenResponseGracePeriod: u64 = 25;
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum MockRuntimeSafeMode {
	CodeRed,
	CodeGreen,
}

impl SafeMode for MockRuntimeSafeMode {
	const CODE_GREEN: Self = MockRuntimeSafeMode::CodeGreen;
	const CODE_RED: Self = MockRuntimeSafeMode::CodeRed;
}

thread_local! {
	pub static SAFE_MODE: RefCell<MockRuntimeSafeMode> = RefCell::new(MockRuntimeSafeMode::CodeGreen);
}

//pub struct MockRuntimeSafeMode;
impl SetSafeMode<MockRuntimeSafeMode> for MockRuntimeSafeMode {
	fn set_safe_mode(mode: MockRuntimeSafeMode) {
		SAFE_MODE.with(|safe_mode| *(safe_mode.borrow_mut()) = mode);
	}
}

impl Get<MockRuntimeSafeMode> for MockRuntimeSafeMode {
	fn get() -> MockRuntimeSafeMode {
		SAFE_MODE.with(|safe_mode| safe_mode.borrow().clone())
	}
}

impl pallet_cf_vaults::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Chain = MockEthereum;
	type SetAggKeyWithAggKey = MockSetAggKeyWithAggKey;
	type WeightInfo = ();
	type Broadcaster = MockBroadcaster;
	type SafeMode = MockRuntimeSafeMode;
	type ChainTracking = BlockHeightProvider<MockEthereum>;
}

cf_test_utilities::impl_test_helpers! {
	Test,
	RuntimeGenesisConfig {
		system: Default::default(),
		vaults_pallet: VaultsPalletConfig {
			deployment_block: Some(0),
		},
	},
	|| {},
}

pub(crate) fn new_test_ext_no_key() -> TestRunner<()> {
	TestRunner::<()>::new(RuntimeGenesisConfig::default())
}

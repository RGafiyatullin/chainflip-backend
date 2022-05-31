use std::cell::RefCell;

use frame_support::{
	construct_runtime, parameter_types, traits::UnfilteredDispatchable, StorageHasher,
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

use crate as pallet_cf_vaults;

use super::*;
use cf_chains::{mocks::MockEthereum, ApiCall, ChainCrypto};
use cf_traits::{
	mocks::{
		ceremony_id_provider::MockCeremonyIdProvider, epoch_info::MockEpochInfo,
		eth_environment_provider::MockEthEnvironmentProvider,
		eth_replay_protection_provider::MockEthReplayProtectionProvider,
		system_state_info::MockSystemStateInfo,
	},
	Chainflip,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<MockRuntime>;
type Block = frame_system::mocking::MockBlock<MockRuntime>;

pub type ValidatorId = u64;

thread_local! {
	pub static BAD_VALIDATORS: RefCell<Vec<ValidatorId>> = RefCell::new(vec![]);
	pub static CURRENT_SYSTEM_STATE: RefCell<SystemState> = RefCell::new(SystemState::Normal);

}

construct_runtime!(
	pub enum MockRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		VaultsPallet: pallet_cf_vaults,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

#[derive(Clone, Eq, PartialEq, Copy, Debug)]
pub enum SystemState {
	Normal,
	Maintenance,
}

// do not know how to solve this mock
pub struct MockSystemStateManager;

impl SystemStateManager for MockSystemStateManager {
	type SystemState = SystemState;
	fn set_system_state(state: Self::SystemState) {
		CURRENT_SYSTEM_STATE.with(|cell| {
			*cell.borrow_mut() = state;
		});
	}
	fn set_maintenance_mode() {
		Self::set_system_state(SystemState::Maintenance);
	}
}

impl MockSystemStateManager {
	pub fn get_current_system_state() -> SystemState {
		CURRENT_SYSTEM_STATE.with(|cell| *cell.borrow())
	}
}

impl frame_system::Config for MockRuntime {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
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

parameter_types! {}

impl Chainflip for MockRuntime {
	type KeyId = Vec<u8>;
	type ValidatorId = ValidatorId;
	type Amount = u128;
	type Call = Call;
	type EnsureWitnessed = cf_traits::mocks::ensure_origin_mock::NeverFailingOriginCheck<Self>;
	type EnsureWitnessedAtCurrentEpoch =
		cf_traits::mocks::ensure_origin_mock::NeverFailingOriginCheck<Self>;
	type EpochInfo = MockEpochInfo;
	type SystemState = MockSystemStateInfo;
}

pub struct MockCallback;

impl UnfilteredDispatchable for MockCallback {
	type Origin = Origin;

	fn dispatch_bypass_filter(
		self,
		_origin: Self::Origin,
	) -> frame_support::dispatch::DispatchResultWithPostInfo {
		Ok(().into())
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct MockSetAggKeyWithAggKey {
	nonce: <MockEthereum as ChainAbi>::ReplayProtection,
	new_key: <MockEthereum as ChainCrypto>::AggKey,
}

impl SetAggKeyWithAggKey<MockEthereum> for MockSetAggKeyWithAggKey {
	fn new_unsigned(
		nonce: <MockEthereum as ChainAbi>::ReplayProtection,
		new_key: <MockEthereum as ChainCrypto>::AggKey,
	) -> Self {
		Self { nonce, new_key }
	}
}

impl ApiCall<MockEthereum> for MockSetAggKeyWithAggKey {
	fn threshold_signature_payload(&self) -> <MockEthereum as ChainCrypto>::Payload {
		unimplemented!()
	}

	fn signed(
		self,
		_threshold_signature: &<MockEthereum as ChainCrypto>::ThresholdSignature,
	) -> Self {
		unimplemented!()
	}

	fn encoded(&self) -> Vec<u8> {
		unimplemented!()
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

	fn threshold_sign_and_broadcast(_api_call: Self::ApiCall) {
		Self::send_broadcast()
	}
}

parameter_types! {
	pub const KeygenResponseGracePeriod: u64 = 25;
}

pub type MockOffenceReporter =
	cf_traits::mocks::offence_reporting::MockOffenceReporter<ValidatorId, PalletOffence>;

impl pallet_cf_vaults::Config for MockRuntime {
	type Event = Event;
	type Offence = PalletOffence;
	type Chain = MockEthereum;
	type OffenceReporter = MockOffenceReporter;
	type ApiCall = MockSetAggKeyWithAggKey;
	type CeremonyIdProvider = MockCeremonyIdProvider<CeremonyId>;
	type WeightInfo = ();
	type KeygenResponseGracePeriod = KeygenResponseGracePeriod;
	type Broadcaster = MockBroadcaster;
	type EthEnvironmentProvider = MockEthEnvironmentProvider;
	type ReplayProtectionProvider = MockEthReplayProtectionProvider<MockEthereum>;
	type SystemStateManager = MockSystemStateManager;
}

pub const ALICE: <MockRuntime as frame_system::Config>::AccountId = 123u64;
pub const BOB: <MockRuntime as frame_system::Config>::AccountId = 456u64;
pub const CHARLIE: <MockRuntime as frame_system::Config>::AccountId = 789u64;
pub const GENESIS_AGG_PUB_KEY: [u8; 33] = [0x2; 33];

pub fn new_agg_pub_key() -> <MockEthereum as ChainCrypto>::AggKey {
	<MockEthereum as ChainCrypto>::AggKey::from_pubkey_compressed([0x3; 33])
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let config = GenesisConfig {
		system: Default::default(),
		vaults_pallet: VaultsPalletConfig {
			vault_key: GENESIS_AGG_PUB_KEY.to_vec(),
			deployment_block: 0,
		},
	};

	let authorities = vec![ALICE, BOB, CHARLIE];
	MockEpochInfo::set_epoch_authority_count(0, authorities.len() as AuthorityCount);
	MockEpochInfo::set_authorities(authorities);

	let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();

	ext.execute_with(|| {
		System::set_block_number(1);
	});

	ext
}

use crate as pallet_cf_emissions;
use cf_chains::{mocks::MockEthereum, AnyChain, ApiCall, ChainCrypto, UpdateFlipSupply};
use cf_primitives::{BroadcastId, FlipBalance, ThresholdSignatureRequestId};
use cf_traits::{
	impl_mock_callback, impl_mock_chainflip, impl_mock_waived_fees,
	mocks::{egress_handler::MockEgressHandler, eth_environment_provider::MockEthEnvironment},
	Broadcaster, FlipBurnInfo, Issuance, RewardsDistribution, WaivedFees,
};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	parameter_types, storage,
	traits::{Imbalance, UnfilteredDispatchable},
	StorageHasher, Twox64Concat,
};
use frame_system as system;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub type AccountId = u64;

pub const FLIP_TO_BURN: u128 = 10_000;
pub const SUPPLY_UPDATE_INTERVAL: u32 = 10;
pub const TOTAL_ISSUANCE: u128 = 1_000_000_000;

cf_traits::impl_mock_on_account_funded!(AccountId, u128);

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Flip: pallet_cf_flip,
		Emissions: pallet_cf_emissions,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<5>;
}

impl_mock_chainflip!(Test);
impl_mock_callback!(RuntimeOrigin);

parameter_types! {
	pub const ExistentialDeposit: u128 = 10;
}

parameter_types! {
	pub const BlocksPerDay: u64 = 14400;
}

parameter_types! {
	pub const HeartbeatBlockInterval: u64 = 150;
}

// Implement mock for RestrictionHandler
impl_mock_waived_fees!(AccountId, RuntimeCall);

impl pallet_cf_flip::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u128;
	type ExistentialDeposit = ExistentialDeposit;
	type BlocksPerDay = BlocksPerDay;
	type OnAccountFunded = MockOnAccountFunded;
	type WeightInfo = ();
	type WaivedFees = WaivedFeesMock;
}

pub const EMISSION_RATE: u128 = 10;
pub struct MockRewardsDistribution;

impl RewardsDistribution for MockRewardsDistribution {
	type Balance = u128;
	type Issuance = pallet_cf_flip::FlipIssuance<Test>;

	fn distribute() {
		let deposit =
			Flip::deposit_reserves(*b"RSVR", Emissions::current_authority_emission_per_block());
		let amount = deposit.peek();
		let _result = deposit.offset(Self::Issuance::mint(amount));
	}
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct MockUpdateFlipSupply {
	pub new_total_supply: u128,
	pub block_number: u64,
}

impl UpdateFlipSupply<MockEthereum> for MockUpdateFlipSupply {
	fn new_unsigned(new_total_supply: u128, block_number: u64) -> Self {
		Self { new_total_supply, block_number }
	}
}

impl ApiCall<MockEthereum> for MockUpdateFlipSupply {
	fn threshold_signature_payload(&self) -> <MockEthereum as ChainCrypto>::Payload {
		[0xcf; 4]
	}

	fn signed(
		self,
		_threshold_signature: &<MockEthereum as ChainCrypto>::ThresholdSignature,
	) -> Self {
		unimplemented!()
	}

	fn chain_encoded(&self) -> Vec<u8> {
		unimplemented!()
	}

	fn is_signed(&self) -> bool {
		unimplemented!()
	}
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct MockBroadcast;

pub struct MockFlipToBurn;

impl FlipBurnInfo for MockFlipToBurn {
	fn take_flip_to_burn() -> cf_primitives::AssetAmount {
		FLIP_TO_BURN
	}
}

impl MockBroadcast {
	pub fn call(outgoing: MockUpdateFlipSupply) {
		storage::hashed::put(&<Twox64Concat as StorageHasher>::hash, b"MockBroadcast", &outgoing);
	}

	pub fn get_called() -> Option<MockUpdateFlipSupply> {
		storage::hashed::get(&<Twox64Concat as StorageHasher>::hash, b"MockBroadcast")
	}
}

impl Broadcaster<MockEthereum> for MockBroadcast {
	type ApiCall = MockUpdateFlipSupply;
	type Callback = MockCallback;

	fn threshold_sign_and_broadcast(
		api_call: Self::ApiCall,
	) -> (BroadcastId, ThresholdSignatureRequestId) {
		Self::call(api_call);
		(1, 2)
	}

	fn threshold_sign_and_broadcast_with_callback(
		_api_call: Self::ApiCall,
		_callback: Self::Callback,
	) -> (BroadcastId, ThresholdSignatureRequestId) {
		unimplemented!()
	}
}

impl pallet_cf_emissions::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type HostChain = MockEthereum;
	type FlipBalance = FlipBalance;
	type ApiCall = MockUpdateFlipSupply;
	type Surplus = pallet_cf_flip::Surplus<Test>;
	type Issuance = pallet_cf_flip::FlipIssuance<Test>;
	type RewardsDistribution = MockRewardsDistribution;
	type CompoundingInterval = HeartbeatBlockInterval;
	type EthEnvironment = MockEthEnvironment;
	type Broadcaster = MockBroadcast;
	type WeightInfo = ();
	type FlipToBurn = MockFlipToBurn;
	type EgressHandler = MockEgressHandler<AnyChain>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(validators: Vec<u64>, issuance: Option<u128>) -> sp_io::TestExternalities {
	let total_issuance = issuance.unwrap_or(TOTAL_ISSUANCE);
	let config = GenesisConfig {
		system: Default::default(),
		flip: FlipConfig { total_issuance },
		emissions: {
			EmissionsConfig {
				current_authority_emission_inflation: 2720,
				backup_node_emission_inflation: 284,
				supply_update_interval: SUPPLY_UPDATE_INTERVAL,
			}
		},
	};

	for v in validators {
		MockEpochInfo::add_authorities(v);
	}

	config.build_storage().unwrap().into()
}

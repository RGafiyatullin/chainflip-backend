use crate as pallet_cf_staking;
use cf_chains::{eth, ChainAbi, ChainCrypto, Ethereum};
use cf_traits::{impl_mock_waived_fees, AsyncResult, ThresholdSigner, WaivedFees};
use frame_support::{dispatch::DispatchResultWithPostInfo, parameter_types};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, BuildStorage,
};
use std::time::Duration;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
// Use a realistic account id for compatibility with `RegisterClaim`.
type AccountId = AccountId32;

use cf_traits::{
	mocks::{ensure_origin_mock::NeverFailingOriginCheck, time_source},
	Chainflip, NonceProvider,
};

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Flip: pallet_cf_flip::{Pallet, Call, Config<T>, Storage, Event<T>},
		Staking: pallet_cf_staking::{Pallet, Call, Config<T>, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = sp_core::H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

impl Chainflip for Test {
	type KeyId = Vec<u8>;
	type ValidatorId = AccountId;
	type Amount = u128;
	type Call = Call;
	type EnsureWitnessed = MockEnsureWitnessed;
	type EpochInfo = MockEpochInfo;
}

parameter_types! {
	pub const ThresholdFailureTimeout: <Test as frame_system::Config>::BlockNumber = 10;
	pub const CeremonyRetryDelay: <Test as frame_system::Config>::BlockNumber = 1;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 10;
}

parameter_types! {
	pub const BlocksPerDay: u64 = 14400;
}

// Implement mock for RestrictionHandler
impl_mock_waived_fees!(AccountId, Call);

impl pallet_cf_flip::Config for Test {
	type Event = Event;
	type Balance = u128;
	type ExistentialDeposit = ExistentialDeposit;
	type EnsureGovernance = NeverFailingOriginCheck<Self>;
	type BlocksPerDay = BlocksPerDay;
	type StakeHandler = MockStakeHandler;
	type WeightInfo = ();
	type WaivedFees = WaivedFeesMock;
}

cf_traits::impl_mock_ensure_witnessed_for_origin!(Origin);
cf_traits::impl_mock_witnesser_for_account_and_call_types!(AccountId, Call, u64);
cf_traits::impl_mock_epoch_info!(AccountId, u128, u32);
cf_traits::impl_mock_stake_transfer!(AccountId, u128);

pub const NONCE: u64 = 42;

impl NonceProvider<Ethereum> for Test {
	fn next_nonce() -> <Ethereum as ChainAbi>::Nonce {
		NONCE
	}
}

pub struct MockThresholdSigner;

thread_local! {
	pub static SIGNATURE_REQUESTS: RefCell<Vec<<Ethereum as ChainCrypto>::Payload>> = RefCell::new(vec![]);
}

impl MockThresholdSigner {
	pub fn received_requests() -> Vec<<Ethereum as ChainCrypto>::Payload> {
		SIGNATURE_REQUESTS.with(|cell| cell.borrow().clone())
	}

	pub fn on_signature_ready(account_id: &AccountId) -> DispatchResultWithPostInfo {
		Staking::post_claim_signature(Origin::root(), account_id.clone(), 0)
	}
}

impl ThresholdSigner<Ethereum> for MockThresholdSigner {
	type RequestId = u32;
	type Error = &'static str;
	type Callback = Call;

	fn request_signature(payload: <Ethereum as ChainCrypto>::Payload) -> Self::RequestId {
		SIGNATURE_REQUESTS.with(|cell| cell.borrow_mut().push(payload));
		0
	}

	fn register_callback(_: Self::RequestId, _: Self::Callback) -> Result<(), Self::Error> {
		Ok(())
	}

	fn signature_result(
		_: Self::RequestId,
	) -> cf_traits::AsyncResult<<Ethereum as ChainCrypto>::ThresholdSignature> {
		AsyncResult::Ready(ETH_DUMMY_SIG)
	}
}

// The dummy signature can't be Default - this would be interpreted as no signature.
pub const ETH_DUMMY_SIG: eth::SchnorrVerificationComponents =
	eth::SchnorrVerificationComponents { s: [0xcf; 32], k_times_g_addr: [0xcf; 20] };

impl pallet_cf_staking::Config for Test {
	type Event = Event;
	type TimeSource = time_source::Mock;
	type Balance = u128;
	type Flip = Flip;
	type WeightInfo = ();
	type StakerId = AccountId;
	type NonceProvider = Self;
	type ThresholdSigner = MockThresholdSigner;
	type ThresholdCallable = Call;
	type EnsureThresholdSigned = NeverFailingOriginCheck<Self>;
	type EnsureGovernance = NeverFailingOriginCheck<Self>;
	type RegisterClaim = eth::api::EthereumApi;
}

pub const ALICE: AccountId = AccountId32::new([0xa1; 32]);
pub const BOB: AccountId = AccountId32::new([0xb0; 32]);
pub const MIN_STAKE: u128 = 10;
// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let config = GenesisConfig {
		system: Default::default(),
		flip: FlipConfig { total_issuance: 1_000 },
		staking: StakingConfig {
			genesis_stakers: vec![],
			minimum_stake: MIN_STAKE,
			claim_ttl: Duration::from_secs(10),
		},
	};

	let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();

	ext.execute_with(|| {
		System::set_block_number(1);
	});

	ext
}

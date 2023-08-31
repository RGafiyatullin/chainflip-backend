pub use crate::{self as pallet_cf_ingress_egress};
use crate::{DepositBalances, DepositWitness};

pub use cf_chains::{
	address::{AddressDerivationApi, ForeignChainAddress},
	eth::{api::EthereumApi, Address as EthereumAddress},
	CcmDepositMetadata, Chain, ChainEnvironment, DepositChannel,
};
use cf_primitives::ChannelId;
pub use cf_primitives::{
	chains::{assets, Ethereum},
	Asset, AssetAmount,
};
use cf_test_utilities::{impl_test_helpers, TestExternalities};
use cf_traits::{
	impl_mock_callback, impl_mock_chainflip,
	mocks::{
		address_converter::MockAddressConverter,
		api_call::{MockEthEnvironment, MockEthereumApiCall},
		broadcaster::MockBroadcaster,
		ccm_handler::MockCcmHandler,
		swap_deposit_handler::MockSwapDepositHandler,
	},
	DepositApi, DepositHandler, GetBlockHeight,
};
use frame_support::traits::{OriginTrait, UnfilteredDispatchable};
use frame_system as system;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup, Zero};

type AccountId = u64;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system,
		IngressEgress: pallet_cf_ingress_egress,
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = frame_support::traits::ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = frame_support::traits::ConstU16<2112>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<5>;
}

impl_mock_chainflip!(Test);
impl_mock_callback!(RuntimeOrigin);

pub struct MockDepositHandler;
impl DepositHandler<Ethereum> for MockDepositHandler {}

pub type MockEgressBroadcaster =
	MockBroadcaster<(MockEthereumApiCall<MockEthEnvironment>, RuntimeCall)>;

pub struct BlockNumberProvider;

pub const OPEN_INGRESS_AT: u64 = 420;

impl GetBlockHeight<Ethereum> for BlockNumberProvider {
	fn get_block_height() -> u64 {
		OPEN_INGRESS_AT
	}
}

pub struct MockAddressDerivation;

impl AddressDerivationApi<Ethereum> for MockAddressDerivation {
	fn generate_address(
		_source_asset: assets::eth::Asset,
		channel_id: ChannelId,
	) -> Result<<Ethereum as Chain>::ChainAccount, sp_runtime::DispatchError> {
		Ok([channel_id as u8; 20].into())
	}

	fn generate_address_and_state(
		source_asset: <Ethereum as Chain>::ChainAsset,
		channel_id: ChannelId,
	) -> Result<
		(<Ethereum as Chain>::ChainAccount, <Ethereum as Chain>::DepositChannelState),
		sp_runtime::DispatchError,
	> {
		Ok((Self::generate_address(source_asset, channel_id)?, Default::default()))
	}
}

impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type TargetChain = Ethereum;
	type AddressDerivation = MockAddressDerivation;
	type AddressConverter = MockAddressConverter;
	type LpBalance = Self;
	type SwapDepositHandler =
		MockSwapDepositHandler<(Ethereum, pallet_cf_ingress_egress::Pallet<Self>)>;
	type ChainApiCall = MockEthereumApiCall<MockEthEnvironment>;
	type Broadcaster = MockEgressBroadcaster;
	type DepositHandler = MockDepositHandler;
	type CcmHandler = MockCcmHandler;
	type ChainTracking = BlockNumberProvider;
	type WeightInfo = ();
}

pub const ALICE: <Test as frame_system::Config>::AccountId = 123u64;
pub const BROKER: <Test as frame_system::Config>::AccountId = 456u64;

// Configure a mock runtime to test the pallet.
impl_test_helpers!(Test);

type TestChainAccount = <<Test as crate::Config>::TargetChain as Chain>::ChainAccount;
type TestChainAmount = <<Test as crate::Config>::TargetChain as Chain>::ChainAmount;
type TestChainAsset = <<Test as crate::Config>::TargetChain as Chain>::ChainAsset;

pub trait RequestAddressAndDeposit {
	fn request_address_and_deposit(
		self,
		requests: &[(DepositRequest, TestChainAmount)],
	) -> TestExternalities<Test, Vec<(DepositRequest, ChannelId, TestChainAccount)>>;
}

impl<Ctx: Clone> RequestAddressAndDeposit for TestRunner<Ctx> {
	#[track_caller]
	fn request_address_and_deposit(
		self,
		requests: &[(DepositRequest, TestChainAmount)],
	) -> TestExternalities<Test, Vec<(DepositRequest, ChannelId, TestChainAccount)>> {
		let (requests, amounts): (Vec<_>, Vec<_>) = requests.iter().cloned().unzip();

		self.request_deposit_addresses(&requests[..])
			.then_apply_extrinsics(move |channels| {
				channels
					.iter()
					.zip(amounts)
					.filter_map(|((request, _channel_id, deposit_address), amount)| {
						(!amount.is_zero()).then_some((
							OriginTrait::none(),
							RuntimeCall::from(pallet_cf_ingress_egress::Call::process_deposits {
								deposit_witnesses: vec![DepositWitness {
									deposit_address: *deposit_address,
									asset: request.source_asset(),
									amount,
									deposit_details: Default::default(),
								}],
								block_height: Default::default(),
							}),
							Ok(()),
						))
					})
					.collect::<Vec<_>>()
			})
	}
}

#[derive(Clone, Debug)]
pub enum DepositRequest {
	Liquidity {
		lp_account: AccountId,
		asset: TestChainAsset,
		expiry_block: BlockNumberFor<Test>,
	},
	/// Do a non-ccm swap using a default broker and no fees.
	SimpleSwap {
		source_asset: TestChainAsset,
		destination_asset: TestChainAsset,
		destination_address: ForeignChainAddress,
		expiry_block: BlockNumberFor<Test>,
	},
}

impl DepositRequest {
	pub fn source_asset(&self) -> TestChainAsset {
		match self {
			Self::Liquidity { asset, .. } => *asset,
			Self::SimpleSwap { source_asset, .. } => *source_asset,
		}
	}
}

pub trait RequestAddress {
	fn request_deposit_addresses(
		self,
		requests: &[DepositRequest],
	) -> TestExternalities<Test, Vec<(DepositRequest, ChannelId, TestChainAccount)>>;
}

impl<Ctx: Clone> RequestAddress for TestExternalities<Test, Ctx> {
	#[track_caller]
	fn request_deposit_addresses(
		self,
		requests: &[DepositRequest],
	) -> TestExternalities<Test, Vec<(DepositRequest, ChannelId, TestChainAccount)>> {
		self.then_execute_at_next_block(|_| {
			requests
				.iter()
				.cloned()
				.map(|request| match request {
					DepositRequest::Liquidity { lp_account, asset, expiry_block } =>
						IngressEgress::request_liquidity_deposit_address(
							lp_account,
							asset,
							expiry_block,
						)
						.map(|(id, addr)| (request, id, TestChainAccount::try_from(addr).unwrap()))
						.unwrap(),
					DepositRequest::SimpleSwap {
						source_asset,
						destination_asset,
						ref destination_address,
						expiry_block,
					} => IngressEgress::request_swap_deposit_address(
						source_asset,
						destination_asset.into(),
						destination_address.clone(),
						Default::default(),
						BROKER,
						None,
						expiry_block,
					)
					.map(|(channel_id, deposit_address)| {
						(request, channel_id, TestChainAccount::try_from(deposit_address).unwrap())
					})
					.unwrap(),
				})
				.collect()
		})
	}
}

pub trait CheckDepositBalances {
	fn check_deposit_balances(
		self,
		expected_balances: &[(TestChainAsset, TestChainAmount)],
	) -> Self;
}

impl<Ctx: Clone> CheckDepositBalances for TestExternalities<Test, Ctx> {
	#[track_caller]
	fn check_deposit_balances(
		self,
		expected_balances: &[(TestChainAsset, TestChainAmount)],
	) -> Self {
		self.inspect_storage(|_| {
			for (asset, expected_balance) in expected_balances {
				assert_eq!(
					DepositBalances::<Test, _>::get(asset).total(),
					*expected_balance,
					"Unexpected balance for {asset:?}. Expected {expected_balance}, got {:?}.",
					DepositBalances::<Test, _>::get(asset)
				);
			}
		})
	}
}

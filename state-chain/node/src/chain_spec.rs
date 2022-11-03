use cf_primitives::AccountRole;
use sc_service::{ChainType, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{
	crypto::{set_default_ss58_version, Ss58AddressFormat, UncheckedInto},
	sr25519, Pair, Public,
};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use state_chain_runtime::{
	constants::common::*, opaque::SessionKeys, AccountId, AccountRolesConfig, AuraConfig,
	BlockNumber, CfeSettings, EmissionsConfig, EnvironmentConfig, EthereumThresholdSignerConfig,
	EthereumVaultConfig, FlipBalance, FlipConfig, GenesisConfig, GovernanceConfig, GrandpaConfig,
	ReputationConfig, SessionConfig, Signature, StakingConfig, SystemConfig, ValidatorConfig,
	WASM_BINARY,
};
use std::{collections::BTreeSet, env, marker::PhantomData};
use utilities::clean_eth_address;

pub mod perseverance;
pub mod sisyphos;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

const FLIP_TOKEN_ADDRESS_DEFAULT: &str = "Cf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9";
const ETH_USDC_ADDRESS_DEFAULT: &str = "a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
const STAKE_MANAGER_ADDRESS_DEFAULT: &str = "9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0";
const KEY_MANAGER_ADDRESS_DEFAULT: &str = "5FbDB2315678afecb367f032d93F642f64180aa3";
const ETH_VAULT_ADDRESS_DEFAULT: &str = "e7f1725E7734CE288F8367e1Bb143E90bb3F0512";
const ETHEREUM_CHAIN_ID_DEFAULT: u64 = cf_chains::eth::CHAIN_ID_GOERLI;
const ETH_INIT_AGG_KEY_DEFAULT: &str =
	"02e61afd677cdfbec838c6f309deff0b2c6056f8a27f2c783b68bba6b30f667be6";
const GENESIS_STAKE_AMOUNT_DEFAULT: FlipBalance = 5_000 * FLIPPERINOS_PER_FLIP;
const ETH_DEPLOYMENT_BLOCK_DEFAULT: u64 = 0;
const ETH_PRIORITY_FEE_PERCENTILE_DEFAULT: u8 = 50;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// generate session keys from Aura and Grandpa keys
pub fn session_keys(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
	SessionKeys { aura, grandpa }
}

pub struct StateChainEnvironment {
	flip_token_address: [u8; 20],
	eth_usdc_address: [u8; 20],
	stake_manager_address: [u8; 20],
	key_manager_address: [u8; 20],
	eth_vault_address: [u8; 20],
	ethereum_chain_id: u64,
	eth_init_agg_key: [u8; 33],
	ethereum_deployment_block: u64,
	genesis_stake_amount: u128,
	/// Note: Minimum stake should be expressed in Flipperinos.
	min_stake: u128,
	// CFE config values starts here
	eth_block_safety_margin: u32,
	max_ceremony_stage_duration: u32,
}
/// Get the values from the State Chain's environment variables. Else set them via the defaults
pub fn get_environment() -> StateChainEnvironment {
	let flip_token_address: [u8; 20] = clean_eth_address(
		&env::var("FLIP_TOKEN_ADDRESS")
			.unwrap_or_else(|_| String::from(FLIP_TOKEN_ADDRESS_DEFAULT)),
	)
	.unwrap();
	let eth_usdc_address: [u8; 20] = clean_eth_address(
		&env::var("ETH_USDC_ADDRESS").unwrap_or_else(|_| String::from(ETH_USDC_ADDRESS_DEFAULT)),
	)
	.unwrap();
	let stake_manager_address: [u8; 20] = clean_eth_address(
		&env::var("STAKE_MANAGER_ADDRESS")
			.unwrap_or_else(|_| String::from(STAKE_MANAGER_ADDRESS_DEFAULT)),
	)
	.unwrap();
	let key_manager_address: [u8; 20] = clean_eth_address(
		&env::var("KEY_MANAGER_ADDRESS")
			.unwrap_or_else(|_| String::from(KEY_MANAGER_ADDRESS_DEFAULT)),
	)
	.unwrap();
	let eth_vault_address: [u8; 20] = clean_eth_address(
		&env::var("ETH_VAULT_ADDRESS").unwrap_or_else(|_| String::from(ETH_VAULT_ADDRESS_DEFAULT)),
	)
	.unwrap();
	let ethereum_chain_id = env::var("ETHEREUM_CHAIN_ID")
		.unwrap_or_else(|_| ETHEREUM_CHAIN_ID_DEFAULT.to_string())
		.parse::<u64>()
		.expect("ETHEREUM_CHAIN_ID env var could not be parsed to u64");
	let eth_init_agg_key = hex::decode(
		env::var("ETH_INIT_AGG_KEY").unwrap_or_else(|_| String::from(ETH_INIT_AGG_KEY_DEFAULT)),
	)
	.unwrap()
	.try_into()
	.expect("ETH_INIT_AGG_KEY cast to agg pub key failed");
	let ethereum_deployment_block = env::var("ETH_DEPLOYMENT_BLOCK")
		.unwrap_or_else(|_| ETH_DEPLOYMENT_BLOCK_DEFAULT.to_string())
		.parse::<u64>()
		.expect("ETH_DEPLOYMENT_BLOCK env var could not be parsed to u64");

	let genesis_stake_amount = env::var("GENESIS_STAKE")
		.unwrap_or_else(|_| GENESIS_STAKE_AMOUNT_DEFAULT.to_string())
		.parse::<u128>()
		.expect("GENESIS_STAKE env var could not be parsed to u128");

	let eth_block_safety_margin = env::var("ETH_BLOCK_SAFETY_MARGIN")
		.unwrap_or_else(|_| CfeSettings::default().eth_block_safety_margin.to_string())
		.parse::<u32>()
		.expect("ETH_BLOCK_SAFETY_MARGIN env var could not be parsed to u32");

	let max_ceremony_stage_duration = env::var("MAX_CEREMONY_STAGE_DURATION")
		.unwrap_or_else(|_| CfeSettings::default().max_ceremony_stage_duration.to_string())
		.parse::<u32>()
		.expect("MAX_CEREMONY_STAGE_DURATION env var could not be parsed to u32");

	let min_stake: u128 = env::var("MIN_STAKE")
		.map(|s| s.parse::<u128>().expect("MIN_STAKE env var could not be parsed to u128"))
		.unwrap_or(DEFAULT_MIN_STAKE);

	StateChainEnvironment {
		flip_token_address,
		eth_usdc_address,
		stake_manager_address,
		key_manager_address,
		eth_vault_address,
		ethereum_chain_id,
		eth_init_agg_key,
		ethereum_deployment_block,
		genesis_stake_amount,
		eth_block_safety_margin,
		max_ceremony_stage_duration,
		min_stake,
	}
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AuraId, GrandpaId) {
	(
		get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
	)
}

/// Start a single node development chain
pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary =
		WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;
	let StateChainEnvironment {
		flip_token_address,
		eth_usdc_address,
		stake_manager_address,
		key_manager_address,
		eth_vault_address,
		ethereum_chain_id,
		eth_init_agg_key,
		ethereum_deployment_block,
		genesis_stake_amount,
		eth_block_safety_margin,
		max_ceremony_stage_duration,
		min_stake,
	} = get_environment();
	Ok(ChainSpec::from_genesis(
		"Develop",
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Governance account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Stakers at genesis
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				1,
				EnvironmentConfig {
					flip_token_address,
					eth_usdc_address,
					stake_manager_address,
					key_manager_address,
					eth_vault_address,
					ethereum_chain_id,
					cfe_settings: CfeSettings {
						eth_block_safety_margin,
						max_ceremony_stage_duration,
						eth_priority_fee_percentile: ETH_PRIORITY_FEE_PERCENTILE_DEFAULT,
					},
				},
				eth_init_agg_key,
				ethereum_deployment_block,
				genesis_stake_amount,
				min_stake,
				8 * HOURS,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Fork ID
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

/// Start a single node development chain - using bashful as genesis node
pub fn cf_development_config() -> Result<ChainSpec, String> {
	let wasm_binary =
		WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

	let snow_white =
		hex_literal::hex!["ced2e4db6ce71779ac40ccec60bf670f38abbf9e27a718b4412060688a9ad212"];
	let bashful_sr25519 =
		hex_literal::hex!["36c0078af3894b8202b541ece6c5d8fb4a091f7e5812b688e703549040473911"];
	let StateChainEnvironment {
		flip_token_address,
		eth_usdc_address,
		stake_manager_address,
		key_manager_address,
		eth_vault_address,
		ethereum_chain_id,
		eth_init_agg_key,
		ethereum_deployment_block,
		genesis_stake_amount,
		eth_block_safety_margin,
		max_ceremony_stage_duration,
		min_stake,
	} = get_environment();
	Ok(ChainSpec::from_genesis(
		"CF Develop",
		"cf-dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![(
					// Bashful
					bashful_sr25519.into(),
					bashful_sr25519.unchecked_into(),
					hex_literal::hex![
						"971b584324592e9977f0ae407eb6b8a1aa5bcd1ca488e54ab49346566f060dd8"
					]
					.unchecked_into(),
				)],
				// Governance account - Snow White
				snow_white.into(),
				// Stakers at genesis
				vec![
					// Bashful
					bashful_sr25519.into(),
					#[cfg(feature = "ibiza")]
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					#[cfg(feature = "ibiza")]
					get_account_id_from_seed::<sr25519::Public>("Bob"),
				],
				1,
				EnvironmentConfig {
					flip_token_address,
					eth_usdc_address,
					stake_manager_address,
					key_manager_address,
					eth_vault_address,
					ethereum_chain_id,
					cfe_settings: CfeSettings {
						eth_block_safety_margin,
						max_ceremony_stage_duration,
						eth_priority_fee_percentile: ETH_PRIORITY_FEE_PERCENTILE_DEFAULT,
					},
				},
				eth_init_agg_key,
				ethereum_deployment_block,
				genesis_stake_amount,
				min_stake,
				8 * HOURS,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Fork ID
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

/// Initialise a Chainflip three-node testnet from the environment.
pub fn chainflip_three_node_testnet_config() -> Result<ChainSpec, String> {
	chainflip_three_node_testnet_config_from_env(
		"Three node testnet",
		"three-node-testnet",
		ChainType::Local,
		get_environment(),
	)
}

fn chainflip_three_node_testnet_config_from_env(
	name: &str,
	id: &str,
	chain_type: ChainType,
	environment: StateChainEnvironment,
) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Wasm binary not available".to_string())?;
	let bashful_sr25519 =
		hex_literal::hex!["36c0078af3894b8202b541ece6c5d8fb4a091f7e5812b688e703549040473911"];
	let doc_sr25519 =
		hex_literal::hex!["8898758bf88855615d459f552e36bfd14e8566c8b368f6a6448942759d5c7f04"];
	let dopey_sr25519 =
		hex_literal::hex!["ca58f2f4ae713dbb3b4db106640a3db150e38007940dfe29e6ebb870c4ccd47e"];
	let snow_white =
		hex_literal::hex!["ced2e4db6ce71779ac40ccec60bf670f38abbf9e27a718b4412060688a9ad212"];
	let StateChainEnvironment {
		flip_token_address,
		eth_usdc_address,
		stake_manager_address,
		key_manager_address,
		eth_vault_address,
		ethereum_chain_id,
		eth_init_agg_key,
		ethereum_deployment_block,
		genesis_stake_amount,
		eth_block_safety_margin,
		max_ceremony_stage_duration,
		min_stake,
	} = environment;
	Ok(ChainSpec::from_genesis(
		name,
		id,
		chain_type,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![
					(
						// Bashful
						bashful_sr25519.into(),
						bashful_sr25519.unchecked_into(),
						hex_literal::hex![
							"971b584324592e9977f0ae407eb6b8a1aa5bcd1ca488e54ab49346566f060dd8"
						]
						.unchecked_into(),
					),
					(
						// Doc
						doc_sr25519.into(),
						doc_sr25519.unchecked_into(),
						hex_literal::hex![
							"e4c4009bd437cba06a2f25cf02f4efc0cac4525193a88fe1d29196e5d0ff54e8"
						]
						.unchecked_into(),
					),
					(
						// Dopey
						dopey_sr25519.into(),
						dopey_sr25519.unchecked_into(),
						hex_literal::hex![
							"5506333c28f3dd39095696362194f69893bc24e3ec553dbff106cdcbfe1beea4"
						]
						.unchecked_into(),
					),
				],
				// Governance account - Snow White
				snow_white.into(),
				// Stakers at genesis
				vec![
					// Bashful
					bashful_sr25519.into(),
					// Doc
					doc_sr25519.into(),
					// Dopey
					dopey_sr25519.into(),
					#[cfg(feature = "ibiza")]
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					#[cfg(feature = "ibiza")]
					get_account_id_from_seed::<sr25519::Public>("Bob"),
				],
				2,
				EnvironmentConfig {
					flip_token_address,
					eth_usdc_address,
					stake_manager_address,
					key_manager_address,
					eth_vault_address,
					ethereum_chain_id,
					cfe_settings: CfeSettings {
						eth_block_safety_margin,
						max_ceremony_stage_duration,
						eth_priority_fee_percentile: ETH_PRIORITY_FEE_PERCENTILE_DEFAULT,
					},
				},
				eth_init_agg_key,
				ethereum_deployment_block,
				genesis_stake_amount,
				min_stake,
				8 * HOURS,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Fork ID
		None,
		// Properties
		Some(chainflip_properties()),
		// Extensions
		None,
	))
}

/// Initialise a Chainflip testnet
pub fn chainflip_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary =
		WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;
	let bashful_sr25519 =
		hex_literal::hex!["36c0078af3894b8202b541ece6c5d8fb4a091f7e5812b688e703549040473911"];
	let doc_sr25519 =
		hex_literal::hex!["8898758bf88855615d459f552e36bfd14e8566c8b368f6a6448942759d5c7f04"];
	let dopey_sr25519 =
		hex_literal::hex!["ca58f2f4ae713dbb3b4db106640a3db150e38007940dfe29e6ebb870c4ccd47e"];
	let grumpy_sr25519 =
		hex_literal::hex!["28b5f5f1654393975f58e78cf06b6f3ab509b3629b0a4b08aaa3dce6bf6af805"];
	let happy_sr25519 =
		hex_literal::hex!["7e6eb0b15c1767360fdad63d6ff78a97374355b00b4d3511a522b1a8688a661d"];
	let snow_white =
		hex_literal::hex!["ced2e4db6ce71779ac40ccec60bf670f38abbf9e27a718b4412060688a9ad212"];
	let StateChainEnvironment {
		flip_token_address,
		eth_usdc_address,
		stake_manager_address,
		key_manager_address,
		eth_vault_address,
		ethereum_chain_id,
		eth_init_agg_key,
		ethereum_deployment_block,
		genesis_stake_amount,
		eth_block_safety_margin,
		max_ceremony_stage_duration,
		min_stake,
	} = get_environment();
	Ok(ChainSpec::from_genesis(
		"Internal testnet",
		"test",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![
					(
						// Bashful
						bashful_sr25519.into(),
						bashful_sr25519.unchecked_into(),
						hex_literal::hex![
							"971b584324592e9977f0ae407eb6b8a1aa5bcd1ca488e54ab49346566f060dd8"
						]
						.unchecked_into(),
					),
					(
						// Doc
						doc_sr25519.into(),
						doc_sr25519.unchecked_into(),
						hex_literal::hex![
							"e4c4009bd437cba06a2f25cf02f4efc0cac4525193a88fe1d29196e5d0ff54e8"
						]
						.unchecked_into(),
					),
					(
						// Dopey
						dopey_sr25519.into(),
						dopey_sr25519.unchecked_into(),
						hex_literal::hex![
							"5506333c28f3dd39095696362194f69893bc24e3ec553dbff106cdcbfe1beea4"
						]
						.unchecked_into(),
					),
					(
						// Grumpy
						grumpy_sr25519.into(),
						grumpy_sr25519.unchecked_into(),
						hex_literal::hex![
							"b9036620f103cce552edbdd15e54810c6c3906975f042e3ff949af075636007f"
						]
						.unchecked_into(),
					),
					(
						// Happy
						happy_sr25519.into(),
						happy_sr25519.unchecked_into(),
						hex_literal::hex![
							"0bb5e73112e716dc54541e87d2287f2252fd479f166969dc37c07a504000dae9"
						]
						.unchecked_into(),
					),
				],
				// Governance account - Snow White
				snow_white.into(),
				// Stakers at genesis
				vec![
					// Bashful
					bashful_sr25519.into(),
					// Doc
					doc_sr25519.into(),
					// Dopey
					dopey_sr25519.into(),
					// Grumpy
					grumpy_sr25519.into(),
					// Happy
					happy_sr25519.into(),
					#[cfg(feature = "ibiza")]
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					#[cfg(feature = "ibiza")]
					get_account_id_from_seed::<sr25519::Public>("Bob"),
				],
				3,
				EnvironmentConfig {
					flip_token_address,
					eth_usdc_address,
					stake_manager_address,
					key_manager_address,
					eth_vault_address,
					ethereum_chain_id,
					cfe_settings: CfeSettings {
						eth_block_safety_margin,
						max_ceremony_stage_duration,
						eth_priority_fee_percentile: ETH_PRIORITY_FEE_PERCENTILE_DEFAULT,
					},
				},
				eth_init_agg_key,
				ethereum_deployment_block,
				genesis_stake_amount,
				min_stake,
				8 * HOURS,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Fork ID
		None,
		// Properties
		Some(chainflip_properties()),
		// Extensions
		None,
	))
}

macro_rules! network_spec {
	( $network:ident ) => {
		impl $network::Config {
			pub fn build_spec() -> Result<ChainSpec, String> {
				use $network::*;

				let wasm_binary =
					WASM_BINARY.ok_or_else(|| "Wasm binary not available".to_string())?;
				let StateChainEnvironment {
					flip_token_address,
					eth_usdc_address,
					stake_manager_address,
					key_manager_address,
					eth_vault_address,
					ethereum_chain_id,
					eth_init_agg_key,
					ethereum_deployment_block,
					genesis_stake_amount,
					eth_block_safety_margin,
					max_ceremony_stage_duration,
					min_stake,
				} = ENV;
				Ok(ChainSpec::from_genesis(
					NETWORK_NAME,
					NETWORK_NAME,
					ChainType::Live,
					move || {
						testnet_genesis(
							wasm_binary,
							// Initial PoA authorities
							vec![
								(
									BASHFUL_SR25519.into(),
									BASHFUL_SR25519.unchecked_into(),
									BASHFUL_ED25519.unchecked_into(),
								),
								(
									DOC_SR25519.into(),
									DOC_SR25519.unchecked_into(),
									DOC_ED25519.unchecked_into(),
								),
								(
									DOPEY_SR25519.into(),
									DOPEY_SR25519.unchecked_into(),
									DOPEY_ED25519.unchecked_into(),
								),
							],
							// Governance account - Snow White
							SNOW_WHITE_SR25519.into(),
							// Stakers at genesis
							vec![BASHFUL_SR25519.into(), DOC_SR25519.into(), DOPEY_SR25519.into()],
							2,
							EnvironmentConfig {
								flip_token_address,
								eth_usdc_address,
								stake_manager_address,
								key_manager_address,
								eth_vault_address,
								ethereum_chain_id,
								cfe_settings: CfeSettings {
									eth_block_safety_margin,
									max_ceremony_stage_duration,
									eth_priority_fee_percentile:
										ETH_PRIORITY_FEE_PERCENTILE_DEFAULT,
								},
							},
							eth_init_agg_key,
							ethereum_deployment_block,
							genesis_stake_amount,
							min_stake,
							3 * HOURS,
						)
					},
					// Bootnodes
					vec![],
					// Telemetry
					None,
					// Protocol ID
					None,
					// Fork ID
					None,
					// Properties
					Some(chainflip_properties()),
					// Extensions
					None,
				))
			}
		}
	};
}

network_spec!(perseverance);
network_spec!(sisyphos);

/// Configure initial storage state for FRAME modules.
/// 150 authority limit
#[allow(clippy::too_many_arguments)]
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
	root_key: AccountId,
	genesis_stakers: Vec<AccountId>,
	min_authorities: u32,
	config_set: EnvironmentConfig,
	eth_init_agg_key: [u8; 33],
	ethereum_deployment_block: u64,
	genesis_stake_amount: u128,
	minimum_stake: u128,
	blocks_per_epoch: BlockNumber,
) -> GenesisConfig {
	let authority_ids: Vec<AccountId> =
		initial_authorities.iter().map(|(id, ..)| id.clone()).collect();
	GenesisConfig {
		account_roles: AccountRolesConfig {
			initial_account_roles: authority_ids
				.clone()
				.into_iter()
				.map(|account_id| (account_id, AccountRole::Validator))
				.collect(),
		},
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		validator: ValidatorConfig {
			genesis_authorities: authority_ids,
			genesis_backups: genesis_stakers
				.iter()
				.cloned()
				.collect::<BTreeSet<_>>()
				.difference(
					&initial_authorities
						.iter()
						.map(|(account_id, _, _)| account_id.clone())
						.collect::<BTreeSet<_>>(),
				)
				.map(|account_id| (account_id.clone(), genesis_stake_amount))
				.collect(),
			blocks_per_epoch,
			claim_period_as_percentage: PERCENT_OF_EPOCH_PERIOD_CLAIMABLE,
			backup_reward_node_percentage: 20,
			bond: genesis_stake_amount,
			authority_set_min_size: min_authorities,
			min_size: min_authorities,
			max_size: MAX_AUTHORITIES,
			max_expansion: MAX_AUTHORITIES,
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.0.clone(), session_keys(x.1.clone(), x.2.clone())))
				.collect::<Vec<_>>(),
		},
		flip: FlipConfig { total_issuance: TOTAL_ISSUANCE },
		staking: StakingConfig {
			genesis_stakers: genesis_stakers
				.iter()
				.map(|acct| (acct.clone(), genesis_stake_amount))
				.collect::<Vec<(AccountId, FlipBalance)>>(),
			minimum_stake,
			claim_ttl: core::time::Duration::from_secs(3 * CLAIM_DELAY_SECS),
		},
		aura: AuraConfig { authorities: vec![] },
		grandpa: GrandpaConfig { authorities: vec![] },
		governance: GovernanceConfig { members: vec![root_key], expiry_span: 80000 },
		reputation: ReputationConfig {
			accrual_ratio: ACCRUAL_RATIO,
			penalties: PENALTIES.to_vec(),
			genesis_nodes: genesis_stakers,
		},
		environment: config_set,
		ethereum_vault: EthereumVaultConfig {
			vault_key: eth_init_agg_key.to_vec(),
			deployment_block: ethereum_deployment_block,
			keygen_response_timeout: KEYGEN_CEREMONY_TIMEOUT_BLOCKS,
		},
		ethereum_threshold_signer: EthereumThresholdSignerConfig {
			threshold_signature_response_timeout: THRESHOLD_SIGNATURE_CEREMONY_TIMEOUT_BLOCKS,
			_instance: PhantomData,
		},
		emissions: EmissionsConfig {
			current_authority_emission_inflation: CURRENT_AUTHORITY_EMISSION_INFLATION_PERBILL,
			backup_node_emission_inflation: BACKUP_NODE_EMISSION_INFLATION_PERBILL,
			supply_update_interval: SUPPLY_UPDATE_INTERVAL_DEFAULT,
		},
		transaction_payment: Default::default(),
	}
}

pub fn chainflip_properties() -> Properties {
	let mut properties = Properties::new();
	properties.insert(
		"ss58Format".into(),
		state_chain_runtime::constants::common::CHAINFLIP_SS58_PREFIX.into(),
	);
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("tokenSymbol".into(), "FLIP".into());
	properties.insert("color".into(), "#61CFAA".into());

	properties
}

/// Sets global that ensures SC AccountId's are printed correctly
pub fn use_chainflip_account_id_encoding() {
	set_default_ss58_version(Ss58AddressFormat::custom(CHAINFLIP_SS58_PREFIX));
}

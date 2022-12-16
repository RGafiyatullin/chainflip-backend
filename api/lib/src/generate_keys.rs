use anyhow::Result;
use rand_legacy::FromEntropy;
use sp_core::Pair;

pub use chainflip_engine::settings;
pub use chainflip_node::chain_spec::use_chainflip_account_id_encoding;

use zeroize::Zeroize;

#[derive(Debug, Zeroize)]
/// Public and Secret keys as bytes
pub struct KeyPair {
	pub secret_key: Vec<u8>,
	pub public_key: Vec<u8>,
}

/// Generate a new random node key.
/// This key is used for secure communication between Validators.
pub fn generate_node_key() -> KeyPair {
	use rand::SeedableRng;

	let mut rng = rand::rngs::StdRng::from_entropy();
	let keypair = ed25519_dalek::Keypair::generate(&mut rng);

	KeyPair {
		secret_key: keypair.secret.as_bytes().to_vec(),
		public_key: keypair.public.to_bytes().to_vec(),
	}
}

/// Generate a signing key (aka validator key) using the seed phrase.
/// If no seed phrase is provided, a new random seed phrase will be created.
/// Returns the key and the seed phrase used to create it.
/// This key is used to stake your node.
pub fn generate_signing_key(seed_phrase: Option<&str>) -> Result<(KeyPair, String)> {
	use bip39::{Language, Mnemonic, MnemonicType};

	// Get a new random seed phrase if one was not provided
	let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
	let seed_phrase = seed_phrase.unwrap_or_else(|| mnemonic.phrase());

	sp_core::Pair::from_phrase(seed_phrase, None)
		.map(|(pair, seed)| {
			let pair: sp_core::sr25519::Pair = pair;
			(
				KeyPair { secret_key: seed.to_vec(), public_key: pair.public().to_vec() },
				seed_phrase.to_string(),
			)
		})
		.map_err(|_| anyhow::Error::msg("Invalid seed phrase"))
}

/// Generate a new random ethereum key.
/// A chainflip validator must have their own Ethereum private keys and be capable of submitting
/// transactions. We recommend importing the generated secret key into metamask for account
/// management.
pub fn generate_ethereum_key() -> KeyPair {
	use secp256k1::Secp256k1;

	let mut rng = rand_legacy::rngs::StdRng::from_entropy();

	let (secret_key, public_key) = Secp256k1::new().generate_keypair(&mut rng);

	KeyPair { secret_key: secret_key[..].to_vec(), public_key: public_key.serialize().to_vec() }
}

#[test]
fn test_generate_signing_key_with_known_seed() {
	const SEED_PHRASE: &str =
		"essay awesome afraid movie wish save genius eyebrow tonight milk agree pretty alcohol three whale";

	let (generate_key, _) = generate_signing_key(Some(SEED_PHRASE)).unwrap();

	// Compare the generated secret key with a known secret key generated using the `chainflip-node
	// key generate` command
	assert_eq!(
		hex::encode(generate_key.secret_key),
		"afabf42a9a99910cdd64795ef05ed71acfa2238f5682d26ae62028df3cc59727"
	);
}

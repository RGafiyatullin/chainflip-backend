use anyhow::Result;
use bip39::{Language, Mnemonic};
use sha2::{Digest, Sha256};
use sp_core::Pair;
use zeroize::Zeroize;

pub use chainflip_engine::settings;
pub use chainflip_node::chain_spec::use_chainflip_account_id_encoding;

// We re-use the seed used to generate the validator key to generate the other 2 keys by hashing it
// with a context string.
const GENERATE_NODE_KEY_CONTEXT: &str = "chainflip_node_key";
const GENERATE_ETHEREUM_KEY_CONTEXT: &str = "chainflip_ethereum_key";

#[derive(Debug, Zeroize)]
/// Public and Secret keys as bytes
pub struct KeyPair {
	pub secret_key: Vec<u8>,
	pub public_key: Vec<u8>,
}

fn get_seed_from_phrase(seed_phrase: &str, context: &str) -> Result<[u8; 32]> {
	let mnemonic = Mnemonic::from_phrase(seed_phrase, Language::English)?;
	let seed = bip39::Seed::new(&mnemonic, "");

	let mut hasher = Sha256::new();

	hasher.update(seed.as_bytes());
	hasher.update(context);

	Ok(*hasher.finalize().as_ref())
}

/// Generate a new random node key.
/// This key is used for secure communication between Validators.
pub fn generate_node_key(seed_phrase: &str) -> Result<KeyPair> {
	use rand::SeedableRng;

	let mut rng = rand::rngs::StdRng::from_seed(get_seed_from_phrase(
		seed_phrase,
		GENERATE_NODE_KEY_CONTEXT,
	)?);
	let keypair = ed25519_dalek::Keypair::generate(&mut rng);

	Ok(KeyPair {
		secret_key: keypair.secret.as_bytes().to_vec(),
		public_key: keypair.public.as_bytes().to_vec(),
	})
}

/// Generate a signing key (aka validator key) using the seed phrase.
/// This key is used to stake your node.
pub fn generate_signing_key(seed_phrase: &str) -> Result<KeyPair> {
	sp_core::Pair::from_phrase(seed_phrase, None)
		.map(|(pair, seed)| {
			let pair: sp_core::sr25519::Pair = pair;

			KeyPair { secret_key: seed.to_vec(), public_key: pair.public().to_vec() }
		})
		.map_err(|e| anyhow::Error::msg(format!("Invalid seed phrase: {:?}", e)))
}

/// Generate a new random ethereum key.
/// A chainflip validator must have their own Ethereum private keys and be capable of submitting
/// transactions. We recommend importing the generated secret key into metamask for account
/// management.
pub fn generate_ethereum_key(seed_phrase: &str) -> Result<KeyPair> {
	use rand_legacy::SeedableRng;
	use secp256k1::Secp256k1;

	let mut rng = rand_legacy::rngs::StdRng::from_seed(get_seed_from_phrase(
		seed_phrase,
		GENERATE_ETHEREUM_KEY_CONTEXT,
	)?);

	let (secret_key, public_key) = Secp256k1::new().generate_keypair(&mut rng);

	Ok(KeyPair { secret_key: secret_key[..].to_vec(), public_key: public_key.serialize().to_vec() })
}

#[cfg(test)]
mod tests {
	use super::*;

	// Seed phrase obtained from use the `chainflip-node key generate` command.
	// Used in all tests to produce expected keys that are validated by 3rd party software.
	const TEST_SEED_PHRASE: &str =
    "essay awesome afraid movie wish save genius eyebrow tonight milk agree pretty alcohol three whale";

	#[test]
	fn test_generate_expected_signing_key_from_known_seed_phrase() {
		const EXPECTED_SECRET_KEY: &str =
			"afabf42a9a99910cdd64795ef05ed71acfa2238f5682d26ae62028df3cc59727";

		let generate_key = generate_signing_key(TEST_SEED_PHRASE).unwrap();

		// Compare the generated secret key with a known secret key generated using the
		// `chainflip-node key generate` command
		assert_eq!(hex::encode(generate_key.secret_key), EXPECTED_SECRET_KEY);
	}

	#[test]
	fn should_generate_expected_node_key_from_known_seed_phrase() {
		// This secret key has been proven to produce this public key at https://cyphr.me/ed25519_applet/ed.html
		const EXPECTED_SECRET_KEY: &str =
			"ae97945eabaf779e6015528cbda697ed66424a4903b0d5def328ebab76ddb64e";
		const EXPECTED_PUBLIC_KEY: &str =
			"f196a9f00b1590000ec21bd52f00335cc6a145480849dcd43ded83a11f3f9334";

		let keypair = generate_node_key(TEST_SEED_PHRASE).unwrap();

		// Check that a known seed makes the same secret key (ie our seed hashing has not changed)
		assert_eq!(hex::encode(keypair.secret_key), EXPECTED_SECRET_KEY);

		// Check that public key correct. This ensures that we are using the correct curve.
		assert_eq!(hex::encode(keypair.public_key), EXPECTED_PUBLIC_KEY)
	}

	#[test]
	fn should_generate_expected_ethereum_key_from_known_seed_phrase() {
		// This secret key has been proven on metamask to produce this eth address
		const EXPECTED_SECRET_KEY: &str =
			"8178fd770be229b7cc590ba09756cf91efa3e552baa9995c8857cb1df621b424";
		const EXPECTED_ETH_ADDRESS: &str = "618f9e0d1373bbbfb57c929341ddb1b9075a5048";

		let keypair = generate_ethereum_key(TEST_SEED_PHRASE).unwrap();

		// Check that a known seed makes the same secret key (ie our seed hashing has not changed)
		assert_eq!(hex::encode(keypair.secret_key), EXPECTED_SECRET_KEY);

		// Check that the derived eth address is correct. This ensures that we are using the correct
		// curve.
		assert_eq!(
			hex::encode(chainflip_engine::eth::utils::pubkey_to_eth_addr(
				secp256k1::PublicKey::from_slice(keypair.public_key.as_slice()).unwrap()
			)),
			EXPECTED_ETH_ADDRESS
		)
	}
}

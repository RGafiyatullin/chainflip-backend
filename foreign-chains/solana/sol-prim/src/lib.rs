#![cfg_attr(not(feature = "std-error"), no_std)]

#[macro_use]
mod macros;
mod utils;

pub mod consts;

define_binary!(address, Address, crate::consts::SOLANA_ADDRESS_LEN);
define_binary!(digest, Digest, crate::consts::SOLANA_DIGEST_LEN);
define_binary!(signature, Signature, crate::consts::SOLANA_SIGNATURE_LEN);

#[cfg(feature = "pda")]
pub mod pda;

#[cfg(test)]
mod tests;

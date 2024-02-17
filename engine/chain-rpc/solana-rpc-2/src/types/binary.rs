macro_rules! define_binary {
	($name: ident, $size: expr) => {
		#[derive(
			Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
		)]
		pub struct $name(#[serde(with = "crate::utils::serde_b58")] pub [u8; $size]);

		impl From<[u8; $size]> for $name {
			fn from(value: [u8; $size]) -> Self {
				Self(value)
			}
		}
		impl From<$name> for [u8; $size] {
			fn from(value: $name) -> [u8; $size] {
				value.0
			}
		}

		impl AsRef<[u8; $size]> for $name {
			fn as_ref(&self) -> &[u8; $size] {
				&self.0
			}
		}
		impl AsRef<[u8]> for $name {
			fn as_ref(&self) -> &[u8] {
				&self.0[..]
			}
		}

		impl std::str::FromStr for $name {
			type Err = String;

			fn from_str(s: &str) -> Result<Self, Self::Err> {
				bs58::decode(s)
					.into_vec()
					.map_err(|e| e.to_string())?
					.try_into()
					.map(Self)
					.map_err(|invalid: Vec<u8>| format!("invalid length: {}", invalid.len()))
			}
		}

		impl std::fmt::Display for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				let encoded = bs58::encode(&self.0).into_string();
				write!(f, "{}", encoded)
			}
		}
		impl std::fmt::Debug for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				let encoded = bs58::encode(&self.0).into_string();
				write!(f, "{}({})", std::any::type_name::<Self>(), encoded)
			}
		}
	};
}

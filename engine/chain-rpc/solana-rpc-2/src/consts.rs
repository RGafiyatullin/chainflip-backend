use std::time::Duration;

pub const ADDRESS_LEN: usize = 32;
pub const HASH_LEN: usize = 32;
pub const SIGNATURE_LEN: usize = 64;

pub const DEFAULT_RETRY_DELAYS: &[Duration] = &[
	Duration::from_millis(100),
	Duration::from_millis(200),
	Duration::from_millis(400),
	Duration::from_millis(800),
	Duration::from_millis(1200),
	Duration::from_millis(2400),
];

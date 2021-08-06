#![no_std]

pub use decode_event_derive::DecodeEvent;
use codec;

pub trait DecodeEvent where Self : Sized {
	fn try_decode<I : codec::Input>(variant : &str, data : &mut I) -> core::result::Result<core::option::Option<Self>, codec::Error>;
}

#![no_std]

#[allow(unused_imports)]
#[macro_use]
extern crate decode_event_derive;
pub use decode_event_derive::DecodeEvent;
use codec;

pub trait DecodeEvent where Self : Sized {
	fn decode_event<I : codec::Input>(variant : &str, data : &mut I) -> core::result::Result<core::option::Option<Self>, codec::Error>;
}

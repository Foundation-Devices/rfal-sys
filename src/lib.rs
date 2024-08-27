#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

mod platform;
pub use platform::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

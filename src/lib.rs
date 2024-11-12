#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod discover;
mod error;
mod ndef;
mod nfc;

pub use discover::*;
pub use error::*;
pub use ndef::*;
pub use nfc::*;
pub use rfal_sys::{
    ndefAttribInfoBlockT3T, ndefCapabilityContainer, ndefCapabilityContainerT1T,
    ndefCapabilityContainerT2T, ndefCapabilityContainerT4T, ndefCapabilityContainerT5T,
    ndefDeviceType, ndefInfo, ndefState, rfalBitRate, rfalComplianceMode, rfalLmNfcidLen,
    rfalNfcDevType, rfalNfcDiscoverParam, rfalNfcState, rfalNfcaListenDevice, Platform,
    RFAL_FWT_NONE, RFAL_NFC_LISTEN_TECH_A, RFAL_NFC_POLL_TECH_A, RFAL_NFC_TECH_NONE,
};

pub struct Rfal {
    pub discover: Discover,
    pub nfc: Nfc,
    pub ndef: Ndef,
}

impl Rfal {
    pub fn new(platform: Platform) -> Result<Self> {
        rfal_sys::rfal_platform_set(platform);
        result(unsafe { rfal_sys::rfalInitialize() })?;
        Nfc::initialize()?;
        Ok(Self {
            discover: Discover::default(),
            nfc: Nfc::default(),
            ndef: Ndef::default(),
        })
    }
}

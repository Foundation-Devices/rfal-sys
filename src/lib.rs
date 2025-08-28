// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

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
    ndefCapabilityContainer, ndefCapabilityContainerT1T, ndefCapabilityContainerT2T,
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

    pub fn reset(&mut self) {
        self.discover = Discover::default();
        self.nfc = Nfc::default();
        self.ndef = Ndef::default();
    }
}

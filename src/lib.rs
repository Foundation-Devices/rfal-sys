// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

use core::marker::PhantomData;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(test)]
extern crate std;

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

/// Owner of the RFAL native state.
///
/// RFAL keeps its state in C globals (the discovered device list, the active
/// device pointer, the transceive buffers and the received length), so there is
/// exactly one instance of it in a program. `Rfal` is the token that owns it:
///
/// * it can only be obtained from [`Rfal::new`], which registers the platform
///   callbacks and therefore fails on any later call,
/// * the operational handles are only reachable through it, and every operation
///   mutating the native state takes `&mut self`, so the borrow checker
///   serializes them,
/// * neither the handle nor the sub-handles are [`Send`] or [`Sync`], so none of
///   them can be moved to or shared with another thread or task.
///
/// # Execution model
///
/// RFAL must be driven from a single execution context. The [`Platform`]
/// callbacks are invoked by the C code from inside these operations, on that same
/// context, and must not re-enter any `Rfal` method: doing so would mutate the
/// native state underneath the operation in progress. An interrupt handler must
/// therefore only signal the polling context, never call into RFAL itself.
pub struct Rfal {
    pub discover: Discover,
    pub nfc: Nfc,
    pub ndef: Ndef,
    /// RFAL's C globals are a singleton driven from one execution context, so
    /// the handle owning them must not cross a thread or task boundary.
    _not_send_sync: PhantomData<*const ()>,
}

impl Rfal {
    /// Initializes RFAL and takes ownership of its native state.
    ///
    /// Returns [`Error::WrongState`] if it has already been called: the platform
    /// callbacks are registered once and for all, so a second handle over the
    /// same C globals cannot be created.
    pub fn new(platform: Platform) -> Result<Self> {
        rfal_sys::rfal_platform_try_set(platform).map_err(|_| Error::WrongState)?;
        result(unsafe { rfal_sys::rfalInitialize() })?;
        Nfc::initialize()?;
        Ok(Self {
            discover: Discover::new(),
            nfc: Nfc::new(),
            ndef: Ndef::new(),
            _not_send_sync: PhantomData,
        })
    }

    /// Drops the Rust-side state of the handles, without touching RFAL itself.
    pub fn reset(&mut self) {
        self.discover = Discover::new();
        self.nfc = Nfc::new();
        self.ndef = Ndef::new();
    }
}

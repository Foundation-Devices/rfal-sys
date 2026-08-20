// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(not(feature = "alloc"))]
use heapless::Vec;

use core::marker::PhantomData;

use crate::{result, rfalNfcDevType, rfalNfcState, rfalNfcaListenDevice, Error, Result};
use rfal_sys::rfalNfcDevice;

pub struct Device(pub(crate) rfalNfcDevice);

impl Device {
    pub fn nfca(&self) -> rfalNfcaListenDevice {
        unsafe { self.0.dev.nfca }
    }
    pub fn id(&self) -> Option<&[u8]> {
        if self.0.nfcidLen != 0 {
            Some(unsafe { core::slice::from_raw_parts(self.0.nfcid, self.0.nfcidLen as usize) })
        } else {
            None
        }
    }
    pub fn dev_type(&self) -> rfalNfcDevType {
        self.0.type_
    }
}

/// Handle over RFAL's NFC state machine.
///
/// Only reachable through [`Rfal`][crate::Rfal]. Every method driving the state
/// machine takes `&mut self`, because it mutates the same C globals as all the
/// other operations: the discovered device list, the active device pointer and
/// the transceive buffers.
pub struct Nfc {
    pub data_exchange: DataExchange,
    /// The handles must stay on the execution context driving RFAL: they are
    /// public fields of [`Rfal`][crate::Rfal] and could otherwise be moved out of
    /// it and sent to another thread or task.
    _not_send_sync: PhantomData<*const ()>,
}

impl Nfc {
    pub(crate) fn new() -> Self {
        Self {
            data_exchange: DataExchange::new(),
            _not_send_sync: PhantomData,
        }
    }
    pub(crate) fn initialize() -> Result<()> {
        result(unsafe { rfal_sys::rfalNfcInitialize() })
    }
    pub fn state(&self) -> rfalNfcState {
        unsafe { rfal_sys::rfalNfcGetState() }
    }
    pub fn worker(&mut self) {
        unsafe {
            rfal_sys::rfalNfcWorker();
        }
    }
    #[cfg(feature = "alloc")]
    pub fn get_devices_found(&mut self) -> Result<Vec<Device>> {
        let mut dev_list: *mut rfalNfcDevice = core::ptr::null_mut();
        let mut dev_cnt: u8 = 0;
        result(unsafe { rfal_sys::rfalNfcGetDevicesFound(&mut dev_list, &mut dev_cnt) })?;
        let devices = unsafe {
            core::slice::from_raw_parts(dev_list, dev_cnt as usize)
                .iter()
                .map(|&d| Device(d))
                .collect()
        };
        Ok(devices)
    }
    #[cfg(not(feature = "alloc"))]
    pub fn get_devices_found(&mut self) -> Result<Vec<Device, 4>> {
        let mut dev_list: *mut rfalNfcDevice = core::ptr::null_mut();
        let mut dev_cnt: u8 = 0;
        result(unsafe { rfal_sys::rfalNfcGetDevicesFound(&mut dev_list, &mut dev_cnt) })?;
        let devices = unsafe {
            core::slice::from_raw_parts(dev_list, dev_cnt as usize)
                .iter()
                .map(|&d| Device(d))
                .collect()
        };
        Ok(devices)
    }
    pub fn select(&mut self, dev_idx: u8) -> Result<()> {
        result(unsafe { rfal_sys::rfalNfcSelect(dev_idx) })
    }
    pub fn active_device(&mut self) -> Result<Device> {
        let mut dev: *mut rfalNfcDevice = core::ptr::null_mut();
        result(unsafe { rfal_sys::rfalNfcGetActiveDevice(&mut dev) })?;
        Ok(unsafe { Device(*dev) })
    }
    pub fn deactivate_and_idle(&mut self) -> Result<()> {
        result(unsafe {
            rfal_sys::rfalNfcDeactivate(rfal_sys::rfalNfcDeactivateType::RFAL_NFC_DEACTIVATE_IDLE)
        })
    }
    pub fn deactivate_and_sleep(&mut self) -> Result<()> {
        result(unsafe {
            rfal_sys::rfalNfcDeactivate(rfal_sys::rfalNfcDeactivateType::RFAL_NFC_DEACTIVATE_SLEEP)
        })
    }
    pub fn deactivate_and_discovery(&mut self) -> Result<()> {
        result(unsafe {
            rfal_sys::rfalNfcDeactivate(
                rfal_sys::rfalNfcDeactivateType::RFAL_NFC_DEACTIVATE_DISCOVERY,
            )
        })
    }
    pub fn enter_wakeup_mode(&mut self) -> Result<()> {
        result(unsafe { rfal_sys::rfalWakeUpModeStart(core::ptr::null()) })
    }
    pub fn exit_wakeup_mode(&mut self) -> Result<()> {
        result(unsafe { rfal_sys::rfalWakeUpModeStop() })
    }
}

/// Handle over RFAL's data exchange.
///
/// Only reachable through [`Nfc`], which is itself only reachable through
/// [`Rfal`][crate::Rfal]: the pointers it keeps refer to the singleton transceive
/// buffers, so a second handle would alias them.
pub struct DataExchange {
    rx_data_ptr: *mut u8,
    rcv_len_ptr: *mut u16,
    state: DataExchangeState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DataExchangeState {
    Idle,
    Pending,
    Complete,
    Failed(Error),
}

impl DataExchange {
    pub(crate) fn new() -> Self {
        Self {
            rx_data_ptr: core::ptr::null_mut(),
            rcv_len_ptr: core::ptr::null_mut(),
            state: DataExchangeState::Idle,
        }
    }

    pub fn start(&mut self, tx_data: Option<&mut [u8]>, fwt: u32) -> Result<()> {
        self.reset();

        let (tx_data, tx_data_len) = if let Some(tx_data) = tx_data {
            if tx_data.len() > u16::MAX as usize {
                self.state = DataExchangeState::Failed(Error::Param);
                return Err(Error::Param);
            }
            (tx_data.as_mut_ptr(), tx_data.len() as u16)
        } else {
            (core::ptr::null_mut(), 0)
        };

        let res = result(unsafe {
            rfal_sys::rfalNfcDataExchangeStart(
                tx_data,
                tx_data_len,
                &mut self.rx_data_ptr,
                &mut self.rcv_len_ptr,
                fwt,
            )
        });
        match res {
            Ok(()) => {
                if self.rx_data_ptr.is_null() || self.rcv_len_ptr.is_null() {
                    self.reset();
                    self.state = DataExchangeState::Failed(Error::Param);
                    Err(Error::Param)
                } else {
                    self.state = DataExchangeState::Pending;
                    Ok(())
                }
            }
            Err(err) => {
                self.reset();
                self.state = DataExchangeState::Failed(err);
                Err(err)
            }
        }
    }

    pub fn get_status(&mut self) -> Result<()> {
        match self.state {
            DataExchangeState::Idle => return Err(Error::NotInitialized),
            DataExchangeState::Complete => return Ok(()),
            DataExchangeState::Failed(err) => return Err(err),
            DataExchangeState::Pending => {}
        }

        let res = result(unsafe { rfal_sys::rfalNfcDataExchangeGetStatus() });
        match res {
            Ok(()) => {
                self.state = DataExchangeState::Complete;
                Ok(())
            }
            Err(Error::Busy) => {
                self.state = DataExchangeState::Pending;
                Err(Error::Busy)
            }
            Err(err) => {
                self.clear_pointers();
                self.state = DataExchangeState::Failed(err);
                Err(err)
            }
        }
    }

    /// Returns the last completed receive buffer.
    ///
    /// The slice points to RFAL-owned storage and is valid only until the next
    /// mutable data-exchange operation or lower-level RFAL operation that may
    /// reuse the receive buffer.
    pub fn rx_data(&mut self) -> Result<&[u8]> {
        match self.state {
            DataExchangeState::Complete => {
                if self.rx_data_ptr.is_null() || self.rcv_len_ptr.is_null() {
                    return Err(Error::NotInitialized);
                }

                let len = unsafe { *self.rcv_len_ptr } as usize;
                if len > core::mem::size_of::<rfal_sys::rfalNfcBuffer>() {
                    return Err(Error::NoMem);
                }

                Ok(unsafe { core::slice::from_raw_parts(self.rx_data_ptr, len) })
            }
            DataExchangeState::Pending => Err(Error::Busy),
            DataExchangeState::Failed(err) => Err(err),
            DataExchangeState::Idle => Err(Error::NotInitialized),
        }
    }

    pub fn rx_data_into(&mut self, buf: &mut [u8]) -> Result<usize> {
        let rx_data = self.rx_data()?;
        if rx_data.len() > buf.len() {
            return Err(Error::NoMem);
        }

        buf[..rx_data.len()].copy_from_slice(rx_data);
        Ok(rx_data.len())
    }

    fn reset(&mut self) {
        self.clear_pointers();
        self.state = DataExchangeState::Idle;
    }

    fn clear_pointers(&mut self) {
        self.rx_data_ptr = core::ptr::null_mut();
        self.rcv_len_ptr = core::ptr::null_mut();
    }
}

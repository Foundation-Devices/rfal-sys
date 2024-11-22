// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(not(feature = "alloc"))]
use heapless::Vec;

use crate::{result, rfalNfcDevType, rfalNfcState, rfalNfcaListenDevice, Result};
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

#[derive(Default)]
pub struct Nfc {
    pub data_exchange: DataExchange,
}

impl Nfc {
    pub fn initialize() -> Result<()> {
        result(unsafe { rfal_sys::rfalNfcInitialize() })
    }
    pub fn state(&self) -> rfalNfcState {
        unsafe { rfal_sys::rfalNfcGetState() }
    }
    pub fn worker(&self) {
        unsafe {
            rfal_sys::rfalNfcWorker();
        }
    }
    #[cfg(feature = "alloc")]
    pub fn get_devices_found(&self) -> Result<Vec<Device>> {
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
    pub fn get_devices_found(&self) -> Result<Vec<Device, 4>> {
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
    pub fn select(&self, dev_idx: u8) -> Result<()> {
        result(unsafe { rfal_sys::rfalNfcSelect(dev_idx) })
    }
    pub fn active_device(&self) -> Result<Device> {
        let mut dev: *mut rfalNfcDevice = core::ptr::null_mut();
        result(unsafe { rfal_sys::rfalNfcGetActiveDevice(&mut dev) })?;
        Ok(unsafe { Device(*dev) })
    }
    pub fn deactivate_and_idle(&self) -> Result<()> {
        result(unsafe {
            rfal_sys::rfalNfcDeactivate(rfal_sys::rfalNfcDeactivateType::RFAL_NFC_DEACTIVATE_IDLE)
        })
    }
    pub fn deactivate_and_sleep(&self) -> Result<()> {
        result(unsafe {
            rfal_sys::rfalNfcDeactivate(rfal_sys::rfalNfcDeactivateType::RFAL_NFC_DEACTIVATE_SLEEP)
        })
    }
    pub fn deactivate_and_discovery(&self) -> Result<()> {
        result(unsafe {
            rfal_sys::rfalNfcDeactivate(
                rfal_sys::rfalNfcDeactivateType::RFAL_NFC_DEACTIVATE_DISCOVERY,
            )
        })
    }
}

#[derive(Default)]
pub struct DataExchange {}

impl DataExchange {
    pub fn start(
        &self,
        tx_data: Option<&mut [u8]>,
        rx_data: &mut [u8],
        rcv_len: &mut u16,
        fwt: u32,
    ) -> Result<()> {
        let (tx_data, tx_data_len) = if let Some(tx_data) = tx_data {
            (tx_data.as_mut_ptr(), tx_data.len() as u16)
        } else {
            (core::ptr::null_mut(), 0)
        };
        let mut rx_data_ptr = rx_data.as_mut_ptr();
        let mut rcv_len_ptr = rcv_len as *mut u16;
        result(unsafe {
            rfal_sys::rfalNfcDataExchangeStart(
                tx_data,
                tx_data_len,
                &mut rx_data_ptr,
                &mut rcv_len_ptr,
                fwt,
            )
        })
    }
    pub fn get_status(&self) -> Result<()> {
        result(unsafe { rfal_sys::rfalNfcDataExchangeGetStatus() })
    }
}

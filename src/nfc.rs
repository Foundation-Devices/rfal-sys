// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(not(feature = "alloc"))]
use heapless::Vec;

use core::marker::PhantomData;

use crate::{
    result, rfalNfcDevType, rfalNfcRfInterface, rfalNfcState, rfalNfcaListenDevice, Error, Result,
};
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
    /// RF interface activated for this device.
    ///
    /// It selects the unit RFAL uses for the data exchange lengths, see
    /// [`DataExchange`].
    pub fn rf_interface(&self) -> rfalNfcRfInterface {
        self.0.rfInterface
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
///
/// # Length units
///
/// RFAL expresses the data exchange lengths in a unit that depends on the RF
/// interface activated for the device:
///
/// * [`RFAL_NFC_INTERFACE_RF`][rfalNfcRfInterface::RFAL_NFC_INTERFACE_RF]: **bits**,
///   so a frame can end on a partial byte,
/// * [`RFAL_NFC_INTERFACE_ISODEP`][rfalNfcRfInterface::RFAL_NFC_INTERFACE_ISODEP]
///   and
///   [`RFAL_NFC_INTERFACE_NFCDEP`][rfalNfcRfInterface::RFAL_NFC_INTERFACE_NFCDEP]:
///   **bytes**.
///
/// This type reads the active interface when the exchange starts and does the
/// conversion, so its own API is in bytes on every interface: [`start`][Self::start]
/// transmits whole bytes, and [`rx_data`][Self::rx_data] returns whole bytes.
/// [`start_bits`][Self::start_bits] and [`rx_bits`][Self::rx_bits] expose the bit
/// length for raw RF frames that are not a whole number of bytes.
pub struct DataExchange {
    rx_data_ptr: *mut u8,
    rcv_len_ptr: *mut u16,
    state: DataExchangeState,
    interface: Option<rfalNfcRfInterface>,
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
            interface: None,
        }
    }

    /// Starts a data exchange transmitting whole bytes.
    ///
    /// The length is converted to the unit the active interface expects, so the
    /// same call works on every interface. Use [`start_bits`][Self::start_bits]
    /// to send a raw RF frame ending on a partial byte.
    pub fn start(&mut self, tx_data: Option<&mut [u8]>, fwt: u32) -> Result<()> {
        let interface = self.begin()?;
        let tx_len = match encode_tx_len(interface, tx_data.as_ref().map_or(0, |d| d.len())) {
            Ok(len) => len,
            Err(err) => {
                self.state = DataExchangeState::Failed(err);
                return Err(err);
            }
        };

        self.start_raw(tx_data, tx_len, interface, fwt)
    }

    /// Starts a raw RF data exchange transmitting `tx_bits` bits.
    ///
    /// Only the raw RF interface counts in bits; on ISO-DEP and NFC-DEP this
    /// returns [`Error::NotSupp`] and [`start`][Self::start] must be used instead.
    /// `tx_data` must hold at least `tx_bits` bits.
    pub fn start_bits(&mut self, tx_data: Option<&mut [u8]>, tx_bits: u16, fwt: u32) -> Result<()> {
        let interface = self.begin()?;

        let err = if !uses_bit_lengths(interface) {
            Some(Error::NotSupp)
        } else if bits_to_bytes(tx_bits) > tx_data.as_ref().map_or(0, |d| d.len()) {
            Some(Error::Param)
        } else {
            None
        };
        if let Some(err) = err {
            self.state = DataExchangeState::Failed(err);
            return Err(err);
        }

        self.start_raw(tx_data, tx_bits, interface, fwt)
    }

    /// Resets the exchange state and reads the interface of the active device.
    fn begin(&mut self) -> Result<rfalNfcRfInterface> {
        self.reset();
        match active_interface() {
            Ok(interface) => Ok(interface),
            Err(err) => {
                self.state = DataExchangeState::Failed(err);
                Err(err)
            }
        }
    }

    fn start_raw(
        &mut self,
        tx_data: Option<&mut [u8]>,
        tx_len: u16,
        interface: rfalNfcRfInterface,
        fwt: u32,
    ) -> Result<()> {
        let tx_data = match tx_data {
            Some(tx_data) => tx_data.as_mut_ptr(),
            None => core::ptr::null_mut(),
        };

        let res = result(unsafe {
            rfal_sys::rfalNfcDataExchangeStart(
                tx_data,
                tx_len,
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
                    self.interface = Some(interface);
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

    /// Returns the last completed receive buffer, as whole bytes.
    ///
    /// On the raw RF interface RFAL reports a bit count, so a frame ending on a
    /// partial byte is rounded up to the byte holding its last bits; use
    /// [`rx_bits`][Self::rx_bits] to recover the exact length.
    ///
    /// The slice points to RFAL-owned storage and is valid only until the next
    /// mutable data-exchange operation or lower-level RFAL operation that may
    /// reuse the receive buffer.
    pub fn rx_data(&mut self) -> Result<&[u8]> {
        let (_, len) = self.rx_len()?;

        Ok(unsafe { core::slice::from_raw_parts(self.rx_data_ptr, len) })
    }

    /// Returns the length of the last completed receive, in bits.
    ///
    /// Only the raw RF interface can report a length that is not a whole number
    /// of bytes; on ISO-DEP and NFC-DEP this is the byte count times eight.
    pub fn rx_bits(&mut self) -> Result<u16> {
        let (bits, _) = self.rx_len()?;

        Ok(bits)
    }

    /// Length of the last completed receive, as (bits, bytes).
    ///
    /// Bounds the length against the buffer RFAL actually filled for the active
    /// interface: the raw RF buffer is far smaller than the ISO-DEP one they share,
    /// so checking against the union would let a bogus length expose uninitialized
    /// bytes past the end of the RF buffer.
    fn rx_len(&self) -> Result<(u16, usize)> {
        match self.state {
            DataExchangeState::Complete => {
                if self.rx_data_ptr.is_null() || self.rcv_len_ptr.is_null() {
                    return Err(Error::NotInitialized);
                }

                let interface = self.interface.ok_or(Error::NotInitialized)?;
                let raw_len = unsafe { *self.rcv_len_ptr };

                decode_rx_len(interface, raw_len)
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
        self.interface = None;
    }

    fn clear_pointers(&mut self) {
        self.rx_data_ptr = core::ptr::null_mut();
        self.rcv_len_ptr = core::ptr::null_mut();
    }
}

/// Reads the RF interface activated for the current device.
fn active_interface() -> Result<rfalNfcRfInterface> {
    let mut dev: *mut rfalNfcDevice = core::ptr::null_mut();
    result(unsafe { rfal_sys::rfalNfcGetActiveDevice(&mut dev) })?;
    if dev.is_null() {
        return Err(Error::WrongState);
    }

    Ok(unsafe { (*dev).rfInterface })
}

/// Whether RFAL counts the data exchange lengths of `interface` in bits.
///
/// See `rfalNfcDataExchangeStart` in rfal_nfc.h: the raw RF interface passes the
/// lengths straight to the transceive layer, which counts bits, while ISO-DEP and
/// NFC-DEP copy whole bytes.
fn uses_bit_lengths(interface: rfalNfcRfInterface) -> bool {
    matches!(interface, rfalNfcRfInterface::RFAL_NFC_INTERFACE_RF)
}

/// Number of bytes RFAL can place in the receive buffer of `interface`.
///
/// The three buffers are a union, so their sizes differ widely: bounding a length
/// against the union instead of the active member would allow a slice reaching
/// past the end of the smaller ones.
fn rx_capacity(interface: rfalNfcRfInterface) -> usize {
    match interface {
        rfalNfcRfInterface::RFAL_NFC_INTERFACE_RF => rfal_sys::RFAL_FEATURE_NFC_RF_BUF_LEN as usize,
        rfalNfcRfInterface::RFAL_NFC_INTERFACE_ISODEP => {
            core::mem::size_of::<rfal_sys::rfalIsoDepApduBufFormat>()
                - core::mem::offset_of!(rfal_sys::rfalIsoDepApduBufFormat, apdu)
        }
        rfalNfcRfInterface::RFAL_NFC_INTERFACE_NFCDEP => {
            core::mem::size_of::<rfal_sys::rfalNfcDepPduBufFormat>()
                - core::mem::offset_of!(rfal_sys::rfalNfcDepPduBufFormat, pdu)
        }
    }
}

/// Number of whole bytes spanned by `bits`.
fn bits_to_bytes(bits: u16) -> usize {
    (bits as usize).div_ceil(8)
}

/// Converts a length in bytes into the unit `interface` expects.
fn encode_tx_len(interface: rfalNfcRfInterface, len: usize) -> Result<u16> {
    let len = if uses_bit_lengths(interface) {
        len.checked_mul(8).ok_or(Error::Param)?
    } else {
        len
    };

    u16::try_from(len).map_err(|_| Error::Param)
}

/// Converts the length RFAL reports for `interface` into (bits, whole bytes).
fn decode_rx_len(interface: rfalNfcRfInterface, raw_len: u16) -> Result<(u16, usize)> {
    let bytes = if uses_bit_lengths(interface) {
        bits_to_bytes(raw_len)
    } else {
        raw_len as usize
    };

    if bytes > rx_capacity(interface) {
        return Err(Error::NoMem);
    }

    // bytes is bounded by the capacity above, so the bit count cannot overflow.
    let bits = if uses_bit_lengths(interface) {
        raw_len
    } else {
        (bytes * 8) as u16
    };

    Ok((bits, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RF: rfalNfcRfInterface = rfalNfcRfInterface::RFAL_NFC_INTERFACE_RF;
    const ISODEP: rfalNfcRfInterface = rfalNfcRfInterface::RFAL_NFC_INTERFACE_ISODEP;
    const NFCDEP: rfalNfcRfInterface = rfalNfcRfInterface::RFAL_NFC_INTERFACE_NFCDEP;

    #[test]
    fn raw_rf_transmits_bits() {
        assert_eq!(encode_tx_len(RF, 0), Ok(0));
        assert_eq!(encode_tx_len(RF, 1), Ok(8));
        assert_eq!(encode_tx_len(RF, 4), Ok(32));
    }

    #[test]
    fn protocol_interfaces_transmit_bytes() {
        assert_eq!(encode_tx_len(ISODEP, 4), Ok(4));
        assert_eq!(encode_tx_len(NFCDEP, 4), Ok(4));
    }

    #[test]
    fn tx_len_rejects_lengths_beyond_the_unit() {
        assert_eq!(encode_tx_len(RF, 8192), Err(Error::Param));
        assert_eq!(
            encode_tx_len(ISODEP, u16::MAX as usize + 1),
            Err(Error::Param)
        );
    }

    #[test]
    fn raw_rf_receives_bits() {
        assert_eq!(decode_rx_len(RF, 32), Ok((32, 4)));
        // a frame ending on a partial byte keeps its bit count
        assert_eq!(decode_rx_len(RF, 28), Ok((28, 4)));
        assert_eq!(decode_rx_len(RF, 1), Ok((1, 1)));
        assert_eq!(decode_rx_len(RF, 0), Ok((0, 0)));
    }

    #[test]
    fn protocol_interfaces_receive_bytes() {
        assert_eq!(decode_rx_len(ISODEP, 32), Ok((256, 32)));
        assert_eq!(decode_rx_len(NFCDEP, 0), Ok((0, 0)));
    }

    #[test]
    fn rx_len_is_bounded_by_the_active_buffer() {
        let rf_capacity = rx_capacity(RF);
        let isodep_capacity = rx_capacity(ISODEP);
        assert!(rf_capacity < isodep_capacity);

        // the largest frame each interface can actually hold is accepted
        assert!(decode_rx_len(RF, (rf_capacity * 8) as u16).is_ok());
        assert!(decode_rx_len(ISODEP, isodep_capacity as u16).is_ok());

        // one byte past the buffer RFAL filled is not
        assert_eq!(
            decode_rx_len(RF, (rf_capacity * 8) as u16 + 1),
            Err(Error::NoMem)
        );
        assert_eq!(
            decode_rx_len(ISODEP, isodep_capacity as u16 + 1),
            Err(Error::NoMem)
        );
        // a byte count that would fit the ISO-DEP buffer must not be accepted on RF
        assert_eq!(decode_rx_len(RF, u16::MAX), Err(Error::NoMem));
    }
}

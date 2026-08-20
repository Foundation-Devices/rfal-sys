// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(not(feature = "alloc"))]
use heapless::Vec;

use core::marker::PhantomData;

use crate::{
    ndefCapabilityContainer, ndefDeviceType, ndefInfo, ndefState, nfc::Device, result, Error,
    Result,
};

pub const RAW_MESSAGE_BUF_LEN: usize = 256;

/// Handle over the NDEF pollers.
///
/// Only reachable through [`Rfal`][crate::Rfal]: the poller operations run RF
/// transactions through the same RFAL globals as [`Nfc`][crate::Nfc].
pub struct Ndef {
    pub poller: Poller,
    /// The handles must stay on the execution context driving RFAL: they are
    /// public fields of [`Rfal`][crate::Rfal] and could otherwise be moved out of
    /// it and sent to another thread or task.
    _not_send_sync: PhantomData<*const ()>,
}

impl Ndef {
    pub(crate) fn new() -> Self {
        Self {
            poller: Poller::new(),
            _not_send_sync: PhantomData,
        }
    }
}

pub struct Poller {
    ctx: Option<rfal_sys::ndefContext>,
}

impl Poller {
    pub(crate) fn new() -> Self {
        Self { ctx: None }
    }

    pub fn initialize(&mut self, nfc_dev: &Device) -> Result<()> {
        // allocate default values manually, thanks bingen to not deriving Default trait...
        let mut ndef_ctx = rfal_sys::ndefContext {
            type_: ndefDeviceType::NDEF_DEV_NONE,
            device: nfc_dev.0,
            state: ndefState::NDEF_STATE_INVALID,
            cc: rfal_sys::ndefCapabilityContainer {
                t1t: rfal_sys::ndefCapabilityContainerT1T {
                    magicNumber: 0,
                    majorVersion: 0,
                    minorVersion: 0,
                    tagMemorySize: 0,
                    readAccess: 0,
                    writeAccess: 0,
                },
            },
            messageLen: 0,
            messageOffset: 0,
            areaLen: 0,
            ccBuf: [0; 17usize],
            ndefPollWrapper: core::ptr::null(),
            subCtx: rfal_sys::ndefContext__bindgen_ty_1 {
                t1t: rfal_sys::ndefT1TContext {
                    rfu: core::ptr::null_mut(),
                },
            },
        };
        result(unsafe { rfal_sys::ndefPollerContextInitialization(&mut ndef_ctx, &nfc_dev.0) })?;
        self.ctx.replace(ndef_ctx);
        Ok(())
    }
    pub fn ndef_detect(&mut self) -> Result<ndefInfo> {
        match self.ctx {
            Some(mut ctx) => {
                // allocate default values manually, thanks bingen to not deriving Default trait...
                let mut ndef_info = ndefInfo {
                    majorVersion: 0,
                    minorVersion: 0,
                    areaLen: 0,
                    areaAvalableSpaceLen: 0,
                    messageLen: 0,
                    state: ndefState::NDEF_STATE_INVALID,
                };
                result(unsafe { rfal_sys::ndefPollerNdefDetect(&mut ctx, &mut ndef_info) })?;
                self.ctx.replace(ctx);
                Ok(ndef_info)
            }
            None => Err(Error::NotInitialized),
        }
    }
    pub fn ndef_ctx_type(&self) -> Option<ndefDeviceType> {
        self.ctx.as_ref().map(|ctx| ctx.type_)
    }
    pub fn ndef_ctx_state(&self) -> Option<ndefState> {
        self.ctx.as_ref().map(|ctx| ctx.state)
    }

    #[cfg(feature = "alloc")]
    pub fn read_raw_message(&mut self) -> Result<Vec<u8>> {
        let message_len = self.raw_message_len()?;
        let mut raw_message = Vec::new();
        raw_message
            .try_reserve_exact(message_len)
            .map_err(|_| Error::NoMem)?;
        raw_message.resize(message_len, 0);

        let received_len = self.read_raw_message_into(&mut raw_message)?;
        raw_message.truncate(received_len);
        Ok(raw_message)
    }

    #[cfg(not(feature = "alloc"))]
    pub fn read_raw_message(&mut self) -> Result<Vec<u8, RAW_MESSAGE_BUF_LEN>> {
        let mut raw_message_buf = [0u8; RAW_MESSAGE_BUF_LEN];
        let received_len = self.read_raw_message_into(&mut raw_message_buf)?;

        let mut raw_message = Vec::new();
        raw_message
            .extend_from_slice(&raw_message_buf[..received_len])
            .map_err(|_| Error::NoMem)?;
        Ok(raw_message)
    }

    pub fn read_raw_message_into(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.ctx {
            Some(mut ctx) => {
                if buf.len() > u32::MAX as usize {
                    return Err(Error::Param);
                }

                let mut received_len = 0u32;
                let res = unsafe {
                    rfal_sys::ndefPollerReadRawMessage(
                        &mut ctx,
                        buf.as_mut_ptr() as *mut _,
                        buf.len() as u32,
                        &mut received_len,
                        true,
                    )
                };
                self.ctx.replace(ctx);
                result(res)?;
                checked_received_len(received_len, buf.len())
            }
            None => Err(Error::NotInitialized),
        }
    }

    #[cfg(feature = "alloc")]
    fn raw_message_len(&self) -> Result<usize> {
        self.ctx
            .as_ref()
            .map(|ctx| ctx.messageLen as usize)
            .ok_or(Error::NotInitialized)
    }

    pub fn write_raw_message(&mut self, msg: &[u8]) -> Result<()> {
        match self.ctx {
            Some(mut ctx) => {
                result(unsafe {
                    rfal_sys::ndefPollerWriteRawMessage(&mut ctx, msg.as_ptr(), msg.len() as u32)
                })?;
                self.ctx.replace(ctx);
                Ok(())
            }
            None => Err(Error::NotInitialized),
        }
    }
    pub fn tag_format(&mut self, cc: ndefCapabilityContainer, option: u32) -> Result<()> {
        match self.ctx {
            Some(mut ctx) => {
                result(unsafe { rfal_sys::ndefPollerTagFormat(&mut ctx, &cc, option) })?;
                self.ctx.replace(ctx);
                Ok(())
            }
            None => Err(Error::NotInitialized),
        }
    }
}

fn checked_received_len(received_len: u32, buffer_len: usize) -> Result<usize> {
    let received_len = received_len as usize;
    if received_len > buffer_len {
        return Err(Error::NoMem);
    }
    Ok(received_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_received_len_accepts_lengths_within_buffer() {
        assert_eq!(checked_received_len(3, 4), Ok(3));
        assert_eq!(checked_received_len(4, 4), Ok(4));
    }

    #[test]
    fn checked_received_len_rejects_lengths_beyond_buffer() {
        assert_eq!(checked_received_len(5, 4), Err(Error::NoMem));
    }
}

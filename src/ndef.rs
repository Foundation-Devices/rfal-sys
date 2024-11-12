use crate::{
    ndefCapabilityContainer, ndefDeviceType, ndefInfo, ndefState, nfc::Device, result, Error,
    Result,
};

#[derive(Default)]
pub struct Ndef {
    pub poller: Poller,
}

#[derive(Default)]
pub struct Poller {
    ctx: Option<rfal_sys::ndefContext>,
}

impl Poller {
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
    pub fn read_raw_message(&mut self) -> Result<&[u8]> {
        match self.ctx {
            Some(mut ctx) => {
                let mut raw_message_buf = [0u8; 256];
                let mut received_len = 0u32;
                result(unsafe {
                    rfal_sys::ndefPollerReadRawMessage(
                        &mut ctx,
                        raw_message_buf.as_mut_ptr() as *mut _,
                        raw_message_buf.len() as u32,
                        &mut received_len,
                        true,
                    )
                })?;
                self.ctx.replace(ctx);
                Ok(unsafe {
                    core::slice::from_raw_parts(raw_message_buf.as_ptr(), received_len as usize)
                })
            }
            None => Err(Error::NotInitialized),
        }
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

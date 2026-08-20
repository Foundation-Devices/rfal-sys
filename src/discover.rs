// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use core::marker::PhantomData;

use crate::{result, rfalNfcDiscoverParam, Result};

/// Discovery configuration.
///
/// Only reachable through [`Rfal`][crate::Rfal]: starting a discovery mutates
/// RFAL's C globals, so it requires exclusive access to the owning handle.
pub struct Discover {
    pub params: rfalNfcDiscoverParam,
    /// The handles must stay on the execution context driving RFAL: they are
    /// public fields of [`Rfal`][crate::Rfal] and could otherwise be moved out of
    /// it and sent to another thread or task.
    _not_send_sync: PhantomData<*const ()>,
}

impl Discover {
    pub(crate) fn new() -> Self {
        // allocate default values manually, thanks bingen to not deriving Default trait...
        let params = rfalNfcDiscoverParam {
            compMode: rfal_sys::rfalComplianceMode::RFAL_COMPLIANCE_MODE_NFC,
            devLimit: 1,
            nfcfBR: rfal_sys::rfalBitRate::RFAL_BR_212,
            ap2pBR: rfal_sys::rfalBitRate::RFAL_BR_424,
            maxBR: rfal_sys::rfalBitRate::RFAL_BR_KEEP,
            isoDepFS: rfal_sys::rfalIsoDepFSxI::RFAL_ISODEP_FSXI_256,
            nfcDepLR: rfal_sys::RFAL_NFCDEP_LR_254 as u8,
            GBLen: 0,
            p2pNfcaPrio: false,
            wakeupEnabled: false,
            wakeupConfigDefault: true,
            wakeupPollBefore: false,
            wakeupNPolls: 1,
            totalDuration: 1000,
            techs2Find: rfal_sys::RFAL_NFC_TECH_NONE as u16,
            techs2Bail: rfal_sys::RFAL_NFC_TECH_NONE as u16,
            nfcid3: [0; 10],
            GB: [0; 48],
            propNfc: rfal_sys::rfalNfcPropCallbacks {
                rfalNfcpPollerInitialize: None,
                rfalNfcpPollerTechnologyDetection: None,
                rfalNfcpPollerStartCollisionResolution: None,
                rfalNfcpPollerGetCollisionResolutionStatus: None,
                rfalNfcpStartActivation: None,
                rfalNfcpGetActivationStatus: None,
            },
            lmConfigPA: rfal_sys::rfalLmConfPA {
                nfcidLen: rfal_sys::rfalLmNfcidLen::RFAL_LM_NFCID_LEN_04,
                nfcid: [0; 10],
                SENS_RES: [0; 2],
                SEL_RES: 0,
            },
            lmConfigPF: rfal_sys::rfalLmConfPF {
                SC: [0; 2],
                SENSF_RES: [0; 19],
            },
            notifyCb: None,
            wakeupConfig: rfal_sys::rfalWakeUpConfig {
                period: rfal_sys::rfalWumPeriod::RFAL_WUM_PERIOD_300MS,
                indAmp: rfal_sys::rfalWakeUpConfig__bindgen_ty_1 {
                    enabled: false,
                    delta: 0,
                    reference: 0,
                },
            },
        };
        Self {
            params,
            _not_send_sync: PhantomData,
        }
    }

    /// Starts the discovery loop with the current [`params`][Self::params].
    ///
    /// RFAL copies the parameters, so later changes only apply to the next call.
    pub fn start(&mut self) -> Result<()> {
        result(unsafe { rfal_sys::rfalNfcDiscover(&self.params as *const _) })
    }
}

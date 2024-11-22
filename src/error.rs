// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, PartialEq)]
pub enum Error {
    NotInitialized,

    NoMem,
    Busy,
    Io,
    Timeout,
    Request,
    NoMsg,
    Param,
    System,
    Framing,
    Overrun,
    Proto,
    Internal,
    Again,
    MemCorrupt,
    NotImplemented,
    PcCorrupt,
    Send,
    Ignore,
    Semantic,
    Syntax,
    Crc,
    NotFound,
    NotUnique,
    NotSupp,
    Write,
    Fifo,
    Par,
    Done,
    RfCollision,
    HwOverrun,
    ReleaseReq,
    SleepReq,
    WrongState,
    MaxReruns,
    Disabled,
    HwMismatch,
    LinkLoss,
    InvalidHandle,
    IncompleteByte,
    Unknown(u16),
}

const RFAL_ERR_NONE: u16 = 0; // no error occurred
const RFAL_ERR_NOMEM: u16 = 1; // not enough memory to perform the requested operation
const RFAL_ERR_BUSY: u16 = 2; // device or resource busy
const RFAL_ERR_IO: u16 = 3; // generic IO error
const RFAL_ERR_TIMEOUT: u16 = 4; // error due to timeout
const RFAL_ERR_REQUEST: u16 = 5; // invalid request or requested function can't be executed at the moment
const RFAL_ERR_NOMSG: u16 = 6; // No message of desired type
const RFAL_ERR_PARAM: u16 = 7; // Parameter error
const RFAL_ERR_SYSTEM: u16 = 8; // System error
const RFAL_ERR_FRAMING: u16 = 9; // Framing error
const RFAL_ERR_OVERRUN: u16 = 10; // lost one or more received bytes
const RFAL_ERR_PROTO: u16 = 11; // protocol error
const RFAL_ERR_INTERNAL: u16 = 12; // Internal Error
const RFAL_ERR_AGAIN: u16 = 13; // Call again
const RFAL_ERR_MEM_CORRUPT: u16 = 14; // memory corruption
const RFAL_ERR_NOT_IMPLEMENTED: u16 = 15; // not implemented
const RFAL_ERR_PC_CORRUPT: u16 = 16; // Program Counter has been manipulated or spike/noise trigger illegal operation
const RFAL_ERR_SEND: u16 = 17; // error sending
const RFAL_ERR_IGNORE: u16 = 18; // indicates error detected but to be ignored
const RFAL_ERR_SEMANTIC: u16 = 19; // indicates error in state machine (unexpected cmd)
const RFAL_ERR_SYNTAX: u16 = 20; // indicates error in state machine (unknown cmd)
const RFAL_ERR_CRC: u16 = 21; // crc error
const RFAL_ERR_NOTFOUND: u16 = 22; // transponder not found
const RFAL_ERR_NOTUNIQUE: u16 = 23; // transponder not unique - more than one transponder in field
const RFAL_ERR_NOTSUPP: u16 = 24; // requested operation not supported
const RFAL_ERR_WRITE: u16 = 25; // write error
const RFAL_ERR_FIFO: u16 = 26; // fifo over or underflow error
const RFAL_ERR_PAR: u16 = 27; // parity error
const RFAL_ERR_DONE: u16 = 28; // transfer has already finished
const RFAL_ERR_RF_COLLISION: u16 = 29; // collision error (Bit Collision or during RF Collision avoidance )
const RFAL_ERR_HW_OVERRUN: u16 = 30; // lost one or more received bytes
const RFAL_ERR_RELEASE_REQ: u16 = 31; // device requested release
const RFAL_ERR_SLEEP_REQ: u16 = 32; // device requested sleep
const RFAL_ERR_WRONG_STATE: u16 = 33; // incorrent state for requested operation
const RFAL_ERR_MAX_RERUNS: u16 = 34; // blocking procedure reached maximum runs
const RFAL_ERR_DISABLED: u16 = 35; // operation aborted due to disabled configuration
const RFAL_ERR_HW_MISMATCH: u16 = 36; // expected hw do not match
const RFAL_ERR_LINK_LOSS: u16 = 37; // Other device's field didn't behave as expected: turned off by Initiator in Passive mode, or AP2P did not turn on field
const RFAL_ERR_INVALID_HANDLE: u16 = 38; // invalid or not initialized device handle
const RFAL_ERR_INCOMPLETE_BYTE: u16 = 40; // Incomplete byte rcvd

// const RFAL_ERR_INCOMPLETE_BYTE_01: u16 = 41; // Incomplete byte rcvd - 1 bit
// const RFAL_ERR_INCOMPLETE_BYTE_02: u16 = 42; // Incomplete byte rcvd - 2 bit
// const RFAL_ERR_INCOMPLETE_BYTE_03: u16 = 43; // Incomplete byte rcvd - 3 bit
// const RFAL_ERR_INCOMPLETE_BYTE_04: u16 = 44; // Incomplete byte rcvd - 4 bit
// const RFAL_ERR_INCOMPLETE_BYTE_05: u16 = 45; // Incomplete byte rcvd - 5 bit
// const RFAL_ERR_INCOMPLETE_BYTE_06: u16 = 46; // Incomplete byte rcvd - 6 bit
// const RFAL_ERR_INCOMPLETE_BYTE_07: u16 = 47; // Incomplete byte rcvd - 7 bit

impl From<u16> for Error {
    fn from(value: u16) -> Self {
        match value {
            RFAL_ERR_NOMEM => Error::NoMem,
            RFAL_ERR_BUSY => Error::Busy,
            RFAL_ERR_IO => Error::Io,
            RFAL_ERR_TIMEOUT => Error::Timeout,
            RFAL_ERR_REQUEST => Error::Request,
            RFAL_ERR_NOMSG => Error::NoMsg,
            RFAL_ERR_PARAM => Error::Param,
            RFAL_ERR_SYSTEM => Error::System,
            RFAL_ERR_FRAMING => Error::Framing,
            RFAL_ERR_OVERRUN => Error::Overrun,
            RFAL_ERR_PROTO => Error::Proto,
            RFAL_ERR_INTERNAL => Error::Internal,
            RFAL_ERR_AGAIN => Error::Again,
            RFAL_ERR_MEM_CORRUPT => Error::MemCorrupt,
            RFAL_ERR_NOT_IMPLEMENTED => Error::NotImplemented,
            RFAL_ERR_PC_CORRUPT => Error::PcCorrupt,
            RFAL_ERR_SEND => Error::Send,
            RFAL_ERR_IGNORE => Error::Ignore,
            RFAL_ERR_SEMANTIC => Error::Semantic,
            RFAL_ERR_SYNTAX => Error::Syntax,
            RFAL_ERR_CRC => Error::Crc,
            RFAL_ERR_NOTFOUND => Error::NotFound,
            RFAL_ERR_NOTUNIQUE => Error::NotUnique,
            RFAL_ERR_NOTSUPP => Error::NotSupp,
            RFAL_ERR_WRITE => Error::Write,
            RFAL_ERR_FIFO => Error::Fifo,
            RFAL_ERR_PAR => Error::Par,
            RFAL_ERR_DONE => Error::Done,
            RFAL_ERR_RF_COLLISION => Error::RfCollision,
            RFAL_ERR_HW_OVERRUN => Error::HwOverrun,
            RFAL_ERR_RELEASE_REQ => Error::ReleaseReq,
            RFAL_ERR_SLEEP_REQ => Error::SleepReq,
            RFAL_ERR_WRONG_STATE => Error::WrongState,
            RFAL_ERR_MAX_RERUNS => Error::MaxReruns,
            RFAL_ERR_DISABLED => Error::Disabled,
            RFAL_ERR_HW_MISMATCH => Error::HwMismatch,
            RFAL_ERR_LINK_LOSS => Error::LinkLoss,
            RFAL_ERR_INVALID_HANDLE => Error::InvalidHandle,
            RFAL_ERR_INCOMPLETE_BYTE => Error::IncompleteByte,
            v => Error::Unknown(v),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

pub(crate) fn result(res: u16) -> Result<()> {
    match res {
        RFAL_ERR_NONE => Ok(()),
        _ => Err(Error::from(res)),
    }
}

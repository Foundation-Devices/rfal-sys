use core::ffi::{c_char, CStr};

static mut RFAL_PLATFORM: Option<Platform> = None;

/// Defines platform HAL functions to be used by RFAL.
pub struct Platform {
    pub spi_poll_send: fn() -> bool,
    pub spi_reset: fn(),
    pub spi_send_cmd: fn(u8, &[u8], bool),
    pub spi_read: fn(&mut u8, &mut [u8]) -> u16,
    pub spi_read_echo: fn() -> bool,
    pub spi_flush: fn(),

    pub handle_error: fn(&CStr, i32),
    pub log: fn(&CStr, usize),

    pub irq_in_pulse_low: fn(),
    pub wait_irq_out_falling_edge: fn(u32) -> bool,

    pub get_ticks_ms: fn() -> u32,
    pub delay_ms: fn(u32),
}

pub fn rfal_platform_set(platform: Platform) {
    unsafe {
        RFAL_PLATFORM.replace(platform);
    }
}

#[no_mangle]
fn ffi_spi_poll_send() -> bool {
    unsafe {
        (RFAL_PLATFORM
            .as_ref()
            .expect("call rfal_platform_set first")
            .spi_poll_send)()
    }
}

#[no_mangle]
fn ffi_spi_reset() {
    unsafe {
        (RFAL_PLATFORM
            .as_ref()
            .expect("call rfal_platform_set first")
            .spi_reset)();
    }
}

#[no_mangle]
fn ffi_spi_send_cmd(cmd: u8, data: *const u8, len: usize, sod: bool) {
    unsafe {
        let data = if len > 0 {
            core::slice::from_raw_parts(data, len)
        } else {
            &[]
        };
        (RFAL_PLATFORM
            .as_ref()
            .expect("call rfal_platform_set first")
            .spi_send_cmd)(cmd, data, sod);
    }
}

#[no_mangle]
fn ffi_spi_read(code: *mut u8, data: *mut u8, len: usize) -> u16 {
    unsafe {
        let code = &mut *code;
        let data = if len > 0 {
            core::slice::from_raw_parts_mut(data, len)
        } else {
            &mut []
        };
        (RFAL_PLATFORM
            .as_ref()
            .expect("call rfal_platform_set first")
            .spi_read)(code, data)
    }
}

#[no_mangle]
fn ffi_spi_read_echo() -> bool {
    unsafe {
        (RFAL_PLATFORM
            .as_ref()
            .expect("call rfal_platform_set first")
            .spi_read_echo)()
    }
}

#[no_mangle]
fn ffi_spi_flush() {
    unsafe {
        (RFAL_PLATFORM
            .as_ref()
            .expect("call rfal_platform_set first")
            .spi_flush)();
    }
}

#[no_mangle]
fn ffi_handle_error(file: *const c_char, line: i32) {
    unsafe {
        let s = CStr::from_ptr(file);
        (RFAL_PLATFORM
            .as_ref()
            .expect("call rfal_platform_set first")
            .handle_error)(s, line);
    }
}

/// # Safety
///
/// This function is unsafe because it expects `msg` to be a valid pointer to a null-terminated
/// string. If `msg` is not a valid pointer or does not point to a null-terminated string, this
/// function may cause undefined behavior.
#[no_mangle]
pub unsafe fn ffi_log(msg: *const c_char, val: usize) {
    let s = CStr::from_ptr(msg);
    (RFAL_PLATFORM
        .as_ref()
        .expect("call rfal_platform_set first")
        .log)(s, val);
}

#[no_mangle]
fn ffi_irq_in_pulse_low() {
    unsafe {
        (RFAL_PLATFORM
            .as_ref()
            .expect("call rfal_platform_set first")
            .irq_in_pulse_low)()
    }
}

#[no_mangle]
fn ffi_wait_irq_out_falling_edge(timeout: u32) -> bool {
    unsafe {
        (RFAL_PLATFORM
            .as_ref()
            .expect("call rfal_platform_set first")
            .wait_irq_out_falling_edge)(timeout)
    }
}

#[no_mangle]
pub fn ffi_get_ticks_ms() -> u32 {
    unsafe {
        (RFAL_PLATFORM
            .as_ref()
            .expect("call rfal_platform_set first")
            .get_ticks_ms)()
    }
}

#[no_mangle]
fn ffi_delay_ms(delay: u32) {
    unsafe {
        (RFAL_PLATFORM
            .as_ref()
            .expect("call rfal_platform_set first")
            .delay_ms)(delay);
    }
}

/// # Safety
///
/// This function is marked as `unsafe` because it does not perform any checks
/// on the pointers passed to it. It is up to the caller to ensure that the
/// pointers are valid and point to null-terminated strings.
#[no_mangle]
pub unsafe fn strcmp(s1: *const i8, s2: *const i8) -> i32 {
    for i in 0.. {
        let s1_i = s1.offset(i);
        let s2_i = s2.offset(i);

        let val = *s1_i as i32 - *s2_i as i32;
        if val != 0 || *s1_i == 0 {
            return val;
        }
    }
    0
}

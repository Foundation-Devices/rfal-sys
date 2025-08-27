use core::ffi::{c_char, CStr};

static mut RFAL_PLATFORM: Option<Platform> = None;

const IRQ_IN: u32 = 0;
const IRQ_OUT: u32 = 1;

/// Defines platform HAL functions to be used by RFAL.
pub struct Platform {
    pub spi_select: fn(),
    pub spi_deselect: fn(),
    pub spi_tx_rx: fn(&[u8], &mut [u8]),

    pub handle_error: fn(&CStr, i32),
    pub log: fn(&CStr, usize),

    pub set_irq_in: fn(bool),
    pub get_irq_out: fn() -> bool,

    pub get_ticks_ms: fn() -> u32,
    pub delay_ms: fn(u32),
}

pub fn rfal_platform_set(platform: Platform) {
    unsafe {
        RFAL_PLATFORM.replace(platform);
    }
}

#[no_mangle]
fn ffi_spi_select() {
    unsafe {
        (RFAL_PLATFORM
            .as_ref()
            .expect("call rfal_platform_set first")
            .spi_select)();
    }
}

#[no_mangle]
fn ffi_spi_deselect() {
    unsafe {
        (RFAL_PLATFORM
            .as_ref()
            .expect("call rfal_platform_set first")
            .spi_deselect)();
    }
}

#[no_mangle]
fn ffi_spi_tx_rx(tx: *const u8, rx: *mut u8, len: usize) {
    unsafe {
        let tx = core::slice::from_raw_parts(tx, len);
        let rx = core::slice::from_raw_parts_mut(rx, len);
        (RFAL_PLATFORM
            .as_ref()
            .expect("call rfal_platform_set first")
            .spi_tx_rx)(tx, rx);
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
fn ffi_gpio_set(port: u32, pin: u32, value: bool) {
    if port == IRQ_IN && pin == IRQ_IN {
        unsafe {
            (RFAL_PLATFORM
                .as_ref()
                .expect("call rfal_platform_set first")
                .set_irq_in)(value)
        }
    }
}

#[no_mangle]
fn ffi_gpio_get(port: u32, pin: u32) -> bool {
    if port == IRQ_OUT && pin == IRQ_OUT {
        unsafe {
            (RFAL_PLATFORM
                .as_ref()
                .expect("call rfal_platform_set first")
                .get_irq_out)()
        }
    } else {
        false
    }
}

#[no_mangle]
fn ffi_irq_out() -> u32 {
    IRQ_OUT
}

#[no_mangle]
fn ffi_irq_in() -> u32 {
    IRQ_IN
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

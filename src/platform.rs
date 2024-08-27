use core::ffi::CStr;

static mut RFAL_PLATFORM: Option<RfalPlatform> = None;

/// Defines platform HAL functions to be used by RFAL.
pub struct RfalPlatform {
    pub delay_ms: fn(u32),

    pub spi_select: fn(),
    pub spi_deselect: fn(),
    pub spi_tx_rx: fn(&[u8], &mut [u8]),

    pub handle_error: fn(&CStr, i32),
    pub log: fn(&CStr, usize),

    pub gpio_set: fn(u32, u32, bool),
    pub gpio_get: fn(u32, u32) -> bool,
    pub gpio_toggle: fn(u32, u32),
    pub ffi_irq_out_pin: fn() -> u32,
    pub ffi_irq_out_port: fn() -> u32,
    pub ffi_irq_in_pin: fn() -> u32,
    pub ffi_irq_in_port: fn() -> u32,

    pub ffi_create_timer: fn(u32) -> u32,
    pub ffi_timer_is_expired: fn(u32) -> bool,
}

pub fn rfal_platform_set(platform: RfalPlatform) -> Result<(), ()> {
    unsafe {
        if RFAL_PLATFORM.is_some() {
            return Err(())
        }

        RFAL_PLATFORM.replace(platform);
    }

    Ok(())
}

#[no_mangle]
fn ffi_delay_ms(delay: u32) {
    unsafe {
        (RFAL_PLATFORM.as_ref().expect("call rfal_platform_set first").delay_ms)(delay);
    }
}

#[no_mangle]
fn ffi_spi_select() {
    unsafe {
        (RFAL_PLATFORM.as_ref().expect("call rfal_platform_set first").spi_select)();
    }
}

#[no_mangle]
fn ffi_spi_deselect() {
    unsafe {
        (RFAL_PLATFORM.as_ref().expect("call rfal_platform_set first").spi_deselect)();
    }
}

#[no_mangle]
fn ffi_spi_tx_rx(tx: *const u8, rx: *mut u8, len: usize) {
    unsafe {
        let tx = core::slice::from_raw_parts(tx, len);
        let rx = core::slice::from_raw_parts_mut(rx, len);
        (RFAL_PLATFORM.as_ref().expect("call rfal_platform_set first").spi_tx_rx)(tx, rx);
    }
}

#[no_mangle]
fn ffi_handle_error(file: *const i8, line: i32) {
    unsafe {
        let s = CStr::from_ptr(file);
        (RFAL_PLATFORM.as_ref().expect("call rfal_platform_set first").handle_error)(s, line);
    }
}

#[no_mangle]
fn ffi_gpio_set(port: u32, pin: u32, value: bool) {
    unsafe {
        (RFAL_PLATFORM.as_ref().expect("call rfal_platform_set first").gpio_set)(port, pin, value);
    }
}

#[no_mangle]
fn ffi_gpio_get(port: u32, pin: u32) -> bool {
    unsafe {
        (RFAL_PLATFORM.as_ref().expect("call rfal_platform_set first").gpio_get)(port, pin)
    }
}

#[no_mangle]
fn ffi_gpio_toggle(port: u32, pin: u32) {
    unsafe {
        (RFAL_PLATFORM.as_ref().expect("call rfal_platform_set first").gpio_toggle)(port, pin);
    }
}

#[no_mangle]
fn ffi_irq_out_pin() -> u32 {
    unsafe {
        (RFAL_PLATFORM.as_ref().expect("call rfal_platform_set first").ffi_irq_out_pin)()
    }
}

#[no_mangle]
fn ffi_irq_out_port() -> u32 {
    unsafe {
        (RFAL_PLATFORM.as_ref().expect("call rfal_platform_set first").ffi_irq_out_port)()
    }
}

#[no_mangle]
fn ffi_irq_in_pin() -> u32 {
    unsafe {
        (RFAL_PLATFORM.as_ref().expect("call rfal_platform_set first").ffi_irq_in_pin)()
    }
}

#[no_mangle]
fn ffi_irq_in_port() -> u32 {
    unsafe {
        (RFAL_PLATFORM.as_ref().expect("call rfal_platform_set first").ffi_irq_in_port)()
    }
}

#[no_mangle]
fn ffi_create_timer(timeout: u32) -> u32 {
    unsafe {
        (RFAL_PLATFORM.as_ref().expect("call rfal_platform_set first").ffi_create_timer)(timeout)
    }
}

#[no_mangle]
fn ffi_timer_is_expired(timer: u32) -> bool {
    unsafe {
        (RFAL_PLATFORM.as_ref().expect("call rfal_platform_set first").ffi_timer_is_expired)(timer)
    }
}

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

#[no_mangle]
pub fn ffi_log(msg: *const i8, len: usize) {
    unsafe {
        let s = CStr::from_ptr(msg);
        (RFAL_PLATFORM.as_ref().expect("call rfal_platform_set first").log)(s, len);
    }
}

#[no_mangle]
pub fn ffi_get_ticks_ms() -> u32 {
    unsafe {
        // TODO
        0
    }
}
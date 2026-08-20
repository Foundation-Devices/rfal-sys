use core::{
    cell::UnsafeCell,
    ffi::{c_char, c_int, CStr},
    mem::MaybeUninit,
    sync::atomic::{AtomicU8, Ordering},
};

const PLATFORM_UNINITIALIZED: u8 = 0;
const PLATFORM_INITIALIZING: u8 = 1;
const PLATFORM_READY: u8 = 2;

static RFAL_PLATFORM: PlatformCell = PlatformCell::new();

/// Defines platform HAL functions to be used by RFAL.
#[derive(Clone, Copy)]
pub struct Platform {
    pub spi_poll_send: fn() -> bool,
    pub spi_reset: fn(),
    pub spi_send_cmd: fn(u8, &[u8], bool),
    pub spi_read: fn(&mut u8, &mut [u8]) -> u16,
    pub spi_read_echo: fn() -> bool,
    pub spi_flush: fn(),

    pub handle_error: fn(&CStr, i32),
    pub log: fn(&CStr, i32),

    pub irq_in_pulse_low: fn(),
    pub wait_irq_out_falling_edge: fn(u32) -> bool,

    pub get_ticks_ms: fn() -> u32,
    pub delay_ms: fn(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlatformAlreadySet;

struct PlatformCell {
    state: AtomicU8,
    platform: UnsafeCell<MaybeUninit<Platform>>,
}

fn assert_platform_send_sync<T: Send + Sync>() {}

const _: fn() = assert_platform_send_sync::<Platform>;

// SAFETY: Platform only contains function pointers. The table is written once
// before the state is published as READY and is never mutated afterwards.
unsafe impl Sync for PlatformCell {}

impl PlatformCell {
    const fn new() -> Self {
        Self {
            state: AtomicU8::new(PLATFORM_UNINITIALIZED),
            platform: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    fn set(&self, platform: Platform) -> Result<(), PlatformAlreadySet> {
        self.state
            .compare_exchange(
                PLATFORM_UNINITIALIZED,
                PLATFORM_INITIALIZING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map_err(|_| PlatformAlreadySet)?;

        unsafe {
            (*self.platform.get()).write(platform);
        }
        self.state.store(PLATFORM_READY, Ordering::Release);

        Ok(())
    }

    fn get(&self) -> &Platform {
        assert!(
            self.state.load(Ordering::Acquire) == PLATFORM_READY,
            "call rfal_platform_set first"
        );

        unsafe { &*(*self.platform.get()).as_ptr() }
    }
}

/// Registers the platform callbacks used by RFAL.
///
/// The platform can be registered only once. Replacing callbacks after RFAL has
/// been initialized is not supported because C callbacks may be executing
/// concurrently.
pub fn rfal_platform_try_set(platform: Platform) -> Result<(), PlatformAlreadySet> {
    RFAL_PLATFORM.set(platform)
}

/// Registers the platform callbacks used by RFAL.
///
/// Panics if the platform has already been registered. Use
/// [`rfal_platform_try_set`] if the caller needs to handle double
/// initialization.
pub fn rfal_platform_set(platform: Platform) {
    rfal_platform_try_set(platform).expect("rfal platform already set");
}

fn platform() -> &'static Platform {
    RFAL_PLATFORM.get()
}

// The symbols below are called from C, so they must use the C calling convention:
// `#[no_mangle]` only controls the symbol name, not the ABI, and the default Rust
// ABI is not a stable cross-language contract. They are also the outermost Rust
// frame of every callback, and `extern "C"` aborts instead of letting a panic
// unwind into C.
//
// Every signature is mirrored by an assertion in `ffi_abi` below and by a
// `_Static_assert` in `ffi_abi_check.c`, so a change on either side of the
// boundary fails the build instead of silently disagreeing at run time.

#[no_mangle]
extern "C" fn ffi_spi_poll_send() -> bool {
    (platform().spi_poll_send)()
}

#[no_mangle]
extern "C" fn ffi_spi_reset() {
    (platform().spi_reset)();
}

/// # Safety
///
/// `data` must be valid for reads of `len` bytes, or `len` must be zero.
#[no_mangle]
unsafe extern "C" fn ffi_spi_send_cmd(cmd: u8, data: *const u8, len: usize, sod: bool) {
    let data = if len > 0 {
        core::slice::from_raw_parts(data, len)
    } else {
        &[]
    };
    (platform().spi_send_cmd)(cmd, data, sod);
}

/// # Safety
///
/// `code` must be valid for reads and writes, and `data` must be valid for
/// writes of `len` bytes, or `len` must be zero.
#[no_mangle]
unsafe extern "C" fn ffi_spi_read(code: *mut u8, data: *mut u8, len: usize) -> u16 {
    let code = &mut *code;
    let data = if len > 0 {
        core::slice::from_raw_parts_mut(data, len)
    } else {
        &mut []
    };
    (platform().spi_read)(code, data)
}

#[no_mangle]
extern "C" fn ffi_spi_read_echo() -> bool {
    (platform().spi_read_echo)()
}

#[no_mangle]
extern "C" fn ffi_spi_flush() {
    (platform().spi_flush)();
}

/// # Safety
///
/// `file` must point to a valid, null-terminated string.
#[no_mangle]
unsafe extern "C" fn ffi_handle_error(file: *const c_char, line: c_int) {
    let s = CStr::from_ptr(file);
    (platform().handle_error)(s, line as i32);
}

/// # Safety
///
/// `msg` must point to a valid, null-terminated string.
#[no_mangle]
unsafe extern "C" fn ffi_log(msg: *const c_char, val: c_int) {
    let s = CStr::from_ptr(msg);
    (platform().log)(s, val as i32);
}

#[no_mangle]
extern "C" fn ffi_irq_in_pulse_low() {
    (platform().irq_in_pulse_low)()
}

#[no_mangle]
extern "C" fn ffi_wait_irq_out_falling_edge(timeout: u32) -> bool {
    (platform().wait_irq_out_falling_edge)(timeout)
}

#[no_mangle]
pub extern "C" fn ffi_get_ticks_ms() -> u32 {
    (platform().get_ticks_ms)()
}

#[no_mangle]
extern "C" fn ffi_delay_ms(delay: u32) {
    (platform().delay_ms)(delay);
}

/// # Safety
///
/// This function is marked as `unsafe` because it does not perform any checks
/// on the pointers passed to it. It is up to the caller to ensure that the
/// pointers are valid and point to null-terminated strings.
#[no_mangle]
pub unsafe extern "C" fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    for i in 0.. {
        let s1_i = s1.offset(i);
        let s2_i = s2.offset(i);

        let val = *s1_i as c_int - *s2_i as c_int;
        if val != 0 || *s1_i == 0 {
            return val;
        }
    }
    0
}

/// Pins the signature of every symbol exported to C.
///
/// These are the Rust half of the ABI contract; `src/ffi_abi_check.c` asserts the
/// same list against the prototypes in `rfal_platform.h`. Changing a signature on
/// one side without the other stops compiling here.
mod ffi_abi {
    use super::*;

    const _: extern "C" fn() -> bool = ffi_spi_poll_send;
    const _: extern "C" fn() = ffi_spi_reset;
    const _: unsafe extern "C" fn(u8, *const u8, usize, bool) = ffi_spi_send_cmd;
    const _: unsafe extern "C" fn(*mut u8, *mut u8, usize) -> u16 = ffi_spi_read;
    const _: extern "C" fn() -> bool = ffi_spi_read_echo;
    const _: extern "C" fn() = ffi_spi_flush;
    const _: unsafe extern "C" fn(*const c_char, c_int) = ffi_handle_error;
    const _: unsafe extern "C" fn(*const c_char, c_int) = ffi_log;
    const _: extern "C" fn() = ffi_irq_in_pulse_low;
    const _: extern "C" fn(u32) -> bool = ffi_wait_irq_out_falling_edge;
    const _: extern "C" fn() -> u32 = ffi_get_ticks_ms;
    const _: extern "C" fn(u32) = ffi_delay_ms;
    const _: unsafe extern "C" fn(*const c_char, *const c_char) -> c_int = strcmp;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spi_poll_send() -> bool {
        true
    }

    fn spi_reset() {}

    fn spi_send_cmd(_: u8, _: &[u8], _: bool) {}

    fn spi_read(_: &mut u8, _: &mut [u8]) -> u16 {
        0
    }

    fn spi_read_echo() -> bool {
        true
    }

    fn spi_flush() {}

    fn handle_error(_: &CStr, _: i32) {}

    fn log(_: &CStr, _: i32) {}

    fn irq_in_pulse_low() {}

    fn wait_irq_out_falling_edge(_: u32) -> bool {
        true
    }

    fn get_ticks_ms() -> u32 {
        0
    }

    fn delay_ms(_: u32) {}

    fn test_platform() -> Platform {
        Platform {
            spi_poll_send,
            spi_reset,
            spi_send_cmd,
            spi_read,
            spi_read_echo,
            spi_flush,
            handle_error,
            log,
            irq_in_pulse_low,
            wait_irq_out_falling_edge,
            get_ticks_ms,
            delay_ms,
        }
    }

    #[test]
    fn set_rejects_double_initialization() {
        let cell = PlatformCell::new();

        assert_eq!(cell.set(test_platform()), Ok(()));
        assert_eq!(cell.set(test_platform()), Err(PlatformAlreadySet));
    }

    #[test]
    fn set_publishes_callbacks() {
        let cell = PlatformCell::new();

        cell.set(test_platform()).unwrap();

        assert!((cell.get().spi_poll_send)());
    }

    #[test]
    #[should_panic(expected = "call rfal_platform_set first")]
    fn get_before_initialization_panics() {
        let cell = PlatformCell::new();

        let _ = cell.get();
    }
}

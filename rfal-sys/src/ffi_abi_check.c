/*
 * Compile-time check of the Rust <-> C platform ABI.
 *
 * The `ffi_*` symbols and `strcmp` are implemented in Rust (src/platform.rs) and
 * called from the ST middleware through the macros in rfal_platform.h. Nothing in
 * the link step compares the two sides: the linker only matches symbol names, so a
 * signature or calling-convention drift would only show up as corrupted arguments
 * at run time.
 *
 * Each assertion below pins one prototype to the audited signature. The Rust half
 * of the same list lives in the `ffi_abi` module of src/platform.rs; changing a
 * signature on either side without the other stops compiling.
 */

#include <string.h>

#include "rfal_platform.h"

#define FFI_ABI_CHECK(fn, ...)                                                 \
    _Static_assert(__builtin_types_compatible_p(__typeof__(fn), __VA_ARGS__),  \
                   #fn " signature does not match the audited platform ABI")

FFI_ABI_CHECK(ffi_irq_in_pulse_low,          void (void));
FFI_ABI_CHECK(ffi_wait_irq_out_falling_edge, bool (uint32_t));
FFI_ABI_CHECK(ffi_delay_ms,                  void (uint32_t));
FFI_ABI_CHECK(ffi_get_ticks_ms,              uint32_t (void));
FFI_ABI_CHECK(ffi_handle_error,              void (const char *, int));
FFI_ABI_CHECK(ffi_log,                       void (const char *, int));
FFI_ABI_CHECK(ffi_spi_poll_send,             bool (void));
FFI_ABI_CHECK(ffi_spi_reset,                 void (void));
FFI_ABI_CHECK(ffi_spi_send_cmd,              void (uint8_t, const uint8_t *, size_t, bool));
FFI_ABI_CHECK(ffi_spi_read,                  uint16_t (uint8_t *, uint8_t *, size_t));
FFI_ABI_CHECK(ffi_spi_read_echo,             bool (void));
FFI_ABI_CHECK(ffi_spi_flush,                 void (void));
FFI_ABI_CHECK(strcmp,                        int (const char *, const char *));

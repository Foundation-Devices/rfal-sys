<!--
SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
SPDX-License-Identifier: GPL-3.0-or-later
-->

# `rfal`

Rust bindings for STMicroelectronics RFAL and NDEF middlewares.

RFAL and NDEF are proprietary source C middlewares written by STMicroelectronics for their ST25R NFC chips.

## High-level bindings

The `rfal` crate contains high-level easy-to-use Rust safe bindings for the RFAL/NDEF middlewares.

## Low-level raw bindings

The `rfal-sys` crate contain low-level bindings, matching 1-1 with the RFAL/NDEF middleware C headers.

They are generated with `bindgen`.

## License

This repo includes the RFAL/NDEF middleware headers, which are licensed under [ST's proprietary license](LICENSE-ST).
Generated `binding.rs` files are a derived work of the headers, so they are also subject to ST's license.

The Card Emulation source is only provided by ST on demand so that they can have a dedicated follow up of their customers on this feature.
If you want to have Card Emulation enabled, you must request the file from ST and place it manually in `rfal-sys/licensed/st25r95_com_ce.c`.

The high level bindings (`rfal`) are licensed under GPL-3.0-or-later ([LICENSE-GPL](LICENSE-GPL-3.0-OR-LATER)).

use std::env;
use std::path::PathBuf;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = cc::Build::new();

    builder
        .flag("-std=c99")
        .flag("-fno-short-enums")
        .flag("-mno-unaligned-access")
        .define("ST25R95", "true")
        .define("ST25R95_DEBUG", "false")
        .define("ST25R95_INTERFACE_SPI", "true")
        .include(".") // Allow the compiler to find rfal_platform.h and rfal_features.h
        .include("utils/Inc")
        .include("RFAL/source/st25r95") // Allow the compiler to find rfal_analogConfigTbl.h
        .include("RFAL/include")
        .include("NDEF/include")
        .include("NDEF/include/message")
        .include("NDEF/include/poller")
        .file("RFAL/source/st25r95/st25r95.c")
        .file("RFAL/source/st25r95/st25r95_com.c")
        .file("RFAL/source/st25r95/st25r95_com_spi.c")
        .file("RFAL/source/st25r95/rfal_rfst25r95.c")
        .file("RFAL/source/rfal_st25tb.c")
        .file("RFAL/source/rfal_analogConfig.c")
        .file("RFAL/source/rfal_crc.c")
        .file("RFAL/source/rfal_nfc.c")
        .file("RFAL/source/rfal_nfca.c")
        .file("RFAL/source/rfal_nfcb.c")
        .file("RFAL/source/rfal_nfcf.c")
        .file("RFAL/source/rfal_nfcv.c")
        .file("RFAL/source/rfal_isoDep.c")
        .file("RFAL/source/rfal_nfcDep.c")
        .file("RFAL/source/rfal_t1t.c")
        .file("RFAL/source/rfal_t2t.c")
        .file("RFAL/source/rfal_t4t.c")
        .file("NDEF/source/message/ndef_message.c")
        .compile("rfal-sys");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let builder = bindgen::Builder::default()
        .use_core()
        .size_t_is_usize(true)
        .derive_debug(true)
        .header("RFAL/include/rfal_utils.h")
        .header("RFAL/include/rfal_nfc.h")
        .header("RFAL/include/rfal_nfca.h")
        .header("RFAL/include/rfal_nfcb.h")
        .header("RFAL/include/rfal_rf.h")
        .rustified_enum("rfalNfcState")
        .rustified_enum("rfalNfcDevType")
        .rustified_enum("rfalNfcDeactivateType")
        .clang_arg("--target=armv7a-none-eabi")
        .clang_arg("-I.")
        .use_core()
        .ctypes_prefix("cty");

    let bindings = builder
        .generate()
        .expect("Unable to generate bindings");

    let out_file = out_dir.join("bindings.rs");

    bindings
        .write_to_file(out_file)
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=rfal_platform.h");
    println!("cargo:rerun-if-changed=rfal_features.h");

    Ok(())
}

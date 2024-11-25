use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src_dir = "ST25NFC_Embedded_Lib_ST25R95_1.7.0/Middlewares/ST";
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let marker_file = PathBuf::from(src_dir).join("patched_marker");
    if !marker_file.exists() {
        let patch_files = [
            "0001_localize_string_h.patch",
            "0002_globalize_ST25R95_DEBUG.patch",
        ];

        for patch_file in patch_files.iter() {
            Command::new("patch")
                .arg("-p2")
                .arg("--binary")
                .arg("-i")
                .arg(format!("patches/{patch_file}"))
                .output()
                .expect("failed to execute patch {:patch_file}");
        }
        fs::write(&marker_file, "patched").expect("Can't create marker file");
    }

    env::set_var("CC", "arm-none-eabi-gcc");

    let mut builder = cc::Build::new();
    builder
        .flag("-std=c99")
        .flag("-fno-short-enums")
        .flag("-mno-unaligned-access") // this is arm-none-eabi dependant
        .define("USE_LOGGER", "LOGGER_ON")
        .define("ST25R95", "true")
        .define("ST25R95_DEBUG", "false")
        .define("ST25R95_INTERFACE_SPI", "true")
        .include("src")
        .include(format!("{src_dir}/st25r_common/firmware/STM/utils/Inc"))
        .include(format!("{src_dir}/RFAL/source/st25r95"))
        .include(format!("{src_dir}/RFAL/include"))
        .include(format!("{src_dir}/NDEF/include"))
        .include(format!("{src_dir}/NDEF/include/message"))
        .include(format!("{src_dir}/NDEF/include/poller"))
        .file("src/utils.c")
        .file(format!("{src_dir}/RFAL/source/st25r95/st25r95.c"))
        .file(format!("{src_dir}/RFAL/source/st25r95/st25r95_com.c"))
        .file(format!("{src_dir}/RFAL/source/st25r95/st25r95_com_spi.c"))
        .file(format!("{src_dir}/RFAL/source/st25r95/rfal_rfst25r95.c"))
        .file(format!("{src_dir}/RFAL/source/rfal_st25tb.c"))
        .file(format!("{src_dir}/RFAL/source/rfal_st25xv.c"))
        .file(format!("{src_dir}/RFAL/source/rfal_analogConfig.c"))
        .file(format!("{src_dir}/RFAL/source/rfal_crc.c"))
        .file(format!("{src_dir}/RFAL/source/rfal_iso15693_2.c"))
        .file(format!("{src_dir}/RFAL/source/rfal_nfc.c"))
        .file(format!("{src_dir}/RFAL/source/rfal_nfca.c"))
        .file(format!("{src_dir}/RFAL/source/rfal_nfcb.c"))
        .file(format!("{src_dir}/RFAL/source/rfal_nfcf.c"))
        .file(format!("{src_dir}/RFAL/source/rfal_nfcv.c"))
        .file(format!("{src_dir}/RFAL/source/rfal_isoDep.c"))
        .file(format!("{src_dir}/RFAL/source/rfal_nfcDep.c"))
        .file(format!("{src_dir}/RFAL/source/rfal_t1t.c"))
        .file(format!("{src_dir}/RFAL/source/rfal_t2t.c"))
        .file(format!("{src_dir}/RFAL/source/rfal_t4t.c"))
        .file(format!("{src_dir}/NDEF/source/message/ndef_record.c"))
        // .file(format!("{src_dir}/NDEF/source/message/ndef_types.c"))
        // .file(format!("{src_dir}/NDEF/source/message/ndef_type_aar.c"))
        // .file(format!("{src_dir}/NDEF/source/message/ndef_type_bluetooth.c"))
        // .file(format!("{src_dir}/NDEF/source/message/ndef_type_deviceinfo.c"))
        // .file(format!("{src_dir}/NDEF/source/message/ndef_type_empty.c"))
        // .file(format!("{src_dir}/NDEF/source/message/ndef_type_flat.c"))
        // .file(format!("{src_dir}/NDEF/source/message/ndef_type_media.c"))
        // .file(format!("{src_dir}/NDEF/source/message/ndef_type_text.c"))
        // .file(format!("{src_dir}/NDEF/source/message/ndef_type_tnep.c"))
        // .file(format!("{src_dir}/NDEF/source/message/ndef_type_uri.c"))
        // .file(format!("{src_dir}/NDEF/source/message/ndef_type_vcard.c"))
        // .file(format!("{src_dir}/NDEF/source/message/ndef_type_wifi.c"))
        // .file(format!("{src_dir}/NDEF/source/message/ndef_type_wlc.c"))
        // .file(format!("{src_dir}/NDEF/source/message/ndef_type_wpcwlc.c"))
        .file(format!("{src_dir}/NDEF/source/message/ndef_message.c"))
        .file(format!("{src_dir}/NDEF/source/poller/ndef_t2t.c"))
        .file(format!("{src_dir}/NDEF/source/poller/ndef_t3t.c"))
        .file(format!("{src_dir}/NDEF/source/poller/ndef_t4t.c"))
        .file(format!("{src_dir}/NDEF/source/poller/ndef_t5t.c"))
        .file(format!("{src_dir}/NDEF/source/poller/ndef_t5t_rf.c"))
        .file(format!("{src_dir}/NDEF/source/poller/ndef_poller.c"))
        .file(format!("{src_dir}/NDEF/source/poller/ndef_poller_rf.c"))
        .file(format!(
            "{src_dir}/NDEF/source/poller/ndef_poller_message.c"
        ))
        .compile("rfal-sys");

    let builder = bindgen::Builder::default()
        .header(format!("{src_dir}/RFAL/include/rfal_utils.h"))
        .header(format!("{src_dir}/RFAL/include/rfal_nfc.h"))
        .header(format!("{src_dir}/RFAL/include/rfal_nfca.h"))
        .header(format!("{src_dir}/RFAL/include/rfal_nfcb.h"))
        .header(format!("{src_dir}/RFAL/include/rfal_rf.h"))
        // .header(format!("{src_dir}/NDEF/include/message/ndef_buffer.h"))
        // .header(format!("{src_dir}/NDEF/include/message/ndef_record.h"))
        // .header(format!("{src_dir}/NDEF/include/message/ndef_message.h"))
        // .header(format!("{src_dir}/NDEF/include/message/ndef_type_aar.h"))
        // .header(format!("{src_dir}/NDEF/include/message/ndef_type_bluetooth.h"))
        // .header(format!("{src_dir}/NDEF/include/message/ndef_type_deviceinfo.h"))
        // .header(format!("{src_dir}/NDEF/include/message/ndef_type_empty.h"))
        // .header(format!("{src_dir}/NDEF/include/message/ndef_type_flat.h"))
        // .header(format!("{src_dir}/NDEF/include/message/ndef_type_media.h"))
        // .header(format!("{src_dir}/NDEF/include/message/ndef_type_text.h"))
        // .header(format!("{src_dir}/NDEF/include/message/ndef_type_tnep.h"))
        // .header(format!("{src_dir}/NDEF/include/message/ndef_type_uri.h"))
        // .header(format!("{src_dir}/NDEF/include/message/ndef_type_vcard.h"))
        // .header(format!("{src_dir}/NDEF/include/message/ndef_type_wifi.h"))
        // .header(format!("{src_dir}/NDEF/include/message/ndef_type_wlc.h"))
        // .header(format!("{src_dir}/NDEF/include/message/ndef_type_wpcwlc.h"))
        .header(format!("{src_dir}/NDEF/include/poller/ndef_poller.h"))
        .rustified_enum("ndefDeviceType")
        .rustified_enum("ndefState")
        .rustified_enum("rfal14443AShortFrameCmd")
        .rustified_enum("rfalBitRate")
        .rustified_enum("rfalComplianceMode")
        .rustified_enum("rfalEHandling")
        .rustified_enum("rfalFeliCaPollSlots")
        .rustified_enum("rfalIsoDepFSx")
        .rustified_enum("rfalIsoDepFSxI")
        .rustified_enum("rfalLmNfcidLen")
        .rustified_enum("rfalLmState")
        .rustified_enum("rfalLpMode")
        .rustified_enum("rfalMode")
        .rustified_enum("rfalNfcaListenDeviceType")
        .rustified_enum("rfalNfcbSensCmd")
        .rustified_enum("rfalNfcbSlots")
        .rustified_enum("rfalNfcDeactivateType")
        .rustified_enum("rfalNfcDepCommMode")
        .rustified_enum("rfalNfcDepRole")
        .rustified_enum("rfalNfcDevType")
        .rustified_enum("rfalNfcRfInterface")
        .rustified_enum("rfalNfcState")
        .rustified_enum("rfalNfcvNumSlots")
        .rustified_enum("rfalState")
        .rustified_enum("rfalT1Tcmds")
        .rustified_enum("rfalT4tCmds")
        .rustified_enum("rfalTransceiveState")
        .rustified_enum("rfalWumPeriod")
        .rustified_enum("rfalWumState")
        .clang_arg("--target=armv7a-none-eabi")
        .clang_arg("-I./src")
        .clang_arg(format!("-I./{src_dir}/st25r_common/firmware/STM/utils/Inc"))
        .clang_arg(format!("-I./{src_dir}/RFAL/source/st25r95"))
        .clang_arg(format!("-I./{src_dir}/RFAL/include"))
        .clang_arg(format!("-I./{src_dir}/NDEF/include"))
        .clang_arg(format!("-I./{src_dir}/NDEF/include/message"))
        .use_core()
        .generate_comments(false)
        .ctypes_prefix("cty");

    let bindings = builder.generate().expect("Unable to generate bindings");

    let out_file = out_dir.join("bindings.rs");

    bindings
        .write_to_file(out_file)
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=rfal_platform.h");
    println!("cargo:rerun-if-changed=rfal_features.h");
    println!("cargo:rerun-if-changed=ndef_config.h");

    Ok(())
}

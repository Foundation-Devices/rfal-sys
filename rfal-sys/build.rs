use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src_dir = "ST25NFC_Embedded_Lib_ST25R95_1.7.0/Middlewares/ST";
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let ce = fs::copy(
        "licensed/st25r95_com_ce.c",
        format!("{src_dir}/RFAL/source/st25r95/st25r95_com_ce.c"),
    )
    .is_ok();

    let marker_file = PathBuf::from(src_dir).join("patched_marker");
    if !marker_file.exists() {
        let mut patch_files = vec![
            "0001_localize_string_h.patch",
            "0002_globalize_ST25R95_DEBUG.patch",
            "0003_big_spi_xfers.patch",
            "0005_nfc_a_only.patch",
            "0006_optimize_reg_modify.patch",
            "0007_optimize_flush.patch",
        ];
        if ce {
            patch_files.push("9999_card_emulation.patch");
        }

        for patch_file in patch_files.iter() {
            let patch_output = Command::new("patch")
                .arg("-p2")
                .arg("--binary")
                .arg("-i")
                .arg(format!("patches/{patch_file}"))
                .output()
                .expect("failed to execute patch {:patch_file}");
            if !patch_output.status.success() {
                panic!("Patch {patch_file} failed: {patch_output:?}");
            }
        }
        fs::write(&marker_file, "patched").expect("Can't create marker file");
    }

    env::set_var("CC", "arm-none-eabi-gcc");
    env::set_var("CXX", "arm-none-eabi-g++");
    env::set_var("AR", "arm-none-eabi-ar");
    env::set_var("RANLIB", "arm-none-eabi-ranlib");

    let mut builder = cc::Build::new();
    builder
        .flag("-std=c99")
        .flag("-fno-short-enums")
        .flag("-mno-unaligned-access") // this is arm-none-eabi dependant
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
        ));
    if ce {
        builder.file(format!("{src_dir}/RFAL/source/st25r95/st25r95_com_ce.c"));
    }
    builder.compile("rfal-sys");

    // Run arm-none-eabi-gcc -print-libgcc-file-name
    let output = Command::new("arm-none-eabi-gcc")
        .arg("-print-libgcc-file-name")
        .output()
        .ok()
        .expect("Failed to run arm-none-eabi-gcc -print-libgcc-file-name");
    // Check if the command was successful
    if !output.status.success() {
        panic!("Failed to run arm-none-eabi-gcc, did you installed it ?");
    }
    // Convert output to string and trim
    let libgcc_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    // libgcc.a is typically in
    // /usr/lib/gcc/arm-none-eabi/13.2.1/libgcc.a on ubuntu (manual and docker)
    // /nix/store/ih9psjpxn2pbbzw4klr9s6hmmngc52n8-gcc-arm-embedded-14.3.rel1/bin/../lib/gcc/arm-none-eabi/14.3.1/libgcc.a using the nix flake
    let version_path = PathBuf::from(&libgcc_path)
        .parent()
        .map(|p| p.to_path_buf())
        .expect("Failed to get version directory");
    // println!("cargo:warning=version_path: {}", version_path.display());
    let nixpkg_path = PathBuf::from(&libgcc_path)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .expect("Failed to get nixpkg_path directory");
    // println!("cargo:warning=nixpkg_path: {}", nixpkg_path.display());

    bindgen::Builder::default()
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
        .clang_arg("-nostdinc") // Disable standard includes (useful for bare-metal)
        .clang_arg(format!("-I{}/include", version_path.display()))
        .clang_arg(format!("-I{}/arm-none-eabi/include", nixpkg_path.display())) // This one resolve in `/usr/arm-none-eabi/include` on ubuntu, which doesn't exists but doesn't prevet building
        .use_core()
        .generate_comments(false)
        .ctypes_prefix("cty")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=rfal_platform.h");
    println!("cargo:rerun-if-changed=rfal_features.h");
    println!("cargo:rerun-if-changed=ndef_config.h");

    Ok(())
}

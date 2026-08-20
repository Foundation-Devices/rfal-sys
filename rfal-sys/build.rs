use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Prebuilt Card Emulation object, spliced into the archive after `cc` runs.
///
/// The matching `.c` is only distributed by ST on request, so it lives in the
/// private `rfal-sys-priv` repository and nothing here compiles it.
const LICENSED_OBJECT: &str = "licensed/723cc7b38d33199c-st25r95_com_ce.o";

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src_dir = "ST25NFC_Embedded_Lib_ST25R95_1.7.0/Middlewares/ST";
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    env::set_var("CC", "arm-none-eabi-gcc");
    env::set_var("CXX", "arm-none-eabi-g++");
    env::set_var("AR", "arm-none-eabi-ar");
    env::set_var("RANLIB", "arm-none-eabi-ranlib");

    // Include directories of the native build. They are also the directories
    // Cargo is asked to watch, which is what covers the transitive headers and
    // the rfal_features.h / ndef_config.h configuration headers.
    let includes = [
        format!("{src_dir}/st25r_common/firmware/STM/utils/Inc"),
        format!("{src_dir}/RFAL/source/st25r95"),
        format!("{src_dir}/RFAL/include"),
        format!("{src_dir}/NDEF/include"),
        format!("{src_dir}/NDEF/include/message"),
        format!("{src_dir}/NDEF/include/poller"),
    ];

    // Compiled translation units. Only a subset of the NDEF type and message
    // sources is enabled, to save flash; uncomment one to add a type.
    let sources = [
        // Compile-time check of the Rust <-> C platform ABI, see src/platform.rs
        "src/ffi_abi_check.c".to_string(),
        format!("{src_dir}/RFAL/source/st25r95/st25r95.c"),
        format!("{src_dir}/RFAL/source/st25r95/st25r95_com.c"),
        format!("{src_dir}/RFAL/source/st25r95/st25r95_com_spi.c"),
        format!("{src_dir}/RFAL/source/st25r95/rfal_rfst25r95.c"),
        format!("{src_dir}/RFAL/source/rfal_st25tb.c"),
        format!("{src_dir}/RFAL/source/rfal_st25xv.c"),
        format!("{src_dir}/RFAL/source/rfal_analogConfig.c"),
        format!("{src_dir}/RFAL/source/rfal_crc.c"),
        format!("{src_dir}/RFAL/source/rfal_iso15693_2.c"),
        format!("{src_dir}/RFAL/source/rfal_nfc.c"),
        format!("{src_dir}/RFAL/source/rfal_nfca.c"),
        format!("{src_dir}/RFAL/source/rfal_nfcb.c"),
        format!("{src_dir}/RFAL/source/rfal_nfcf.c"),
        format!("{src_dir}/RFAL/source/rfal_nfcv.c"),
        format!("{src_dir}/RFAL/source/rfal_isoDep.c"),
        format!("{src_dir}/RFAL/source/rfal_nfcDep.c"),
        format!("{src_dir}/RFAL/source/rfal_t1t.c"),
        format!("{src_dir}/RFAL/source/rfal_t2t.c"),
        format!("{src_dir}/RFAL/source/rfal_t4t.c"),
        format!("{src_dir}/NDEF/source/message/ndef_record.c"),
        // format!("{src_dir}/NDEF/source/message/ndef_types.c"),
        // format!("{src_dir}/NDEF/source/message/ndef_type_aar.c"),
        // format!("{src_dir}/NDEF/source/message/ndef_type_bluetooth.c"),
        // format!("{src_dir}/NDEF/source/message/ndef_type_deviceinfo.c"),
        // format!("{src_dir}/NDEF/source/message/ndef_type_empty.c"),
        // format!("{src_dir}/NDEF/source/message/ndef_type_flat.c"),
        // format!("{src_dir}/NDEF/source/message/ndef_type_media.c"),
        // format!("{src_dir}/NDEF/source/message/ndef_type_text.c"),
        // format!("{src_dir}/NDEF/source/message/ndef_type_tnep.c"),
        // format!("{src_dir}/NDEF/source/message/ndef_type_uri.c"),
        // format!("{src_dir}/NDEF/source/message/ndef_type_vcard.c"),
        // format!("{src_dir}/NDEF/source/message/ndef_type_wifi.c"),
        // format!("{src_dir}/NDEF/source/message/ndef_type_wlc.c"),
        // format!("{src_dir}/NDEF/source/message/ndef_type_wpcwlc.c"),
        format!("{src_dir}/NDEF/source/message/ndef_message.c"),
        format!("{src_dir}/NDEF/source/poller/ndef_t2t.c"),
        format!("{src_dir}/NDEF/source/poller/ndef_t3t.c"),
        format!("{src_dir}/NDEF/source/poller/ndef_t4t.c"),
        format!("{src_dir}/NDEF/source/poller/ndef_t5t.c"),
        format!("{src_dir}/NDEF/source/poller/ndef_t5t_rf.c"),
        format!("{src_dir}/NDEF/source/poller/ndef_poller.c"),
        format!("{src_dir}/NDEF/source/poller/ndef_poller_rf.c"),
        format!("{src_dir}/NDEF/source/poller/ndef_poller_message.c"),
    ];

    // Root headers handed to bindgen. Exposing a new C function means adding the
    // header that declares it here.
    let headers = [
        format!("{src_dir}/RFAL/include/rfal_utils.h"),
        format!("{src_dir}/RFAL/include/rfal_nfc.h"),
        format!("{src_dir}/RFAL/include/rfal_nfca.h"),
        format!("{src_dir}/RFAL/include/rfal_nfcb.h"),
        format!("{src_dir}/RFAL/include/rfal_rf.h"),
        // format!("{src_dir}/NDEF/include/message/ndef_buffer.h"),
        // format!("{src_dir}/NDEF/include/message/ndef_record.h"),
        // format!("{src_dir}/NDEF/include/message/ndef_message.h"),
        // format!("{src_dir}/NDEF/include/message/ndef_type_aar.h"),
        // format!("{src_dir}/NDEF/include/message/ndef_type_bluetooth.h"),
        // format!("{src_dir}/NDEF/include/message/ndef_type_deviceinfo.h"),
        // format!("{src_dir}/NDEF/include/message/ndef_type_empty.h"),
        // format!("{src_dir}/NDEF/include/message/ndef_type_flat.h"),
        // format!("{src_dir}/NDEF/include/message/ndef_type_media.h"),
        // format!("{src_dir}/NDEF/include/message/ndef_type_text.h"),
        // format!("{src_dir}/NDEF/include/message/ndef_type_tnep.h"),
        // format!("{src_dir}/NDEF/include/message/ndef_type_uri.h"),
        // format!("{src_dir}/NDEF/include/message/ndef_type_vcard.h"),
        // format!("{src_dir}/NDEF/include/message/ndef_type_wifi.h"),
        // format!("{src_dir}/NDEF/include/message/ndef_type_wlc.h"),
        // format!("{src_dir}/NDEF/include/message/ndef_type_wpcwlc.h"),
        format!("{src_dir}/NDEF/include/poller/ndef_poller.h"),
    ];

    let mut builder = cc::Build::new();
    builder
        .flag("-std=c99")
        .flag("-fno-short-enums")
        .flag("-fno-omit-frame-pointer") // enable full backtrace
        .flag("-mno-unaligned-access") // this is arm-none-eabi dependant
        .pic(true)
        .flag("-fPIC")
        .define("ST25R95", "true")
        .define("ST25R95_DEBUG", "false")
        .define("ST25R95_INTERFACE_SPI", "true")
        // src/ comes first so rfal_platform.h shadows ST's own platform header
        .include("src")
        .includes(&includes)
        .files(&sources);
    builder.compile("rfal-sys");

    // post-process the archive file to add the licensed object
    let mut archiver = builder.get_archiver();
    archiver.args([
        "r",
        out_dir.join("librfal-sys.a").to_str().unwrap(),
        LICENSED_OBJECT,
    ]);
    let status = archiver.status().expect("failed to run archiver");
    assert!(status.success());

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

    let mut bindings = bindgen::Builder::default();
    for header in &headers {
        bindings = bindings.header(header);
    }
    bindings = bindings
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
        .clang_arg("-I./src");
    for include in &includes {
        bindings = bindings.clang_arg(format!("-I./{include}"));
    }
    bindings
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

    // Declare every input of the native build, so Cargo reruns this script when
    // one of them changes and only then. The include directories are emitted as
    // directories: Cargo scans them recursively, which covers the transitive
    // headers and the rfal_features.h / ndef_config.h configuration headers. The
    // ST doc/ directories sit outside them and are therefore ignored.
    //
    // src/ is deliberately not emitted as a directory: only these two files are
    // inputs, and watching the whole directory would rebuild the C sources on
    // every edit to the Rust ones.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/rfal_platform.h");
    println!("cargo:rerun-if-changed={LICENSED_OBJECT}");
    for path in includes.iter().chain(sources.iter()).chain(headers.iter()) {
        println!("cargo:rerun-if-changed={path}");
    }

    Ok(())
}

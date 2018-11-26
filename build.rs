extern crate cc;
extern crate bindgen;

use cc::Build;
use std::collections::{HashMap, HashSet};

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Put the linker script somewhere the linker can find it
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // If any of these files/folders change, we should regenerate
    //   the whole C + bindings component
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=bindings.h");

    // We're going to generate app_config.h from their feature
    // selection, so let's extract that from the env.
    let features: HashSet<_> = env::vars()
        .filter_map(|(k, _)| {
            if k.starts_with("CARGO_FEATURE_") && k != "CARGO_FEATURE_DEFAULT" {
                Some(k[14..].to_owned())
            } else {
                None
            }
        })
        .collect();
    write_app_config(&out, &features);

    let mut info = SdkInfo::default();
    info.add_from_path(&PathBuf::from("nRF5-sdk"));
    info.add_from_path(&PathBuf::from("shims"));

    for src in info.srcs.iter() {
        println!("cargo:rerun-if-changed={}", src.display());
    }
    for hdr in info.hdrs.iter() {
        println!("cargo:rerun-if-changed={}", hdr.display());
    }

    // process_linker_file(&out);
    generate_ble(&out, &info);
    make_c_deps(&out, &info, &features);
}

/// Emit app_config.h based on the enabled features.  This is used
/// to override things in sdk_config.h
fn write_app_config(out: &PathBuf, features: &HashSet<String>) {
    let mut app_config = File::create(out.join("app_config.h")).unwrap();
    for feature in features.iter() {
        writeln!(app_config, "#define {}_ENABLED 1", feature).ok();
    }
}

#[derive(Default)]
struct SdkInfo {
    /// Things to compile
    srcs: Vec<PathBuf>,
    /// Headers to depend upon
    hdrs: Vec<PathBuf>,
    /// All visited dirs (we'll add includes for them).
    dirs: Vec<PathBuf>,
}

impl SdkInfo {
    fn add_from_path(&mut self, path: &PathBuf) {
        self.dirs.push(path.clone());
        for entry in path.read_dir().expect("read_dir failed") {
            if let Ok(entry) = entry {
                let base_name = entry.file_name().into_string().unwrap();
                if base_name.starts_with(".") {
                    continue;
                }

                let file_type = entry.metadata().unwrap().file_type();
                if file_type.is_dir() {
                    self.add_from_path(&entry.path());
                    continue;
                }
                if file_type.is_file() {
                    if base_name.ends_with(".h") {
                        self.hdrs.push(entry.path());
                        continue;
                    }

                    if base_name.ends_with(".c") || base_name.ends_with(".S") {
                        self.srcs.push(entry.path());
                        continue;
                    }
                }
            }
        }
    }
}

// fn process_linker_file(out: &PathBuf) {
//     // Copy over the target specific linker script
//     File::create(out.join("nrf52dk-sys.ld"))
//         .unwrap()
//         .write_all(include_bytes!("nrf52dk-sys.ld"))
//         .unwrap();

//     // Also copy the nrf general linker script
//     File::create(out.join("nrf5x_common.ld"))
//         .unwrap()
//         .write_all(include_bytes!("nrf5x_common.ld"))
//         .unwrap();

//     println!("cargo:rustc-link-search={}", out.display());
// }

fn make_c_deps(out: &PathBuf, info: &SdkInfo, features: &HashSet<String>) {
    let mut config = Build::new();

    config.out_dir(out);

    for f in FLAGS {
        config.flag(f);
    }

    for &(var, val) in DEFINES {
        config.define(var, val);
    }

    let feat_map = compile_src_to_feat();
    for f in info.srcs.iter() {
        if is_src_enabled(f, &feat_map, features) {
            config.file(f);
        }
    }

    // out is where we find the app_config.h that we generate
    // from the enabled features
    config.include(out);
    for i in info.dirs.iter() {
        config.include(i);
    }

    config.compile("libnrf.a");

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rustc-link-lib=static=nrf");
}

/// Extract the default include path from the target compiler
fn find_system_includes() -> Vec<PathBuf> {
    let output = Command::new("arm-none-eabi-gcc")
        .arg("-E")
        .arg("-Wp,-v")
        .arg("-xc")
        .arg("/dev/null")
        .arg("-o/dev/null")
        .output()
        .expect("failed to invoke arg-none-eabi-gcc; it needs to be in your PATH");

    let mut res = Vec::new();
    for line in String::from_utf8_lossy(&output.stderr).split("\n") {
        if line.starts_with(" ") {
            res.push(PathBuf::from(line.trim()));
        }
    }

    res
}

fn generate_ble(out: &PathBuf, info: &SdkInfo) {

    static CLANG_ARGS: &[&str] = &[
        "-nostdlib",
        "-nostdinc",
        "-ffreestanding",
        "-D__CMSIS_GCC_H",
        "-DSVCALL_AS_NORMAL_FUNCTION",
        "-fshort-enums",
    ];

    let mut search_paths = vec![format!("-I{}", out.display())];

    for inc in find_system_includes() {
        search_paths.push(format!("-I{}", inc.display()));
    }

    for inc in info.dirs.iter() {
        search_paths.push(format!("-I{}", inc.display()));
    }

    let mut defines = Vec::new();
    for &(var, oval) in DEFINES {
        match oval {
            None => defines.push(format!("-D{}", var)),
            Some(val) => defines.push(format!("-D{}={}", var, val)),
        };
    }

    let target = env::var("TARGET").unwrap();
    let target_args: &[&str] = &[
        "-target",
        &target,
    ];

    let bindings = bindgen::Builder::default()
        .header("bindings.h")
        .use_core()
        .ctypes_prefix("ctypes")
        .derive_default(true)
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .blacklist_type("IRQn_Type")
        .blacklist_type("__va_list")
        .clang_args(CLANG_ARGS)
        .clang_args(defines)
        .clang_args(search_paths)
        .clang_args(target_args)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

/// Build SRC_TO_FEAT into something using PathBufs
fn compile_src_to_feat() -> HashMap<PathBuf, String> {
    let mut map = HashMap::new();
    for &(file, feat) in SRC_TO_FEAT.iter() {
        map.insert(PathBuf::from(file), feat.to_owned());
    }
    map
}

/// Test whether `src` has any prefixes in the SRC_TO_FEAT map,
/// and if it does whether the RHS is a disabled feature.
/// Returns true if the src is considered to be enabled,
/// false otherwise.
fn is_src_enabled(
    src: &PathBuf,
    feat_map: &HashMap<PathBuf, String>,
    features: &HashSet<String>,
) -> bool {
    for (prefix, feat) in feat_map.iter() {
        if src.starts_with(prefix) {
            if !features.contains(feat) {
                return false;
            }
        }
    }
    true
}

static FLAGS: &[&str] = &[
    "-std=c99",
    "-mcpu=cortex-m4",
    "-mthumb",
    "-mabi=aapcs",
    "-mfloat-abi=hard",
    "-mfpu=fpv4-sp-d16",
    "-ffunction-sections",
    "-fdata-sections",
    "-fno-pic",
    "-fno-strict-aliasing",
    "-fno-builtin",
    // the headers are riddled with unused parameters and emit
    // hundreds of warnings: suppress them.
    "-Wno-unused-parameter",
    "-Wno-sign-compare",
    "-Wno-missing-field-initializers",
    "-Wno-expansion-to-defined",
    "-Wimplicit-fallthrough=0",
    "--short-enums",
];

static DEFINES: &[(&str, Option<&str>)] = &[
    ("USE_APP_CONFIG", None),
    ("BLE_STACK_SUPPORT_REQD", None),
    ("BOARD_PCA10040", None),
    ("CONFIG_GPIO_AS_PINRESET", None),
    ("NRF52", None),
    ("NRF52832_XXAA", None),
    ("NRF52_PAN_12", None),
    ("NRF52_PAN_15", None),
    ("NRF52_PAN_20", None),
    ("NRF52_PAN_31", None),
    ("NRF52_PAN_36", None),
    ("NRF52_PAN_51", None),
    ("NRF52_PAN_54", None),
    ("NRF52_PAN_55", None),
    ("NRF52_PAN_58", None),
    ("NRF52_PAN_64", None),
    ("NRF52_PAN_74", None),
    ("NRF_SD_BLE_API_VERSION", Some("4")),
    ("S132", None),
    ("SOFTDEVICE_PRESENT", None),
    ("SWI_DISABLE0", None),
];

/// The feature names on the RHS need to be enabled in order for the
/// sources on the LFS to get compiled in.
static SRC_TO_FEAT: &[(&str, &str)] = &[
    ("nRF5-sdk/components/ble/ble_advertising", "BLE_ADVERTISING"),
    ("nRF5-sdk/components/ble/peer_manager", "PEER_MANAGER"),
    ("nRF5-sdk/components/libraries/log", "NRF_LOG"),
    ("nRF5-sdk/components/libraries/crc16", "CRC16"),
    ("nRF5-sdk/components/libraries/button", "BUTTON"),
    ("nRF5-sdk/components/drivers_nrf/uart", "UART"),
    ("nRF5-sdk/components/drivers_nrf/gpiote", "GPIOTE"),
];

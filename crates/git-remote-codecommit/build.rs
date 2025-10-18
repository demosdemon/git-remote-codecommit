// Largely lifted from https://github.com/dtolnay/anyhow/blob/1d7ef1db5414ac155ad6254685673c90ea4c7d77/build.rs
// under the Apache License, Version 2.0 or the MIT License, same as this crate.

//! Probes the compiler to determine whether it supports specific unstable
//! features and enables them if so.
//!
//! This is currently used to check if `bool_to_result` is available, which is
//! very new but replaces the manually implemented `BoolExt` trait.

use std::ffi::OsString;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;

macro_rules! die {
    ($($arg:tt)*) => {{
        eprintln!("ERROR: {}", format_args!($($arg)*));
        std::process::exit(1);
    }};
}

fn main() {
    println!("cargo:rustc-check-cfg=cfg(build_feature_probe)");
    println!("cargo:rustc-check-cfg=cfg(bool_to_result)");
    println!("cargo:rustc-check-cfg=cfg(windows_process_exit_code_from)");
    println!("cargo:rerun-if-changed=src/nightly/mod.rs");
    println!("cargo:rerun-if-changed=src/nightly/bool_or.rs");
    println!("cargo:rerun-if-changed=src/nightly/windows_process_exit_code.rs");

    let CompilerProbeResult {
        supported: bool_to_result,
        consider_rustc_bootstrap,
    } = compiler_probe("bool_or.rs");

    if bool_to_result {
        println!("cargo:rustc-cfg=bool_to_result");
    }

    #[cfg(windows)]
    let consider_rustc_bootstrap = {
        let CompilerProbeResult {
            supported,
            consider_rustc_bootstrap: consider_rustc_bootstrap_windows,
        } = compiler_probe("windows_process_exit_code.rs");
        if supported {
            println!("cargo:rustc-cfg=windows_process_exit_code_from");
        }

        consider_rustc_bootstrap || consider_rustc_bootstrap_windows
    };

    if consider_rustc_bootstrap {
        println!("cargo:rerun-if-env-changed=RUSTC_BOOTSTRAP");
    }
}

struct CompilerProbeResult {
    supported: bool,
    consider_rustc_bootstrap: bool,
}

fn compiler_probe(filename: impl AsRef<Path>) -> CompilerProbeResult {
    if compile_probe(&filename, false) {
        CompilerProbeResult {
            supported: true,
            consider_rustc_bootstrap: false,
        }
    } else if let Some(rustc_bootstrap) = std::env::var_os("RUSTC_BOOTSTRAP") {
        if compile_probe("bool_or.rs", true) {
            CompilerProbeResult {
                supported: true,
                consider_rustc_bootstrap: true,
            }
        } else if rustc_bootstrap == "1" {
            CompilerProbeResult {
                supported: false,
                consider_rustc_bootstrap: false,
            }
        } else {
            CompilerProbeResult {
                supported: false,
                consider_rustc_bootstrap: true,
            }
        }
    } else {
        CompilerProbeResult {
            supported: false,
            consider_rustc_bootstrap: true,
        }
    }
}

fn compile_probe(filename: impl AsRef<Path>, rustc_bootstrap: bool) -> bool {
    if std::env::var_os("RUSTC_STAGE").is_some() {
        println!("cargo:rerun-if-env-changed=RUSTC_STAGE");
        // We are running inside rustc bootstrap. This is a highly non-standard
        // environment with issues such as:
        //
        //     https://github.com/rust-lang/cargo/issues/11138
        //     https://github.com/rust-lang/rust/issues/114839
        //
        // Let's just not use nightly features here.
        return false;
    }

    let probe_dir = ["probe-stable", "probe"][usize::from(rustc_bootstrap)];

    let out_dir = cargo_required_env_var_os("OUT_DIR");
    let out_subdir = Path::new(&out_dir).join(probe_dir);
    mkdir(&out_subdir);

    let probefile = Path::new("src").join("nightly").join(filename);

    let mut cmd = rustc_command();

    if !rustc_bootstrap {
        cmd.env_remove("RUSTC_BOOTSTRAP");
    }

    let stderr = out_subdir.join("probe-stderr");
    let stderr = fs::File::create(&stderr)
        .unwrap_or_else(|err| die!("filed to create stderr file {}: {}", stderr.display(), err));

    cmd.stderr(stderr)
        .arg("--cfg=build_feature_probe")
        .arg("--edition=2024")
        .arg("--crate-name=git_remote_codecommit")
        .arg("--crate-type=lib")
        .arg("--emit=dep-info,metadata")
        .arg("--cap-lints=allow")
        .arg("--out-dir")
        .arg(&out_subdir)
        .arg(probefile);

    if let Some(target) = std::env::var_os("TARGET") {
        cmd.arg("--target").arg(target);
    }

    // If Cargo wants to set RUSTFLAGS, use that.
    if let Ok(rustflags) = std::env::var("CARGO_ENCODED_RUSTFLAGS") {
        for arg in rustflags.split('\x1f').filter(|s| !s.is_empty()) {
            cmd.arg(arg);
        }
    }

    let success = match cmd.status() {
        Ok(status) => status.success(),
        Err(_) => false,
    };

    rmrf(&out_subdir);

    success
}

fn mkdir(path: &Path) {
    match fs::create_dir_all(path) {
        Ok(()) => {}
        Err(err) if err.kind() == ErrorKind::AlreadyExists => {}
        Err(err) => die!("failed to create {}: {err}", path.display()),
    }
}

fn rmrf(path: &Path) {
    match fs::remove_dir_all(path) {
        Ok(()) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => die!("failed to remove {}: {err}", path.display()),
    }
}

fn cargo_required_env_var_os(key: &str) -> OsString {
    std::env::var_os(key).unwrap_or_else(|| {
        die!("Environment variable ${key} is not set during execution of build script")
    })
}

fn rustc_command() -> Command {
    let mut cmd = None;

    if let Some(path) = std::env::var_os("RUSTC_WRAPPER") {
        // this is implicit, but included for debugging
        println!("cargo:rerun-if-env-changed=RUSTC_WRAPPER");
        cmd = Some(Command::new(path));
    }

    if let Some(path) = std::env::var_os("RUSTC_WORKSPACE_WRAPPER") {
        // this is implicit, but included for debugging
        println!("cargo:rerun-if-env-changed=RUSTC_WORKSPACE_WRAPPER");
        if let Some(ref mut cmd) = cmd {
            cmd.arg(path);
        } else {
            cmd = Some(Command::new(path));
        }
    }

    let path = cargo_required_env_var_os("RUSTC");
    if let Some(mut cmd) = cmd {
        cmd.arg(path);
        cmd
    } else {
        Command::new(path)
    }
}

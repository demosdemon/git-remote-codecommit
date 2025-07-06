use std::ffi::OsString;
use std::fmt;
use std::fmt::Write;
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

    let bool_to_result;
    println!("cargo:rerun-if-changed=src/nightly.rs");

    let consider_rustc_bootstrap;
    if compile_probe(false) {
        // This is a nightly or dev compiler, so it supports unstable
        // features regardless of RUSTC_BOOTSTRAP. No need to rerun build
        // script if RUSTC_BOOTSTRAP is changed.
        bool_to_result = true;
        consider_rustc_bootstrap = false;
    } else if let Some(rustc_bootstrap) = std::env::var_os("RUSTC_BOOTSTRAP") {
        if compile_probe(true) {
            // This is a stable or beta compiler for which the user has set
            // RUSTC_BOOTSTRAP to turn on unstable features. Rerun build
            // script if they change it.
            bool_to_result = true;
            consider_rustc_bootstrap = true;
        } else if rustc_bootstrap == "1" {
            // This compiler does not support the generic member access API
            // in the form that anyhow expects. No need to pay attention to
            // RUSTC_BOOTSTRAP.
            bool_to_result = false;
            consider_rustc_bootstrap = false;
        } else {
            // This is a stable or beta compiler for which RUSTC_BOOTSTRAP
            // is set to restrict the use of unstable features by this
            // crate.
            bool_to_result = false;
            consider_rustc_bootstrap = true;
        }
    } else {
        // Without RUSTC_BOOTSTRAP, this compiler does not support the
        // generic member access API in the form that anyhow expects, but
        // try again if the user turns on unstable features.
        bool_to_result = false;
        consider_rustc_bootstrap = true;
    }

    if bool_to_result {
        println!("cargo:rustc-cfg=bool_to_result");
    }

    if consider_rustc_bootstrap {
        println!("cargo:rerun-if-env-changed=RUSTC_BOOTSTRAP");
    }
}

fn compile_probe(rustc_bootstrap: bool) -> bool {
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

    let probefile = Path::new("src").join("nightly.rs");

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

    let debug = rustc_probe_debug();
    if debug {
        let mut msg = String::new();
        write_command(&mut msg, &cmd).expect("write to string does not fail");
        std::fs::write(out_subdir.join("probe-command"), msg)
            .unwrap_or_else(|err| die!("failed to write probe command: {err}"));
    }

    let success = match cmd.status() {
        Ok(status) => status.success(),
        Err(_) => false,
    };

    // Clean up to avoid leaving nondeterministic absolute paths in the dep-info
    // file in OUT_DIR, which causes nonreproducible builds in build systems
    // that treat the entire OUT_DIR as an artifact.
    if !debug {
        rmrf(&out_subdir);
    }

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

fn rustc_probe_debug() -> bool {
    fn is_falsy(mut s: OsString) -> bool {
        if s.len() <= 5 {
            s.make_ascii_lowercase();
            s.is_empty() || s == "0" || s == "false" || s == "no" || s == "off"
        } else {
            // otherwise it's too long to be a falsy value and a waste to lowercase
            false
        }
    }

    println!("cargo:rerun-if-env-changed=RUSTC_PROBE_KEEP_PROBE");
    match std::env::var_os("RUSTC_PROBE_KEEP_PROBE") {
        Some(v) => !is_falsy(v),
        None => false,
    }
}

fn write_command(out: &mut impl Write, cmd: &Command) -> fmt::Result {
    writeln!(out, "Running RUSTC command:")?;
    if let Some(cwd) = cmd.get_current_dir() {
        writeln!(out, "  cwd : {}", cwd.display())?;
    } else if let Ok(cwd) = std::env::current_dir() {
        writeln!(out, "  cwd : {}", cwd.display())?;
    } else {
        writeln!(out, "  cwd : <unknown>")?;
    }

    writeln!(out, "  prog: {}", cmd.get_program().display())?;

    let mut args = cmd.get_args();
    if let Some(arg) = args.next() {
        writeln!(out, "  args: - {}", arg.display())?;
        for arg in args {
            writeln!(out, "        - {}", arg.display())?;
        }
    } else {
        writeln!(out, "  args: <none>")?;
    }

    let mut envs = cmd.get_envs();
    if let Some((env, opt)) = envs.next() {
        if let Some(value) = opt {
            writeln!(out, "  env : {}={}", env.display(), value.display())?;
        } else {
            writeln!(out, "  env : {}=<removed>", env.display())?;
        }

        for (env, opt) in envs {
            if let Some(value) = opt {
                writeln!(out, "        {}={}", env.display(), value.display())?;
            } else {
                writeln!(out, "        {}=<removed>", env.display())?;
            }
        }
    } else {
        writeln!(out, "  env : <none>")?;
    }

    Ok(())
}

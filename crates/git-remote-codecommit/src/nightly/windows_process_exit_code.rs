#![cfg_attr(build_feature_probe, feature(windows_process_exit_code_from))]

#[cfg(build_feature_probe)]
const _: () = {
    use std::process::ExitCode;

    fn _probe() -> std::process::ExitCode {
        std::os::windows::process::ExitCodeExt::from_raw(0_u32)
    }
};

#[cfg(build_feature_probe)]
const _: Option<&str> = option_env!("RUSTC_BOOTSTRAP");

#[cfg(not(windows_process_exit_code_from))]
pub(crate) trait ExitCodeExt {
    fn from_raw(code: u32) -> Self;
}

#[cfg(not(windows_process_exit_code_from))]
impl ExitCodeExt for std::process::ExitCode {
    fn from_raw(code: u32) -> Self {
        if let Ok(code) = u8::try_from(code) {
            std::process::ExitCode::from(code)
        } else {
            std::process::ExitCode::FAILURE
        }
    }
}

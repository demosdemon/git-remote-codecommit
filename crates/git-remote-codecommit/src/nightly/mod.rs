#[cfg(not(build_feature_probe))]
mod windows_process_exit_code;

#[cfg(windows_process_exit_code_from)]
pub(crate) use std::os::windows::process::ExitCodeExt;

#[cfg(not(windows_process_exit_code_from))]
pub(crate) use self::windows_process_exit_code::ExitCodeExt;

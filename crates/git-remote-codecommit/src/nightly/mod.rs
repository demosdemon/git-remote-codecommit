#[cfg(not(build_feature_probe))]
mod bool_or;

#[cfg(all(windows, not(build_feature_probe)))]
mod windows_process_exit_code;

#[cfg(all(windows, windows_process_exit_code_from))]
pub(crate) use std::os::windows::process::ExitCodeExt;

#[cfg(not(bool_to_result))]
pub(crate) use self::bool_or::BoolExt;
#[cfg(all(windows, not(windows_process_exit_code_from)))]
pub(crate) use self::windows_process_exit_code::ExitCodeExt;

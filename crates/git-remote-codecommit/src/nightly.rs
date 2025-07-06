#![cfg_attr(build_feature_probe, feature(bool_to_result))]

#[cfg(build_feature_probe)]
const _: () = {
    enum Void {}
    fn _probe() -> Result<Void, ()> {
        (false).ok_or(())?;
        unreachable!();
    }
};

#[cfg(build_feature_probe)]
const _: Option<&str> = option_env!("RUSTC_BOOTSTRAP");

#[cfg(not(bool_to_result))]
pub(crate) trait BoolExt {
    fn ok_or<E>(self, or: E) -> Result<(), E>;
}

#[cfg(not(bool_to_result))]
impl BoolExt for bool {
    fn ok_or<E>(self, or: E) -> Result<(), E> {
        if self { Ok(()) } else { Err(or) }
    }
}

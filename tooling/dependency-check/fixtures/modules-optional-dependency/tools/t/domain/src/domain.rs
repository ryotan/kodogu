#[cfg(feature = "kernel-helper")]
pub fn rule() -> u8 {
    crate::use_cases::run()
}

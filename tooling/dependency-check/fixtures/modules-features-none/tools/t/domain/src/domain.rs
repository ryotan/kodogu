#[cfg(all(not(feature = "default"), not(feature = "one")))]
pub fn rule() -> u8 {
    crate::use_cases::run()
}

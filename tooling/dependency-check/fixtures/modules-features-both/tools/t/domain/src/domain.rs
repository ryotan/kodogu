#[cfg(all(feature = "default", feature = "one"))]
pub fn rule() -> u8 {
    crate::use_cases::run()
}

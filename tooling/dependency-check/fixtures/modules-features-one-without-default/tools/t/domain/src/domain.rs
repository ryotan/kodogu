#[cfg(all(not(feature = "default"), feature = "one"))]
pub fn rule() -> u8 {
    crate::use_cases::run()
}

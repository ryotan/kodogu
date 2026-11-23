macro_rules! run {
    () => {
        $crate::use_cases::run()
    };
}
pub mod domain;
pub mod use_cases;

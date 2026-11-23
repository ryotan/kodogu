use proc_macro::TokenStream;

/// Replaces the module it is applied to with a module whose code uses the use cases.
#[proc_macro_attribute]
pub fn replace(_: TokenStream, _item: TokenStream) -> TokenStream {
    "pub mod domain { pub fn rule() -> u8 { crate::use_cases::run() } }"
        .parse()
        .expect("the replacement is valid Rust")
}

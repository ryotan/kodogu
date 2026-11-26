// `tauri-build` sets the configuration names `desktop` and `mobile` for the application's crate only.
// This crate uses them too, so it sets them itself, by the rule that `tauri-build` uses.
fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let mobile = target_os == "ios" || target_os == "android";
    println!("cargo:rustc-check-cfg=cfg(desktop)");
    println!("cargo:rustc-check-cfg=cfg(mobile)");
    println!("cargo:rustc-cfg={}", if mobile { "mobile" } else { "desktop" });
}

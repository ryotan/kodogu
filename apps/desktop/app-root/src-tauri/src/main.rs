// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[suzunari_error::report]
fn main() -> Result<(), kodogu_lib::RunError> {
    kodogu_lib::run()?;
    Ok(())
}

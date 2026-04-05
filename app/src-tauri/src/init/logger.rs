use suzunari_error::*;
use tauri::{AppHandle, Runtime};
use time::UtcOffset;
use time::macros::format_description;
use tracing::Level;
use tracing_subscriber::fmt::time::OffsetTime;

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub enum LoggerSetupError {
    #[cfg_attr(not(all(desktop, not(debug_assertions))), allow(dead_code))]
    #[suzu(display("Log directory was not found"))]
    LogDirectoryNotFound { source: tauri::Error },

    #[cfg_attr(not(all(desktop, not(debug_assertions))), allow(dead_code))]
    #[suzu(display("Failed to create the log file"))]
    LogFileCreation { source: std::io::Error },
}

pub(crate) fn setup<R: Runtime>(
    #[cfg_attr(not(all(desktop, not(debug_assertions))), allow(unused_variables))] app: &AppHandle<R>,
    zone_offset: UtcOffset,
) -> Result<(), LoggerSetupError> {
    let fmt = if cfg!(debug_assertions) {
        format_description!("[hour]:[minute]:[second].[subsecond digits:3]")
    } else {
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]")
    };

    #[cfg(all(desktop, not(debug_assertions)))]
    let writer = {
        use std::{fs::File, sync::Mutex};
        use tauri::Manager;
        let log_file = File::create(
            app.path().app_log_dir().context(LogDirectoryNotFoundError)?.join("kodogu.log"),
        )
        .context(LogFileCreationError)?;
        Mutex::new(log_file)
    };

    #[cfg(not(all(desktop, not(debug_assertions))))]
    let writer = std::io::stderr;

    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter("kodogu_lib")
        .with_target(false)
        .with_timer(OffsetTime::new(zone_offset, fmt))
        .with_writer(writer)
        .with_ansi(cfg!(not(all(desktop, not(debug_assertions)))))
        .init();

    Ok(())
}

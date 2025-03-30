use tauri::AppHandle;
use time::macros::{format_description, offset};
use tracing::Level;
use tracing_subscriber::fmt::time::OffsetTime;

#[derive(Debug, thiserror::Error)]
pub(crate) enum LoggerSetupError {
    #[cfg_attr(not(all(desktop, not(debug_assertions))), allow(dead_code))]
    #[error("Log directory was not found")]
    LogDirectoryNotFoundError(tauri::Error),

    #[cfg_attr(not(all(desktop, not(debug_assertions))), allow(dead_code))]
    #[error("Failed to create the log file")]
    LogFileCreationError(std::io::Error),
}

pub(crate) fn setup(
    #[cfg_attr(not(all(desktop, not(debug_assertions))), allow(unused_variables))] app: &AppHandle,
) -> Result<(), LoggerSetupError> {
    let fmt = if cfg!(debug_assertions) {
        format_description!("[hour]:[minute]:[second].[subsecond digits:3]")
    } else {
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]")
    };

    #[cfg(all(desktop, not(debug_assertions)))]
    let writer = {
        use crate::init::logger::LoggerSetupError::{
            LogDirectoryNotFoundError, LogFileCreationError,
        };
        use std::{fs::File, sync::Mutex};
        use tauri::Manager;
        let log_file = File::create(
            app.path()
                .app_log_dir()
                .map_err(LogDirectoryNotFoundError)?
                .join("kodogu.log"),
        )
        .map_err(LogFileCreationError)?;
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
        .with_timer(OffsetTime::new(offset!(+9), fmt))
        .with_writer(writer)
        .with_ansi(cfg!(not(all(desktop, not(debug_assertions)))))
        .init();

    Ok(())
}

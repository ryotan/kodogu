use snafu::{OptionExt, ResultExt};
use suzunari_error::*;
use tauri::AppHandle;
use tauri::menu::{Menu, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

#[suzunari_error]
#[snafu(context(suffix(Error)))]
pub(crate) enum TraySetupError {
    #[snafu(display("Failed to build tray icon"))]
    Build { source: tauri::Error },
    #[snafu(display("Default window icon is not configured in tauri.conf.json"))]
    IconNotConfigured,
}

pub(crate) fn setup(app: &AppHandle) -> Result<(), TraySetupError> {
    let icon = app.default_window_icon().context(IconNotConfiguredError)?.clone();

    let quit = PredefinedMenuItem::quit(app, Some("Quit KoDoGu")).context(BuildError)?;
    let menu = Menu::with_items(app, &[&quit]).context(BuildError)?;

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(true)
        .tooltip("KoDoGu")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray_handle, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                super::popover::toggle(tray_handle.app_handle(), rect);
            }
        })
        .build(app)
        .context(BuildError)?;
    Ok(())
}

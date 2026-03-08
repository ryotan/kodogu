use suzunari_error::*;
use tauri::AppHandle;
use tauri::image::Image;
use tauri::menu::{Menu, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub(crate) enum TraySetupError {
    #[suzu(display("Failed to load tray icon resource"))]
    IconLoad { source: tauri::Error },
    #[suzu(display("Failed to add quit menu item"))]
    AddQuitMenu { source: tauri::Error },
    #[suzu(display("Failed to add menu item"))]
    AddMenu { source: tauri::Error },
    #[suzu(display("Failed to build tray icon"))]
    BuildIcon { source: tauri::Error },
}

pub(crate) fn setup(app: &AppHandle) -> Result<TrayIcon, TraySetupError> {
    let icon =
        Image::from_bytes(include_bytes!("../../icons/tray-icon.png")).context(IconLoadError)?;

    let quit = PredefinedMenuItem::quit(app, Some("Quit KoDoGu")).context(AddQuitMenuError)?;
    let menu = Menu::with_items(app, &[&quit]).context(AddMenuError)?;

    TrayIconBuilder::new()
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
        .context(BuildIconError)
}

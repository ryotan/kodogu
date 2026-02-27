use tauri::App;
use tauri::menu::{Menu, PredefinedMenuItem, Submenu};

pub(crate) fn setup(app: &mut App) -> Result<(), tauri::Error> {
    let handle = app.handle();

    let edit_menu = Submenu::with_items(
        handle,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(handle, None)?,
            &PredefinedMenuItem::redo(handle, None)?,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::cut(handle, None)?,
            &PredefinedMenuItem::copy(handle, None)?,
            &PredefinedMenuItem::paste(handle, None)?,
            &PredefinedMenuItem::select_all(handle, None)?,
        ],
    )?;

    app.set_menu(Menu::with_items(handle, &[&edit_menu])?)?;

    Ok(())
}

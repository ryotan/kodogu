use suzunari_error::*;
use tauri::menu::{Menu, PredefinedMenuItem, Submenu};
use tauri::{App, Runtime};

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub enum MenuSetupError {
    #[suzu(display("Failed to build predefined menu item: {item}"))]
    BuildPredefinedMenuItem { item: &'static str, source: tauri::Error },
    #[suzu(display("Failed to build edit menu"))]
    BuildEditMenu { source: tauri::Error },
    #[suzu(display("Failed to build menu"))]
    BuildMenu { source: tauri::Error },
    #[suzu(display("Failed to set menu"))]
    SetMenu { source: tauri::Error },
}

pub(crate) fn setup<R: Runtime>(app: &mut App<R>) -> Result<Option<Menu<R>>, MenuSetupError> {
    let handle = app.handle();

    let edit_menu = Submenu::with_items(
        handle,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(handle, None)
                .context(BuildPredefinedMenuItemError { item: "undo" })?,
            &PredefinedMenuItem::redo(handle, None)
                .context(BuildPredefinedMenuItemError { item: "redo" })?,
            &PredefinedMenuItem::separator(handle)
                .context(BuildPredefinedMenuItemError { item: "separator" })?,
            &PredefinedMenuItem::cut(handle, None)
                .context(BuildPredefinedMenuItemError { item: "cut" })?,
            &PredefinedMenuItem::copy(handle, None)
                .context(BuildPredefinedMenuItemError { item: "copy" })?,
            &PredefinedMenuItem::paste(handle, None)
                .context(BuildPredefinedMenuItemError { item: "paste" })?,
            &PredefinedMenuItem::select_all(handle, None)
                .context(BuildPredefinedMenuItemError { item: "select_all" })?,
        ],
    )
    .context(BuildEditMenuError)?;

    app.set_menu(Menu::with_items(handle, &[&edit_menu]).context(BuildMenuError)?)
        .context(SetMenuError)
}

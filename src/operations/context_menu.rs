use gtk4::prelude::*;
use gtk4::PopoverMenu;
use std::path::PathBuf;

/// Create a context menu popover for the given file path.
/// Returns the popover; the caller connects action handlers via
/// `app.set_accels_for_action()` and `gio::SimpleActionGroup`.
pub fn create_context_menu(path: &PathBuf) -> PopoverMenu {
    let menu = gio::Menu::new();
    menu.append(Some("Open"), Some("ctx.open"));
    menu.append(Some("Get Info"), Some("ctx.info"));
    menu.append(Some("Rename"), Some("ctx.rename"));
    menu.append(Some("Move to Trash"), Some("ctx.trash"));

    let popover = PopoverMenu::from_model(Some(&menu));

    // Store the path as the widget name for later retrieval
    let path_str = path.to_string_lossy().to_string();
    popover.set_widget_name(&path_str);

    popover
}

/// Get the file path associated with a context menu popover.
pub fn get_context_menu_path(popover: &PopoverMenu) -> Option<PathBuf> {
    let name = popover.widget_name();
    if name.is_empty() {
        None
    } else {
        Some(PathBuf::from(name.as_str()))
    }
}

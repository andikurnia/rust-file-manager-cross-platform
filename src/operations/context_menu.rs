use gtk4::PopoverMenu;

pub fn create_context_menu() -> PopoverMenu {
    let menu = gio::Menu::new();
    menu.append(Some("Open"), Some("app.open"));
    menu.append(Some("Get Info"), Some("app.info"));
    menu.append(Some("Rename"), Some("app.rename"));
    menu.append(Some("Move to Trash"), Some("app.trash"));

    PopoverMenu::from_model(Some(&menu))
}

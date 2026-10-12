use gtk4::prelude::*;
use gtk4::{Box, Orientation, ListBox, ListBoxRow, Label, Image};

pub struct Sidebar {
    pub widget: Box,
    pub list: ListBox,
}

impl Sidebar {
    pub fn new() -> Self {
        let widget = Box::new(Orientation::Vertical, 0);
        widget.add_css_class("sidebar");

        let list = ListBox::new();
        list.set_selection_mode(gtk4::SelectionMode::None);

        let places = [
            ("user-home-symbolic", "Home", "home"),
            ("user-desktop-symbolic", "Desktop", "desktop"),
            ("folder-documents-symbolic", "Documents", "documents"),
            ("folder-download-symbolic", "Downloads", "downloads"),
            ("user-trash-symbolic", "Trash", "trash"),
        ];

        for (icon, label, _key) in places {
            let row = ListBoxRow::new();
            let hbox = Box::new(Orientation::Horizontal, 8);
            let image = Image::from_icon_name(icon);
            image.set_pixel_size(16);
            let label_widget = Label::new(Some(label));
            label_widget.set_halign(gtk4::Align::Start);
            hbox.append(&image);
            hbox.append(&label_widget);
            row.set_child(Some(&hbox));
            list.append(&row);
        }

        widget.append(&list);
        Self { widget, list }
    }
}

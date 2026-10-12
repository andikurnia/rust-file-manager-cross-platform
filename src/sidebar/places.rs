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

        // Favorites section header
        let fav_header = Label::new(Some("Favorites"));
        fav_header.add_css_class("sidebar-section-header");
        fav_header.set_halign(gtk4::Align::Start);
        fav_header.set_margin_top(8);
        fav_header.set_margin_bottom(4);
        fav_header.set_margin_start(12);
        widget.append(&fav_header);

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
            row.add_css_class("sidebar-row");
            let hbox = Box::new(Orientation::Horizontal, 8);
            hbox.set_margin_top(2);
            hbox.set_margin_bottom(2);
            hbox.set_margin_start(8);
            hbox.set_margin_end(8);

            let image = Image::from_icon_name(icon);
            image.set_pixel_size(18);

            let label_widget = Label::new(Some(label));
            label_widget.set_halign(gtk4::Align::Start);
            label_widget.set_hexpand(true);

            hbox.append(&image);
            hbox.append(&label_widget);
            row.set_child(Some(&hbox));
            list.append(&row);
        }

        widget.append(&list);

        // Devices section header
        let devices_header = Label::new(Some("Devices"));
        devices_header.add_css_class("sidebar-section-header");
        devices_header.set_halign(gtk4::Align::Start);
        devices_header.set_margin_top(16);
        devices_header.set_margin_bottom(4);
        devices_header.set_margin_start(12);
        widget.append(&devices_header);

        // Add a placeholder for devices
        let devices_placeholder = Label::new(Some("No devices mounted"));
        devices_placeholder.add_css_class("sidebar-placeholder");
        devices_placeholder.set_halign(gtk4::Align::Start);
        devices_placeholder.set_margin_start(12);
        devices_placeholder.set_margin_bottom(8);
        widget.append(&devices_placeholder);

        Self { widget, list }
    }
}

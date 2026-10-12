use gtk4::prelude::*;
use gtk4::{Box, Orientation, Image, Label, FlowBox, FlowBoxChild};
use crate::models::file_item::FileItem;

pub struct IconView {
    pub widget: FlowBox,
}

impl IconView {
    pub fn new() -> Self {
        let widget = FlowBox::new();
        widget.set_selection_mode(gtk4::SelectionMode::Multiple);
        widget.set_homogeneous(true);
        widget.set_max_children_per_line(16);
        widget.set_min_children_per_line(2);
        widget.set_column_spacing(2);
        widget.set_row_spacing(2);
        widget.set_margin_top(4);
        widget.set_margin_bottom(4);
        widget.set_margin_start(4);
        widget.set_margin_end(4);
        widget.add_css_class("iconview");

        Self { widget }
    }

    pub fn populate(&self, items: &[FileItem]) {
        while let Some(child) = self.widget.first_child() {
            self.widget.remove(&child);
        }

        for item in items {
            let child = self.create_item_widget(item);
            self.widget.append(&child);
        }
    }

    fn create_item_widget(&self, item: &FileItem) -> FlowBoxChild {
        let container = FlowBoxChild::new();
        container.add_css_class("icon-item");

        let vbox = Box::new(Orientation::Vertical, 2);
        vbox.set_halign(gtk4::Align::Center);
        vbox.set_valign(gtk4::Align::Start);

        // Icon or thumbnail
        let image = if item.is_dir {
            let img = Image::from_icon_name("folder");
            img.set_pixel_size(64);
            img
        } else if item.mime_type.starts_with("image/") {
            // Try to load a thumbnail for images
            let file = gio::File::for_path(&item.path);
            match gtk4::gdk::Texture::from_file(&file) {
                Ok(texture) => {
                    let img = Image::from_paintable(Some(&texture));
                    img.set_pixel_size(64);
                    img
                }
                Err(_) => {
                    let img = Image::from_icon_name(&item.icon_name());
                    img.set_pixel_size(64);
                    img
                }
            }
        } else {
            let img = Image::from_icon_name(&item.icon_name());
            img.set_pixel_size(64);
            img
        };

        // Label with better styling
        let label = Label::new(Some(&item.name));
        label.set_max_width_chars(14);
        label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        label.set_justify(gtk4::Justification::Center);
        label.set_wrap(true);
        label.set_lines(2);

        vbox.append(&image);
        vbox.append(&label);
        container.set_child(Some(&vbox));
        container
    }
}

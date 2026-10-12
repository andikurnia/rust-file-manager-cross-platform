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
        widget.set_max_children_per_line(10);
        widget.set_min_children_per_line(2);
        widget.set_column_spacing(16);
        widget.set_row_spacing(16);
        widget.set_margin_top(12);
        widget.set_margin_bottom(12);
        widget.set_margin_start(12);
        widget.set_margin_end(12);

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
        let vbox = Box::new(Orientation::Vertical, 4);
        vbox.set_halign(gtk4::Align::Center);

        let image = Image::from_icon_name(&item.icon_name());
        image.set_pixel_size(48);

        let label = Label::new(Some(&item.name));
        label.set_max_width_chars(12);
        label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        label.set_justify(gtk4::Justification::Center);

        vbox.append(&image);
        vbox.append(&label);
        container.set_child(Some(&vbox));
        container
    }
}

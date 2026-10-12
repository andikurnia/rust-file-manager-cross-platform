use gtk4::prelude::*;
use gtk4::{DragSource, DropTarget};
use gtk4::gdk::{DragAction, ContentProvider};
use std::path::PathBuf;

pub fn setup_drag_source(widget: &impl IsA<gtk4::Widget>, paths: &[PathBuf]) {
    let drag_source = DragSource::builder()
        .actions(DragAction::COPY | DragAction::MOVE)
        .build();

    let paths_clone = paths.to_vec();
    drag_source.connect_prepare(move |_drag, _x, _y| {
        if paths_clone.is_empty() {
            return None;
        }
        let uris: Vec<String> = paths_clone
            .iter()
            .filter_map(|p| {
                let uri = gio::File::for_path(p).uri();
                Some(uri.to_string())
            })
            .collect();
        let bytes = glib::Bytes::from(uris.join("\n").as_bytes());
        let provider = ContentProvider::for_bytes("text/uri-list", &bytes);
        Some(provider)
    });

    widget.add_controller(drag_source);
}

pub fn setup_drop_target(widget: &impl IsA<gtk4::Widget>, on_drop: Box<dyn Fn(Vec<PathBuf>)>) {
    let drop_target = DropTarget::new(gio::File::static_type(), DragAction::COPY | DragAction::MOVE);

    drop_target.connect_drop(move |_drop, value, _x, _y| {
        if let Ok(uris) = value.get::<Vec<String>>() {
            let paths: Vec<PathBuf> = uris
                .into_iter()
                .filter_map(|uri| gio::File::for_uri(&uri).path())
                .collect();
            on_drop(paths);
            return true;
        }
        false
    });

    widget.add_controller(drop_target);
}

use gtk4::prelude::*;
use gtk4::{
    ApplicationWindow, HeaderBar, Button, Box as GtkBox, Orientation,
    Entry, ScrolledWindow, ColumnView, ColumnViewColumn,
    SignalListItemFactory, SingleSelection, StringList, StringObject,
    Paned, Image, Label, ListItem, Stack, SearchEntry,
    EventControllerKey, FileChooserNative, DialogFlags,
};
use crate::app::NavigationState;
use crate::settings::ViewMode;
use crate::models::file_item::FileItem;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use gtk4::gdk;

/// A single column in the Miller column view.
struct MillerColumn {
    pub path: PathBuf,
    pub list: StringList,
    pub selection: SingleSelection,
    pub view: ColumnView,
}

pub struct MainWindow {
    pub window: ApplicationWindow,
    pub nav: Arc<Mutex<NavigationState>>,
    pub path_entry: Entry,
    pub back_button: Button,
    pub forward_button: Button,
    pub column_view: ColumnView,
    pub string_list: StringList,
    pub miller_columns: Arc<Mutex<Vec<MillerColumn>>>,
    pub miller_scroll: ScrolledWindow,
    pub miller_box: GtkBox,
    pub stack: Stack,
    pub icon_view: crate::views::icon_view::IconView,
    pub search_entry: SearchEntry,
    pub all_items: Arc<Mutex<Vec<FileItem>>>,
    pub file_monitor: Arc<Mutex<Option<gio::FileMonitor>>>,
}

impl MainWindow {
    pub fn new(app: &gtk4::Application, initial_path: PathBuf) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Finder")
            .default_width(1100)
            .default_height(650)
            .build();

        // ── Header bar ──────────────────────────────────────────────
        let header = HeaderBar::new();
        window.set_titlebar(Some(&header));

        let back_button = Button::from_icon_name("go-previous-symbolic");
        back_button.set_tooltip_text(Some("Back"));
        header.pack_start(&back_button);

        let forward_button = Button::from_icon_name("go-next-symbolic");
        forward_button.set_tooltip_text(Some("Forward"));
        header.pack_start(&forward_button);

        let path_entry = Entry::builder()
            .placeholder_text("Path")
            .hexpand(true)
            .build();
        header.pack_start(&path_entry);

        // Search entry
        let search_entry = SearchEntry::builder()
            .placeholder_text("Search")
            .hexpand(true)
            .build();
        header.pack_start(&search_entry);

        // View switcher buttons (list / columns / icons)
        let list_btn = Button::from_icon_name("view-list-symbolic");
        list_btn.set_tooltip_text(Some("List View"));
        header.pack_end(&list_btn);

        let columns_btn = Button::from_icon_name("view-column-symbolic");
        columns_btn.set_tooltip_text(Some("Column View"));
        header.pack_end(&columns_btn);

        let icons_btn = Button::from_icon_name("view-grid-symbolic");
        icons_btn.set_tooltip_text(Some("Icon View"));
        header.pack_end(&icons_btn);

        // ── Navigation state ────────────────────────────────────────
        let nav = Arc::new(Mutex::new(NavigationState::new(initial_path.clone())));
        path_entry.set_text(&initial_path.to_string_lossy());

        // ── Main list view (ColumnView with Name/Date/Size) ─────────
        let string_list = StringList::new(&[]);
        let selection = SingleSelection::new(Some(string_list.clone()));
        let column_view = ColumnView::new(Some(selection.clone()));
        column_view.set_vexpand(true);

        let list_scroll = ScrolledWindow::new();
        list_scroll.set_child(Some(&column_view));
        list_scroll.set_vexpand(true);

        // ── Miller column view ──────────────────────────────────────
        let miller_box = GtkBox::new(Orientation::Horizontal, 0);
        let miller_scroll = ScrolledWindow::new();
        miller_scroll.set_child(Some(&miller_box));
        miller_scroll.set_vexpand(true);
        miller_scroll.set_hexpand(true);

        // ── Icon view ───────────────────────────────────────────────
        let icon_scroll = ScrolledWindow::new();
        let icon_view = crate::views::icon_view::IconView::new();
        icon_scroll.set_child(Some(&icon_view.widget));
        icon_scroll.set_vexpand(true);

        // ── Stack to switch between views ───────────────────────────
        let stack = Stack::new();
        stack.add_titled(&list_scroll, Some("list"), "List");
        stack.add_titled(&miller_scroll, Some("columns"), "Columns");
        stack.add_titled(&icon_scroll, Some("icon"), "Icon");
        stack.set_visible_child_name("columns");

        // Wire view-switcher buttons
        list_btn.connect_clicked(glib::clone!(
            #[weak]
            stack,
            move |_| stack.set_visible_child_name("list")
        ));
        columns_btn.connect_clicked(glib::clone!(
            #[weak]
            stack,
            move |_| stack.set_visible_child_name("columns")
        ));
        icons_btn.connect_clicked(glib::clone!(
            #[weak]
            stack,
            move |_| stack.set_visible_child_name("icon")
        ));

        // ── Sidebar + content split ─────────────────────────────────
        let sidebar = crate::sidebar::places::Sidebar::new();
        let paned = Paned::new(Orientation::Horizontal);
        paned.set_start_child(Some(&sidebar.widget));
        paned.set_end_child(Some(&stack));
        paned.set_position(200);

        window.set_child(Some(&paned));

        let miller_columns = Arc::new(Mutex::new(Vec::new()));
        let all_items = Arc::new(Mutex::new(Vec::<FileItem>::new()));
        let file_monitor = Arc::new(Mutex::new(None));

        // Wire search entry — filters files by name in real-time
        {
            let all_items_search = all_items.clone();
            let string_list_search = string_list.clone();
            search_entry.connect_search_changed(move |entry| {
                let query = entry.text().to_lowercase();
                let items = all_items_search.lock().unwrap();
                let filtered: Vec<String> = if query.is_empty() {
                    items.iter().map(|i| i.name.clone()).collect()
                } else {
                    items.iter()
                        .filter(|i| i.name.to_lowercase().contains(&query))
                        .map(|i| i.name.clone())
                        .collect()
                };
                let n = string_list_search.n_items();
                for i in (0..n).rev() {
                    string_list_search.remove(i);
                }
                for name in &filtered {
                    string_list_search.append(name);
                }
            });
        }

        // Store icon_view widget reference for later use
        let icon_widget = icon_view.widget.clone();

        let win = Self {
            window,
            nav: nav.clone(),
            path_entry: path_entry.clone(),
            back_button: back_button.clone(),
            forward_button: forward_button.clone(),
            column_view: column_view.clone(),
            string_list: string_list.clone(),
            miller_columns: miller_columns.clone(),
            miller_scroll,
            miller_box: miller_box.clone(),
            stack: stack.clone(),
            icon_view: crate::views::icon_view::IconView { widget: icon_widget },
            search_entry: search_entry.clone(),
            all_items: all_items.clone(),
            file_monitor: file_monitor.clone(),
        };

        win.setup_list_columns();
        win.wire_nav_buttons();
        win.wire_path_entry();
        win.wire_sidebar(&sidebar);
        win.wire_keyboard_shortcuts();
        win.wire_context_menu();
        win.add_miller_column(&initial_path);
        win.update_nav_buttons();
        win
    }

    pub fn set_view_mode(&self, mode: &str) {
        self.stack.set_visible_child_name(mode);
    }

    // ── List view columns (Name, Date, Size) ───────────────────────

    pub fn setup_list_columns(&self) {
        // Name column
        let name_factory = SignalListItemFactory::new();
        name_factory.connect_setup(|_, item| {
            let list_item = item.clone().downcast::<ListItem>().unwrap();
            let row = GtkBox::new(Orientation::Horizontal, 8);
            let icon = Image::from_icon_name("text-x-generic");
            icon.set_pixel_size(16);
            let label = Label::new(None);
            label.set_halign(gtk4::Align::Start);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            row.append(&icon);
            row.append(&label);
            list_item.set_child(Some(&row));
        });
        let all_items_for_name = self.all_items.clone();
        name_factory.connect_bind(move |_, item| {
            let list_item = item.clone().downcast::<ListItem>().unwrap();
            let row = list_item.child().and_downcast::<GtkBox>().unwrap();
            let icon = row.first_child().and_then(|w| w.downcast::<Image>().ok()).unwrap();
            let label = row.last_child().and_then(|w| w.downcast::<Label>().ok()).unwrap();
            let obj = list_item.item().and_downcast::<StringObject>().unwrap();
            let name = obj.string();
            label.set_text(&name);
            let items = all_items_for_name.lock().unwrap();
            if let Some(file_item) = items.iter().find(|i| i.name == name) {
                icon.set_icon_name(Some(&file_item.icon_name()));
            } else {
                icon.set_icon_name(Some("text-x-generic"));
            }
        });
        let name_col = ColumnViewColumn::new(Some("Name"), Some(name_factory));
        name_col.set_expand(true);
        self.column_view.append_column(&name_col);

        // Date column
        let date_factory = SignalListItemFactory::new();
        date_factory.connect_setup(|_, item| {
            let list_item = item.clone().downcast::<ListItem>().unwrap();
            let label = Label::new(None);
            label.set_halign(gtk4::Align::Start);
            list_item.set_child(Some(&label));
        });
        let all_items_for_date = self.all_items.clone();
        date_factory.connect_bind(move |_, item| {
            let list_item = item.clone().downcast::<ListItem>().unwrap();
            let label = list_item.child().and_downcast::<Label>().unwrap();
            let obj = list_item.item().and_downcast::<StringObject>().unwrap();
            let name = obj.string();
            let items = all_items_for_date.lock().unwrap();
            if let Some(file_item) = items.iter().find(|i| i.name == name) {
                if file_item.modified > 0 {
                    let dt = glib::DateTime::from_unix_local(file_item.modified).ok();
                    if let Some(dt) = dt {
                        label.set_text(&dt.format("%Y-%m-%d %H:%M").unwrap_or_default());
                    } else {
                        label.set_text("—");
                    }
                } else {
                    label.set_text("—");
                }
            } else {
                label.set_text("—");
            }
        });
        let date_col = ColumnViewColumn::new(Some("Date"), Some(date_factory));
        self.column_view.append_column(&date_col);

        // Size column
        let size_factory = SignalListItemFactory::new();
        size_factory.connect_setup(|_, item| {
            let list_item = item.clone().downcast::<ListItem>().unwrap();
            let label = Label::new(None);
            label.set_halign(gtk4::Align::End);
            list_item.set_child(Some(&label));
        });
        let all_items_for_size = self.all_items.clone();
        size_factory.connect_bind(move |_, item| {
            let list_item = item.clone().downcast::<ListItem>().unwrap();
            let label = list_item.child().and_downcast::<Label>().unwrap();
            let obj = list_item.item().and_downcast::<StringObject>().unwrap();
            let name = obj.string();
            let items = all_items_for_size.lock().unwrap();
            if let Some(file_item) = items.iter().find(|i| i.name == name) {
                if file_item.is_dir {
                    label.set_text("—");
                } else {
                    label.set_text(&format_size(file_item.size));
                }
            } else {
                label.set_text("—");
            }
        });
        let size_col = ColumnViewColumn::new(Some("Size"), Some(size_factory));
        self.column_view.append_column(&size_col);
    }

    // ── Miller column view ─────────────────────────────────────────

    fn add_miller_column(&self, path: &std::path::Path) {
        {
            let mut cols = self.miller_columns.lock().unwrap();
            if let Some(idx) = cols.iter().position(|c| c.path == path) {
                while cols.len() > idx + 1 {
                    if let Some(col) = cols.pop() {
                        self.miller_box.remove(&col.view);
                    }
                }
                return;
            }
        }

        let list = StringList::new(&[]);
        let sel = SingleSelection::new(Some(list.clone()));
        let view = ColumnView::new(Some(sel.clone()));
        view.set_vexpand(true);
        view.set_size_request(200, -1);

        let factory = SignalListItemFactory::new();
        factory.connect_setup(|_, item| {
            let list_item = item.clone().downcast::<ListItem>().unwrap();
            let row = GtkBox::new(Orientation::Horizontal, 6);
            let icon = Image::from_icon_name("text-x-generic");
            icon.set_pixel_size(14);
            let label = Label::new(None);
            label.set_halign(gtk4::Align::Start);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            row.append(&icon);
            row.append(&label);
            list_item.set_child(Some(&row));
        });
        factory.connect_bind(|_, item| {
            let list_item = item.clone().downcast::<ListItem>().unwrap();
            let row = list_item.child().and_downcast::<GtkBox>().unwrap();
            let icon = row.first_child().and_then(|w| w.downcast::<Image>().ok()).unwrap();
            let label = row.last_child().and_then(|w| w.downcast::<Label>().ok()).unwrap();
            let obj = list_item.item().and_downcast::<StringObject>().unwrap();
            let name = obj.string().to_string();
            label.set_text(&name);
            let item_path = PathBuf::from(&name);
            let is_dir = item_path.is_dir();
            icon.set_icon_name(Some(if is_dir { "folder" } else { "text-x-generic" }));
        });
        let col = ColumnViewColumn::new(Some("Name"), Some(factory));
        col.set_expand(true);
        view.append_column(&col);

        let path_owned = path.to_path_buf();
        glib::spawn_future_local(glib::clone!(
            #[weak]
            list,
            async move {
                match crate::models::file_item::list_directory(&path_owned).await {
                    Ok(items) => {
                        let mut items = items;
                        items.sort_by(|a, b| {
                            match (a.is_dir, b.is_dir) {
                                (true, false) => std::cmp::Ordering::Less,
                                (false, true) => std::cmp::Ordering::Greater,
                                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                            }
                        });
                        let n = list.n_items();
                        for i in (0..n).rev() {
                            list.remove(i);
                        }
                        for item in &items {
                            list.append(&item.name);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error listing {}: {}", path_owned.display(), e);
                    }
                }
            }
        ));

        let path_for_nav = path.to_path_buf();
        let miller_cols = self.miller_columns.clone();
        let miller_box = self.miller_box.clone();
        let nav = self.nav.clone();
        let path_entry = self.path_entry.clone();
        sel.connect_notify_local(Some("selected"), move |sel, _| {
            if let Some(selected_obj) = sel.selected_item() {
                let name = selected_obj.downcast::<StringObject>().unwrap().string();
                let child_path = path_for_nav.join(&name);
                if child_path.is_dir() {
                    {
                        let mut cols = miller_cols.lock().unwrap();
                        if let Some(idx) = cols.iter().position(|c| c.path == path_for_nav) {
                            while cols.len() > idx + 1 {
                                if let Some(col) = cols.pop() {
                                    miller_box.remove(&col.view);
                                }
                            }
                        }
                    }
                    let list = StringList::new(&[]);
                    let new_sel = SingleSelection::new(Some(list.clone()));
                    let new_view = ColumnView::new(Some(new_sel.clone()));
                    new_view.set_vexpand(true);
                    new_view.set_size_request(200, -1);

                    let factory = SignalListItemFactory::new();
                    factory.connect_setup(|_, item| {
                        let list_item = item.clone().downcast::<ListItem>().unwrap();
                        let row = GtkBox::new(Orientation::Horizontal, 6);
                        let icon = Image::from_icon_name("text-x-generic");
                        icon.set_pixel_size(14);
                        let label = Label::new(None);
                        label.set_halign(gtk4::Align::Start);
                        label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
                        row.append(&icon);
                        row.append(&label);
                        list_item.set_child(Some(&row));
                    });
                    factory.connect_bind(|_, item| {
                        let list_item = item.clone().downcast::<ListItem>().unwrap();
                        let row = list_item.child().and_downcast::<GtkBox>().unwrap();
                        let icon = row.first_child().and_then(|w| w.downcast::<Image>().ok()).unwrap();
                        let label = row.last_child().and_then(|w| w.downcast::<Label>().ok()).unwrap();
                        let obj = list_item.item().and_downcast::<StringObject>().unwrap();
                        let name = obj.string().to_string();
                        label.set_text(&name);
                        let item_path = PathBuf::from(&name);
                        let is_dir = item_path.is_dir();
                        icon.set_icon_name(Some(if is_dir { "folder" } else { "text-x-generic" }));
                    });
                    let col = ColumnViewColumn::new(Some("Name"), Some(factory.clone()));
                    col.set_expand(true);
                    new_view.append_column(&col);

                    let child_path_clone = child_path.clone();
                    glib::spawn_future_local(glib::clone!(
                        #[weak]
                        list,
                        async move {
                            match crate::models::file_item::list_directory(&child_path_clone).await {
                                Ok(items) => {
                                    let mut items = items;
                                    items.sort_by(|a, b| {
                                        match (a.is_dir, b.is_dir) {
                                            (true, false) => std::cmp::Ordering::Less,
                                            (false, true) => std::cmp::Ordering::Greater,
                                            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                                        }
                                    });
                                    let n = list.n_items();
                                    for i in (0..n).rev() {
                                        list.remove(i);
                                    }
                                    for item in &items {
                                        list.append(&item.name);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error listing {}: {}", child_path_clone.display(), e);
                                }
                            }
                        }
                    ));

                    miller_box.append(&new_view);
                    miller_cols.lock().unwrap().push(MillerColumn {
                        path: child_path.clone(),
                        list,
                        selection: new_sel,
                        view: new_view,
                    });

                    nav.lock().unwrap().navigate_to(child_path.clone());
                    path_entry.set_text(&child_path.to_string_lossy());
                }
            }
        });

        self.miller_box.append(&view);
        self.miller_columns.lock().unwrap().push(MillerColumn {
            path: path.to_path_buf(),
            list,
            selection: sel,
            view,
        });
    }

    // ── Navigation wiring ──────────────────────────────────────────

    fn wire_nav_buttons(&self) {
        let nav = self.nav.clone();
        let path_entry = self.path_entry.clone();
        let miller_columns = self.miller_columns.clone();
        let miller_box = self.miller_box.clone();
        let back_button = self.back_button.clone();

        back_button.connect_clicked(move |_| {
            if let Some(prev) = nav.lock().unwrap().go_back() {
                path_entry.set_text(&prev.to_string_lossy());
                {
                    let mut cols = miller_columns.lock().unwrap();
                    while let Some(col) = cols.pop() {
                        miller_box.remove(&col.view);
                    }
                }
                // Re-add the column for the previous path
                let prev_clone = prev.clone();
                // We need to add the Miller column back
                // This is handled by the caller via add_miller_column
                let _ = prev_clone;
            }
        });

        let nav = self.nav.clone();
        let path_entry = self.path_entry.clone();
        let miller_columns = self.miller_columns.clone();
        let miller_box = self.miller_box.clone();
        let forward_button = self.forward_button.clone();

        forward_button.connect_clicked(move |_| {
            if let Some(next) = nav.lock().unwrap().go_forward() {
                path_entry.set_text(&next.to_string_lossy());
                {
                    let mut cols = miller_columns.lock().unwrap();
                    while let Some(col) = cols.pop() {
                        miller_box.remove(&col.view);
                    }
                }
            }
        });
    }

    fn wire_path_entry(&self) {
        let nav = self.nav.clone();
        let path_entry = self.path_entry.clone();
        let miller_columns = self.miller_columns.clone();
        let miller_box = self.miller_box.clone();

        path_entry.connect_activate(move |entry| {
            let text = entry.text();
            let path = PathBuf::from(&text);
            if path.is_dir() {
                nav.lock().unwrap().navigate_to(path.clone());
                {
                    let mut cols = miller_columns.lock().unwrap();
                    while let Some(col) = cols.pop() {
                        miller_box.remove(&col.view);
                    }
                }
            }
        });
    }

    fn wire_sidebar(&self, sidebar: &crate::sidebar::places::Sidebar) {
        let nav = self.nav.clone();
        let path_entry = self.path_entry.clone();
        let miller_columns = self.miller_columns.clone();
        let miller_box = self.miller_box.clone();

        sidebar.list.connect_row_activated(move |_list, row| {
            let index = row.index();
            let path = match index {
                0 => std::env::home_dir(),
                1 => std::env::home_dir().map(|h| h.join("Desktop")),
                2 => std::env::home_dir().map(|h| h.join("Documents")),
                3 => std::env::home_dir().map(|h| h.join("Downloads")),
                4 => Some(PathBuf::from("trash:///")),
                _ => None,
            };
            if let Some(p) = path {
                if p.is_dir() {
                    nav.lock().unwrap().navigate_to(p.clone());
                    path_entry.set_text(&p.to_string_lossy());
                    {
                        let mut cols = miller_columns.lock().unwrap();
                        while let Some(col) = cols.pop() {
                            miller_box.remove(&col.view);
                        }
                    }
                }
            }
        });
    }

    // ── Keyboard shortcuts ─────────────────────────────────────────

    fn wire_keyboard_shortcuts(&self) {
        let controller = EventControllerKey::new();
        let window = self.window.clone();
        let column_view = self.column_view.clone();
        let string_list = self.string_list.clone();

        controller.connect_key_pressed(move |_, key, _mods, _state| {
            match key {
                gdk::Key::Delete | gdk::Key::KP_Delete => {
                    // Move selected items to trash
                    eprintln!("Delete pressed");
                    glib::Propagation::Stop
                }
                gdk::Key::F2 => {
                    // Rename selected item
                    eprintln!("F2 pressed");
                    glib::Propagation::Stop
                }
                gdk::Key::Return | gdk::Key::KP_Enter => {
                    // Open selected item
                    eprintln!("Enter pressed");
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });

        self.window.add_controller(controller);
    }

    // ── Context menu ───────────────────────────────────────────────

    fn wire_context_menu(&self) {
        // Right-click on the column view shows a context menu
        let right_click = gtk4::GestureClick::new();
        right_click.set_button(3); // Right mouse button
        right_click.connect_pressed(move |_gesture, _n_press, x, y| {
            eprintln!("Right click at ({}, {})", x, y);
        });
        self.column_view.add_controller(right_click);
    }

    // ── Public API ─────────────────────────────────────────────────

    pub async fn load_directory(&self, path: &std::path::Path) {
        self.path_entry.set_text(&path.to_string_lossy());

        match crate::models::file_item::list_directory(path).await {
            Ok(items) => {
                let mut items = items;
                items.sort_by(|a, b| {
                    match (a.is_dir, b.is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    }
                });

                *self.all_items.lock().unwrap() = items.clone();

                let n = self.string_list.n_items();
                for i in (0..n).rev() {
                    self.string_list.remove(i);
                }
                for item in &items {
                    self.string_list.append(&item.name);
                }

                self.icon_view.populate(&items);

                // Set up file monitoring
                self.setup_file_monitor(path);

                eprintln!("Loaded {} items from {}", items.len(), path.display());
            }
            Err(e) => {
                eprintln!("Error listing {}: {}", path.display(), e);
            }
        }
    }

    fn setup_file_monitor(&self, path: &std::path::Path) {
        let file = gio::File::for_path(path);
        if let Ok(monitor) = file.monitor_directory(gio::FileMonitorFlags::NONE, None::<&gio::Cancellable>) {
            let path_clone = path.to_path_buf();
            monitor.connect_changed(move |_monitor, file, _other_file, event| {
                match event {
                    gio::FileMonitorEvent::Created => {
                        eprintln!("File created: {:?}", file.path());
                    }
                    gio::FileMonitorEvent::Deleted => {
                        eprintln!("File deleted: {:?}", file.path());
                    }
                    gio::FileMonitorEvent::Changed => {
                        eprintln!("File changed: {:?}", file.path());
                    }
                    _ => {}
                }
                let _ = &path_clone;
            });
            *self.file_monitor.lock().unwrap() = Some(monitor);
        }
    }

    pub fn update_nav_buttons(&self) {
        let nav = self.nav.lock().unwrap();
        self.back_button.set_sensitive(nav.can_go_back());
        self.forward_button.set_sensitive(nav.can_go_forward());
    }

    /// Generate a thumbnail for an image file. Returns None for non-images or on error.
    pub fn generate_thumbnail(&self, path: &std::path::Path, _size: i32) -> Option<gtk4::gdk::Texture> {
        let file = gio::File::for_path(path);
        gtk4::gdk::Texture::from_file(&file).ok()
    }
}

/// Format a file size in bytes to a human-readable string.
fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    if size == 0 {
        return "0 B".to_string();
    }
    let exp = (size as f64).log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
    let value = size as f64 / 1024f64.powi(exp as i32);
    if exp == 0 {
        format!("{} {}", size, UNITS[0])
    } else {
        format!("{:.1} {}", value, UNITS[exp])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(5 * 1024 * 1024), "5.0 MB");
    }

    #[test]
    fn test_format_size_gigabytes() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }
}

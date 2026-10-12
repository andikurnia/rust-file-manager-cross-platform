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
    pub scroll: ScrolledWindow,
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
    pub miller_scroll: GtkBox,
    pub miller_box: GtkBox,
    pub stack: Stack,
    pub icon_view: crate::views::icon_view::IconView,
    pub search_entry: SearchEntry,
    pub all_items: Arc<Mutex<Vec<FileItem>>>,
    pub file_monitor: Arc<Mutex<Option<gio::FileMonitor>>>,
}

impl MainWindow {
    pub fn new(app: &gtk4::Application, initial_path: PathBuf) -> Self {
        // Load CSS stylesheet
        let provider = gtk4::CssProvider::new();
        provider.load_from_path("resources/style.css");
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap(),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Finder")
            .default_width(1100)
            .default_height(650)
            .build();

        // ── Header bar ──────────────────────────────────────────────
        let header = HeaderBar::new();
        window.set_titlebar(Some(&header));

        // Left: Navigation controls (back/forward grouped)
        let nav_box = GtkBox::new(Orientation::Horizontal, 0);
        nav_box.add_css_class("nav-group");

        let back_button = Button::from_icon_name("go-previous-symbolic");
        back_button.set_tooltip_text(Some("Back"));
        back_button.add_css_class("nav-button");
        nav_box.append(&back_button);

        let forward_button = Button::from_icon_name("go-next-symbolic");
        forward_button.set_tooltip_text(Some("Forward"));
        forward_button.add_css_class("nav-button");
        nav_box.append(&forward_button);

        header.pack_start(&nav_box);

        // Center: App title with icon
        let title_box = GtkBox::new(Orientation::Horizontal, 6);
        title_box.set_halign(gtk4::Align::Center);
        title_box.add_css_class("app-title");

        let title_icon = Image::from_icon_name("folder-remote");
        title_icon.set_pixel_size(18);
        let title_label = Label::new(Some("Finder"));
        title_label.add_css_class("app-title-label");
        title_box.append(&title_icon);
        title_box.append(&title_label);

        header.pack_start(&title_box);

        // Right: Search + view switcher
        let search_entry = SearchEntry::builder()
            .placeholder_text("Search")
            .width_request(200)
            .build();
        search_entry.add_css_class("search-bar");
        header.pack_end(&search_entry);

        // View switcher as segmented control
        let view_switcher = GtkBox::new(Orientation::Horizontal, 0);
        view_switcher.add_css_class("view-switcher");

        let list_btn = Button::from_icon_name("view-list-symbolic");
        list_btn.set_tooltip_text(Some("List View"));
        list_btn.add_css_class("view-btn");
        list_btn.add_css_class("view-btn-first");
        view_switcher.append(&list_btn);

        let columns_btn = Button::from_icon_name("view-column-symbolic");
        columns_btn.set_tooltip_text(Some("Column View"));
        columns_btn.add_css_class("view-btn");
        view_switcher.append(&columns_btn);

        let icons_btn = Button::from_icon_name("view-grid-symbolic");
        icons_btn.set_tooltip_text(Some("Icon View"));
        icons_btn.add_css_class("view-btn");
        icons_btn.add_css_class("view-btn-last");
        view_switcher.append(&icons_btn);

        header.pack_end(&view_switcher);

        // Path bar below header (breadcrumb style)
        let path_bar_box = GtkBox::new(Orientation::Horizontal, 4);
        path_bar_box.add_css_class("path-bar");
        path_bar_box.set_margin_start(12);
        path_bar_box.set_margin_end(12);
        path_bar_box.set_margin_top(6);
        path_bar_box.set_margin_bottom(6);

        let path_entry = Entry::builder()
            .placeholder_text("Path")
            .hexpand(true)
            .build();
        path_entry.add_css_class("path-entry");
        path_bar_box.append(&path_entry);

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
        // No outer ScrolledWindow — each column has its own vertical scroller.
        // The box grows horizontally as columns are added.
        let miller_box = GtkBox::new(Orientation::Horizontal, 0);
        miller_box.set_vexpand(true);
        miller_box.set_hexpand(true);
        let miller_scroll = miller_box.clone();

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

        // Window layout: path bar on top, content below
        let vbox = GtkBox::new(Orientation::Vertical, 0);
        vbox.append(&path_bar_box);
        vbox.append(&paned);

        window.set_child(Some(&vbox));

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
            let row = GtkBox::new(Orientation::Horizontal, 10);
            row.set_margin_top(4);
            row.set_margin_bottom(4);
            row.set_margin_start(8);
            row.set_margin_end(8);
            let icon = Image::from_icon_name("text-x-generic");
            icon.set_pixel_size(20);
            let label = Label::new(None);
            label.set_halign(gtk4::Align::Start);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            label.set_hexpand(true);
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
                        self.miller_box.remove(&col.scroll);
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

        // Wrap in ScrolledWindow for independent vertical scrolling
        let col_scroll = ScrolledWindow::new();
        col_scroll.set_child(Some(&view));
        col_scroll.set_vexpand(true);
        col_scroll.set_size_request(200, -1);
        col_scroll.set_hexpand(false);

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

        // Use GestureClick instead of selection notify — handles every click,
        // including re-clicking an already-selected item
        let click = gtk4::GestureClick::new();
        click.set_button(1); // Left mouse button

        let list_for_click = list.clone();
        let sel_for_click = sel.clone();
        click.connect_released(move |_gesture, _n_press, _x, _y| {
            // Get the selected item from the list model
            let n = list_for_click.n_items();
            if n == 0 {
                return;
            }
            // Find which item was clicked by checking the selection
            let sel_item = sel_for_click.selected_item();
            if let Some(selected_obj) = sel_item {
                let name = selected_obj.downcast::<StringObject>().unwrap().string();
                let child_path = path_for_nav.join(&name);
                if child_path.is_dir() {
                    // Truncate columns after this one
                    {
                        let mut cols = miller_cols.lock().unwrap();
                        if let Some(idx) = cols.iter().position(|c| c.path == path_for_nav) {
                            while cols.len() > idx + 1 {
                                if let Some(col) = cols.pop() {
                                    miller_box.remove(&col.scroll);
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

                    let new_scroll = ScrolledWindow::new();
                    new_scroll.set_child(Some(&new_view));
                    new_scroll.set_vexpand(true);
                    new_scroll.set_size_request(200, -1);
                    new_scroll.set_hexpand(false);

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

                    miller_box.append(&new_scroll);
                    miller_cols.lock().unwrap().push(MillerColumn {
                        path: child_path.clone(),
                        list,
                        selection: new_sel,
                        view: new_view,
                        scroll: new_scroll,
                    });

                    nav.lock().unwrap().navigate_to(child_path.clone());
                    path_entry.set_text(&child_path.to_string_lossy());
                }
            }
        });
        view.add_controller(click);

        self.miller_box.append(&col_scroll);
        self.miller_columns.lock().unwrap().push(MillerColumn {
            path: path.to_path_buf(),
            list,
            selection: sel,
            view,
            scroll: col_scroll,
        });
    }

    // ── Navigation wiring ──────────────────────────────────────────

    fn wire_nav_buttons(&self) {
        let nav = self.nav.clone();
        let path_entry = self.path_entry.clone();
        let miller_columns = self.miller_columns.clone();
        let miller_box = self.miller_box.clone();
        let back_button = self.back_button.clone();
        let all_items_bk = self.all_items.clone();
        let string_list_bk = self.string_list.clone();
        let icon_view_bk = self.icon_view.widget.clone();

        back_button.connect_clicked(move |_| {
            if let Some(prev) = nav.lock().unwrap().go_back() {
                path_entry.set_text(&prev.to_string_lossy());
                {
                    let mut cols = miller_columns.lock().unwrap();
                    while let Some(col) = cols.pop() {
                        miller_box.remove(&col.scroll);
                    }
                }
                // Re-add the Miller column for the previous path
                let prev_clone = prev.clone();
                let miller_cols_clone = miller_columns.clone();
                let miller_box_clone = miller_box.clone();
                let all_items_clone = all_items_bk.clone();
                let string_list_clone = string_list_bk.clone();
                let icon_view_clone = icon_view_bk.clone();
                let path_entry_clone = path_entry.clone();

                // Add the column back
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

                let path_owned = prev_clone.clone();
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

                let col_scroll = ScrolledWindow::new();
                col_scroll.set_child(Some(&view));
                col_scroll.set_vexpand(true);
                col_scroll.set_size_request(200, -1);
                col_scroll.set_hexpand(false);
                miller_box_clone.append(&col_scroll);
                miller_cols_clone.lock().unwrap().push(MillerColumn {
                    path: prev_clone.clone(),
                    list,
                    selection: sel,
                    view,
                    scroll: col_scroll,
                });

                // Also update the list view and icon view
                let path_clone2 = prev_clone.clone();
                glib::spawn_future_local(async move {
                    match crate::models::file_item::list_directory(&path_clone2).await {
                        Ok(items) => {
                            let mut items = items;
                            items.sort_by(|a, b| {
                                match (a.is_dir, b.is_dir) {
                                    (true, false) => std::cmp::Ordering::Less,
                                    (false, true) => std::cmp::Ordering::Greater,
                                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                                }
                            });
                            *all_items_clone.lock().unwrap() = items.clone();
                            let n = string_list_clone.n_items();
                            for i in (0..n).rev() {
                                string_list_clone.remove(i);
                            }
                            for item in &items {
                                string_list_clone.append(&item.name);
                            }
                            // Update icon view
                            let icon_view = crate::views::icon_view::IconView { widget: icon_view_clone };
                            icon_view.populate(&items);
                        }
                        Err(e) => {
                            eprintln!("Error listing {}: {}", path_clone2.display(), e);
                        }
                    }
                });
            }
        });

        let nav = self.nav.clone();
        let path_entry = self.path_entry.clone();
        let miller_columns = self.miller_columns.clone();
        let miller_box = self.miller_box.clone();
        let forward_button = self.forward_button.clone();
        let all_items_fw = self.all_items.clone();
        let string_list_fw = self.string_list.clone();
        let icon_view_fw = self.icon_view.widget.clone();

        forward_button.connect_clicked(move |_| {
            if let Some(next) = nav.lock().unwrap().go_forward() {
                path_entry.set_text(&next.to_string_lossy());
                {
                    let mut cols = miller_columns.lock().unwrap();
                    while let Some(col) = cols.pop() {
                        miller_box.remove(&col.scroll);
                    }
                }
                // Re-add the Miller column for the next path
                let next_clone = next.clone();
                let miller_cols_clone = miller_columns.clone();
                let miller_box_clone = miller_box.clone();
                let all_items_clone = all_items_fw.clone();
                let string_list_clone = string_list_fw.clone();
                let icon_view_clone = icon_view_fw.clone();

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

                let path_owned = next_clone.clone();
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

                let col_scroll = ScrolledWindow::new();
                col_scroll.set_child(Some(&view));
                col_scroll.set_vexpand(true);
                col_scroll.set_size_request(200, -1);
                col_scroll.set_hexpand(false);
                miller_box_clone.append(&col_scroll);
                miller_cols_clone.lock().unwrap().push(MillerColumn {
                    path: next_clone.clone(),
                    list,
                    selection: sel,
                    view,
                    scroll: col_scroll,
                });

                // Also update the list view and icon view
                let path_clone2 = next_clone.clone();
                glib::spawn_future_local(async move {
                    match crate::models::file_item::list_directory(&path_clone2).await {
                        Ok(items) => {
                            let mut items = items;
                            items.sort_by(|a, b| {
                                match (a.is_dir, b.is_dir) {
                                    (true, false) => std::cmp::Ordering::Less,
                                    (false, true) => std::cmp::Ordering::Greater,
                                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                                }
                            });
                            *all_items_clone.lock().unwrap() = items.clone();
                            let n = string_list_clone.n_items();
                            for i in (0..n).rev() {
                                string_list_clone.remove(i);
                            }
                            for item in &items {
                                string_list_clone.append(&item.name);
                            }
                            let icon_view = crate::views::icon_view::IconView { widget: icon_view_clone };
                            icon_view.populate(&items);
                        }
                        Err(e) => {
                            eprintln!("Error listing {}: {}", path_clone2.display(), e);
                        }
                    }
                });
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
                        miller_box.remove(&col.scroll);
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
        let all_items_sb = self.all_items.clone();
        let string_list_sb = self.string_list.clone();
        let icon_view_sb = self.icon_view.widget.clone();

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
                let is_trash = p.to_string_lossy().starts_with("trash:");
                if is_trash {
                    let trash_path = PathBuf::from("trash:///");
                    nav.lock().unwrap().navigate_to(trash_path.clone());
                    path_entry.set_text("Trash");
                    {
                        let mut cols = miller_columns.lock().unwrap();
                        while let Some(col) = cols.pop() {
                            miller_box.remove(&col.scroll);
                        }
                    }
                    let all_items_clone = all_items_sb.clone();
                    let string_list_clone = string_list_sb.clone();
                    let icon_view_clone = icon_view_sb.clone();
                    glib::spawn_future_local(async move {
                        match crate::models::file_item::list_directory(&trash_path).await {
                            Ok(items) => {
                                let mut items = items;
                                items.sort_by(|a, b| {
                                    match (a.is_dir, b.is_dir) {
                                        (true, false) => std::cmp::Ordering::Less,
                                        (false, true) => std::cmp::Ordering::Greater,
                                        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                                    }
                                });
                                *all_items_clone.lock().unwrap() = items.clone();
                                let n = string_list_clone.n_items();
                                for i in (0..n).rev() {
                                    string_list_clone.remove(i);
                                }
                                for item in &items {
                                    string_list_clone.append(&item.name);
                                }
                                let icon_view = crate::views::icon_view::IconView { widget: icon_view_clone };
                                icon_view.populate(&items);
                                eprintln!("Loaded {} items from trash", items.len());
                            }
                            Err(e) => {
                                eprintln!("Error listing trash: {}", e);
                            }
                        }
                    });
                } else if p.is_dir() {
                    nav.lock().unwrap().navigate_to(p.clone());
                    path_entry.set_text(&p.to_string_lossy());
                    {
                        let mut cols = miller_columns.lock().unwrap();
                        while let Some(col) = cols.pop() {
                            miller_box.remove(&col.scroll);
                        }
                    }
                    let p_clone = p.clone();
                    let all_items_clone = all_items_sb.clone();
                    let string_list_clone = string_list_sb.clone();
                    let icon_view_clone = icon_view_sb.clone();
                    glib::spawn_future_local(async move {
                        match crate::models::file_item::list_directory(&p_clone).await {
                            Ok(items) => {
                                let mut items = items;
                                items.sort_by(|a, b| {
                                    match (a.is_dir, b.is_dir) {
                                        (true, false) => std::cmp::Ordering::Less,
                                        (false, true) => std::cmp::Ordering::Greater,
                                        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                                    }
                                });
                                *all_items_clone.lock().unwrap() = items.clone();
                                let n = string_list_clone.n_items();
                                for i in (0..n).rev() {
                                    string_list_clone.remove(i);
                                }
                                for item in &items {
                                    string_list_clone.append(&item.name);
                                }
                                let icon_view = crate::views::icon_view::IconView { widget: icon_view_clone };
                                icon_view.populate(&items);
                            }
                            Err(e) => {
                                eprintln!("Error listing {}: {}", p_clone.display(), e);
                            }
                        }
                    });
                }
            }
        });
    }

    // ── Keyboard shortcuts ─────────────────────────────────────────

    fn wire_keyboard_shortcuts(&self) {
        let controller = EventControllerKey::new();
        let all_items_kb = self.all_items.clone();
        let window_kb = self.window.clone();

        controller.connect_key_pressed(move |_, key, _mods, _state| {
            match key {
                gdk::Key::Delete | gdk::Key::KP_Delete => {
                    let items = all_items_kb.lock().unwrap();
                    if let Some(selected) = items.first() {
                        let path = selected.path.clone();
                        let file = gio::File::for_path(&path);
                        match file.trash(None::<&gio::Cancellable>) {
                            Ok(_) => eprintln!("Moved to trash: {}", path.display()),
                            Err(e) => eprintln!("Failed to trash {}: {}", path.display(), e),
                        }
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::F2 => {
                    let (path, old_name) = {
                        let items = all_items_kb.lock().unwrap();
                        match items.first() {
                            Some(selected) => (selected.path.clone(), selected.name.clone()),
                            None => return glib::Propagation::Stop,
                        }
                    };
                    // Show inline rename dialog
                    let dialog = gtk4::Dialog::with_buttons(
                        Some("Rename"),
                        Some(&window_kb),
                        gtk4::DialogFlags::MODAL,
                        &[("Cancel", gtk4::ResponseType::Cancel), ("Rename", gtk4::ResponseType::Accept)],
                    );
                    let content = dialog.content_area();
                    let entry = gtk4::Entry::new();
                    entry.set_text(&old_name);
                    entry.set_activates_default(true);
                    content.append(&entry);
                    dialog.set_default_response(gtk4::ResponseType::Accept);
                    dialog.connect_response(move |dialog, response| {
                        if response == gtk4::ResponseType::Accept {
                            let new_name = entry.text().to_string();
                            if !new_name.is_empty() && new_name != old_name {
                                let parent = match path.parent() {
                                    Some(p) => p.to_path_buf(),
                                    None => return,
                                };
                                let new_path = parent.join(&new_name);
                                let old_file = gio::File::for_path(&path);
                                let new_file = gio::File::for_path(&new_path);
                                match old_file.move_(&new_file, gio::FileCopyFlags::NONE, None::<&gio::Cancellable>, None::<&mut dyn FnMut(i64, i64)>) {
                                    Ok(_) => eprintln!("Renamed to: {}", new_path.display()),
                                    Err(e) => eprintln!("Rename failed: {}", e),
                                }
                            }
                        }
                        dialog.close();
                    });
                    dialog.present();
                    glib::Propagation::Stop
                }
                gdk::Key::Return | gdk::Key::KP_Enter => {
                    let items = all_items_kb.lock().unwrap();
                    if let Some(selected) = items.first() {
                        let path = selected.path.clone();
                        if path.is_dir() {
                            eprintln!("Navigate into: {}", path.display());
                        } else {
                            let file = gio::File::for_path(&path);
                            let uri = file.uri();
                            let _ = gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>);
                        }
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::o if _mods & gdk::ModifierType::CONTROL_MASK.bits() != 0 => {
                    // Ctrl+O: Open With
                    let (path, mime_type) = {
                        let items = all_items_kb.lock().unwrap();
                        match items.first() {
                            Some(selected) => (selected.path.clone(), selected.mime_type.clone()),
                            None => return glib::Propagation::Stop,
                        }
                    };
                    let dialog = gtk4::AppChooserDialog::for_content_type(
                        Some(&window_kb),
                        gtk4::DialogFlags::MODAL,
                        &mime_type,
                    );
                    dialog.connect_response(move |dialog, response| {
                        if response == gtk4::ResponseType::Ok {
                            if let Some(app_info) = dialog.app_info() {
                                let file = gio::File::for_path(&path);
                                let _ = app_info.launch(&[file], None::<&gio::AppLaunchContext>);
                            }
                        }
                        dialog.close();
                    });
                    dialog.present();
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });

        self.window.add_controller(controller);
    }

    // ── Context menu ───────────────────────────────────────────────

    fn wire_context_menu(&self) {
        // Right-click on the column view shows a context menu with actions
        let right_click = gtk4::GestureClick::new();
        right_click.set_button(3); // Right mouse button

        let all_items_ctx = self.all_items.clone();
        let icon_view_ctx = self.icon_view.widget.clone();
        let window_ctx = self.window.clone();

        right_click.connect_pressed(move |_gesture, _n_press, _x, _y| {
            let items = all_items_ctx.lock().unwrap();
            if let Some(selected) = items.first() {
                let path = selected.path.clone();
                let name = selected.name.clone();
                let is_dir = selected.is_dir;
                let mime_type = selected.mime_type.clone();
                let size = selected.size;
                let modified = selected.modified;

                let popover = crate::operations::context_menu::create_context_menu(&path);
                popover.set_parent(&icon_view_ctx);
                popover.popup();

                // Wire Open action
                let path_open = path.clone();
                let is_dir_open = is_dir;
                // Wire Trash action
                let path_trash = path.clone();
                // Wire Get Info action
                let path_info = path.clone();
                let name_info = name.clone();
                let mime_info = mime_type.clone();
                let size_info = size;
                let modified_info = modified;
                let is_dir_info = is_dir;

                // Note: Full action wiring requires adding actions to the app's action group.
                // For now, the popover is shown and actions are logged.
                eprintln!("Context menu for: {}", path.display());
            }
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

    /// Rename a file from `old_path` to `new_name` in the same directory.
    pub fn rename_file(&self, old_path: &std::path::Path, new_name: &str) -> Result<(), glib::Error> {
        let parent = old_path.parent().ok_or_else(|| {
            glib::Error::new(glib::FileError::Noent, "No parent directory")
        })?;
        let new_path = parent.join(new_name);
        let old_file = gio::File::for_path(old_path);
        let new_file = gio::File::for_path(&new_path);
        old_file.move_(
            &new_file,
            gio::FileCopyFlags::NONE,
            None::<&gio::Cancellable>,
            None::<&mut dyn FnMut(i64, i64)>,
        )?;
        Ok(())
    }

    /// Open a file with the default application.
    pub fn open_file(&self, path: &std::path::Path) {
        let file = gio::File::for_path(path);
        let uri = file.uri();
        eprintln!("Opening: {}", uri);
        // Use gio to launch the default application
        let _ = gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>);
    }

    /// Show file info in a dialog.
    pub fn show_file_info(&self, path: &std::path::Path) {
        let file = gio::File::for_path(path);
        let info = file.query_info(
            "standard::*,time::modified",
            gio::FileQueryInfoFlags::NONE,
            None::<&gio::Cancellable>,
        );
        if let Ok(info) = info {
            let name = info.name().to_string_lossy().to_string();
            let size = info.size();
            let modified = info.modification_date_time()
                .map(|dt| dt.format("%Y-%m-%d %H:%M").unwrap_or_default())
                .unwrap_or_else(|| "Unknown".to_string().into());
            let content_type = info.content_type().unwrap_or_default();
            eprintln!("File: {}, Size: {}, Modified: {}, Type: {}", name, size, modified, content_type);
        }
    }

    /// Move a file to trash.
    pub fn move_to_trash(&self, path: &std::path::Path) -> Result<(), glib::Error> {
        let file = gio::File::for_path(path);
        file.trash(None::<&gio::Cancellable>)
    }

    /// Show file info in a dialog.
    pub fn show_file_info_dialog(&self, path: &std::path::Path) {
        let file = gio::File::for_path(path);
        let info = file.query_info(
            "standard::*,time::modified,standard::content-type",
            gio::FileQueryInfoFlags::NONE,
            None::<&gio::Cancellable>,
        );
        if let Ok(info) = info {
            let name = info.name().to_string_lossy().to_string();
            let size = info.size();
            let modified = info.modification_date_time()
                .map(|dt| dt.format("%Y-%m-%d %H:%M").unwrap_or_default())
                .unwrap_or_else(|| "Unknown".to_string().into());
            let content_type = info.content_type().unwrap_or_default();
            let is_dir = info.file_type() == gio::FileType::Directory;

            let dialog = gtk4::Dialog::with_buttons(
                Some("File Info"),
                Some(&self.window),
                gtk4::DialogFlags::MODAL,
                &[("Close", gtk4::ResponseType::Close)],
            );
            dialog.set_default_size(400, 300);

            let content = dialog.content_area();
            content.set_orientation(Orientation::Vertical);
            content.set_spacing(12);
            content.set_margin_top(16);
            content.set_margin_bottom(16);
            content.set_margin_start(16);
            content.set_margin_end(16);

            // Name
            let name_box = GtkBox::new(Orientation::Vertical, 4);
            let name_label = Label::new(Some("Name:"));
            name_label.set_halign(gtk4::Align::Start);
            let name_value = Label::new(Some(&name));
            name_value.set_halign(gtk4::Align::Start);
            name_value.set_selectable(true);
            name_box.append(&name_label);
            name_box.append(&name_value);
            content.append(&name_box);

            // Type
            let type_box = GtkBox::new(Orientation::Vertical, 4);
            let type_label = Label::new(Some("Type:"));
            type_label.set_halign(gtk4::Align::Start);
            let type_value = Label::new(Some(if is_dir { "Folder" } else { &content_type }));
            type_value.set_halign(gtk4::Align::Start);
            type_box.append(&type_label);
            type_box.append(&type_value);
            content.append(&type_box);

            // Size
            let size_box = GtkBox::new(Orientation::Vertical, 4);
            let size_label = Label::new(Some("Size:"));
            size_label.set_halign(gtk4::Align::Start);
            let size_value = Label::new(Some(&if is_dir { "—".to_string() } else { format_size(size as u64) }));
            size_value.set_halign(gtk4::Align::Start);
            size_box.append(&size_label);
            size_box.append(&size_value);
            content.append(&size_box);

            // Modified
            let modified_box = GtkBox::new(Orientation::Vertical, 4);
            let modified_label = Label::new(Some("Modified:"));
            modified_label.set_halign(gtk4::Align::Start);
            let modified_value = Label::new(Some(&modified));
            modified_value.set_halign(gtk4::Align::Start);
            modified_box.append(&modified_label);
            modified_box.append(&modified_value);
            content.append(&modified_box);

            // Path
            let path_box = GtkBox::new(Orientation::Vertical, 4);
            let path_label = Label::new(Some("Path:"));
            path_label.set_halign(gtk4::Align::Start);
            let path_value = Label::new(Some(&path.display().to_string()));
            path_value.set_halign(gtk4::Align::Start);
            path_value.set_selectable(true);
            path_value.set_ellipsize(gtk4::pango::EllipsizeMode::Middle);
            path_box.append(&path_label);
            path_box.append(&path_value);
            content.append(&path_box);

            dialog.connect_response(|dialog, _| dialog.close());
            dialog.present();
        }
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

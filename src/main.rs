mod app;
mod models;
mod operations;
mod settings;
mod sidebar;
mod views;
mod window;

use gtk4::prelude::*;
use gtk4::Application;

fn main() {
    let app = Application::builder()
        .application_id("com.example.finder")
        .build();

    app.connect_activate(|app| {
        // Load persisted settings
        let settings = settings::AppSettings::load();

        let home = std::env::home_dir().unwrap_or_else(|| "/".into());
        let win = window::MainWindow::new(app, home.clone());

        // Apply settings
        win.window.set_default_size(settings.window_width, settings.window_height);
        match settings.view_mode {
            settings::ViewMode::List => win.set_view_mode("list"),
            settings::ViewMode::Column => win.set_view_mode("columns"),
            settings::ViewMode::Icon => win.set_view_mode("icon"),
        }

        win.setup_list_columns();
        win.update_nav_buttons();
        win.window.present();

        // Save settings on close
        let win_weak = win.window.clone();
        let settings_clone = settings.clone();
        win_weak.connect_close_request(move |_win| {
            let mut s = settings_clone.clone();
            s.window_width = _win.default_size().0;
            s.window_height = _win.default_size().1;
            s.save();
            glib::Propagation::Proceed
        });

        let path = home.clone();
        glib::spawn_future_local(async move {
            win.load_directory(&path).await;
        });
    });

    app.run();
}

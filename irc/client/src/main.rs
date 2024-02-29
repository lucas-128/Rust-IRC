mod app;
use app::setup_app::build_ui;

use gtk::gdk;

use gtk::prelude::*;
use gtk::Application;
fn main() {
    let application = Application::new(None, Default::default()); //Application::builder().application_id(&app_id).build();

    // Load CSS and Build App
    application.connect_activate(|app| {
        let provider = gtk::CssProvider::new();
        let style = include_bytes!("resources/main_style.css");
        provider.load_from_data(style).expect("Failed to load CSS");
        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::default().expect("Error initializing gtk css provider."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        build_ui(app);
    });

    application.run();
}

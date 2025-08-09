use gtk4::prelude::*;
use gtk4::Application;

pub fn run() {
    let app = Application::builder()
        .application_id("com.example.DesktopEntryCreator")
        .build();

    app.connect_activate(|app| {
        crate::ui::windows::main_window::show_main_window(app);
    });

    app.run();
}

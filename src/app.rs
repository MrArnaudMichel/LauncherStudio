use gtk4::prelude::*;
use gtk4::Application;
use crate::ui;

pub fn run() {
    // Crée une nouvelle application GTK4 avec un ID unique
    let app = Application::builder()
        .application_id("com.example.desktopentrymanager")
        .build();

    // Connecte le signal 'activate' à la fonction qui ouvre la fenêtre principale
    app.connect_activate(|app| {
        ui::windows::main_window::show_main_window(app);
    });

    // Lance la boucle principale GTK
    app.run();
}

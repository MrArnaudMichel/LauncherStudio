use adw::prelude::*;
use adw::Application;
use crate::ui;

pub fn run() {
    // Initialise Libadwaita pour un style moderne
    let _ = adw::init();

    // Crée une nouvelle application Libadwaita (sous-classe de GTK4 Application)
    let app = Application::builder()
        .application_id("com.example.desktopentrymanager")
        .build();

    // Connecte le signal 'activate' à la fonction qui ouvre la fenêtre principale
    app.connect_activate(|app| {
        // Note: adw::Application est compatible avec &gtk4::Application attendu en paramètre
        ui::windows::main_window::show_main_window(app);
    });

    // Lance la boucle principale
    app.run();
}

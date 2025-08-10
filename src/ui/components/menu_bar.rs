use gtk4::{PopoverMenuBar, ApplicationWindow};
use gtk4::gio::Menu;
use gtk4::prelude::*;
use gtk4::Application;

// Builds the application menu bar with File/View/Tools/Help/Credits.
// It wires no actions itself; it only defines the action names expected by the main window.
pub fn build_menu_bar(_app: &Application, _win: &ApplicationWindow) -> PopoverMenuBar {
    let menu_model = Menu::new();

    // File menu
    let file_menu = Menu::new();
    file_menu.append(Some("New"), Some("app.new"));
    file_menu.append(Some("Open"), Some("app.open"));
    file_menu.append(Some("Save"), Some("app.save"));
    file_menu.append(Some("Refresh"), Some("app.refresh"));
    file_menu.append(Some("Quit"), Some("app.quit"));
    menu_model.append_submenu(Some("File"), &file_menu);

    // View menu
    let view_menu = Menu::new();
    view_menu.append(Some("Toggle Fullscreen"), Some("win.toggle_fullscreen"));
    menu_model.append_submenu(Some("View"), &view_menu);

    // Tools / Help / Credits placeholders
    let tools_menu = Menu::new();
    menu_model.append_submenu(Some("Tools"), &tools_menu);
    let help_menu = Menu::new();
    menu_model.append_submenu(Some("Help"), &help_menu);
    let credits_menu = Menu::new();
    menu_model.append_submenu(Some("Credits"), &credits_menu);

    PopoverMenuBar::from_model(Some(&menu_model))
}

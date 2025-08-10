use gtk4::prelude::*;
use gtk4::{self, Align, Application, ApplicationWindow, Box as GtkBox, Button, FileChooserDialog, FileChooserAction, Orientation, ResponseType, ScrolledWindow, Label, Image, ListBoxRow, ToggleButton};
use gtk4::gio::SimpleAction;
use adw::{ApplicationWindow as AdwApplicationWindow, HeaderBar as AdwHeaderBar, StyleManager, ColorScheme, ToolbarView, AboutWindow};
use adw::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::domain::desktop_entry::DesktopEntry;
use crate::services::desktop_writer::DesktopWriter;
use crate::services::desktop_reader::DesktopReader;

pub fn show_main_window(app: &impl IsA<Application>) {
    // Upcast and take a strong reference to GtkApplication (works for both Gtk and Adw apps)
    let app: Application = app.upcast_ref::<Application>().clone();

    let win = AdwApplicationWindow::builder()
        .application(&app)
        .title("Desktop Entry Manager")
        .default_width(1000)
        .default_height(700)
        .resizable(true)
        .build();

    // Header bar (Libadwaita) stays minimal (title), we build our own menu + toolbar below
    let header = AdwHeaderBar::new();

    // Theme toggle (dark/light) button placed at the right side of the header bar,
    // which is visually left of the window close button in GNOME CSD.
    let theme_btn = ToggleButton::new();
    let theme_icon = Image::from_icon_name("weather-night-symbolic");
    theme_icon.set_pixel_size(16);
    theme_btn.set_child(Some(&theme_icon));
    theme_btn.set_tooltip_text(Some("Toggle dark theme"));
    header.pack_end(&theme_btn);

    // Initialize from Adwaita style manager and wire toggling
    let style_manager = StyleManager::default();
    let is_dark = style_manager.is_dark();
    theme_btn.set_active(is_dark);
    let initial_icon = if is_dark { "weather-sunny-symbolic" } else { "weather-night-symbolic" };
    theme_icon.set_icon_name(Some(initial_icon));

    let theme_icon_c = theme_icon.clone();
    let style_manager_c = style_manager.clone();
    theme_btn.connect_toggled(move |btn| {
        let active = btn.is_active();
        // Force dark/light according to toggle
        if active {
            style_manager_c.set_color_scheme(ColorScheme::ForceDark);
        } else {
            style_manager_c.set_color_scheme(ColorScheme::ForceLight);
        }
        let name = if active { "weather-sunny-symbolic" } else { "weather-night-symbolic" };
        theme_icon_c.set_icon_name(Some(name));
    });

    // Use ToolbarView to blend the window bar with app content
    header.add_css_class("flat");

    let root = GtkBox::new(Orientation::Vertical, 6);
    root.set_margin_top(0);
    root.set_margin_bottom(12);
    root.set_margin_start(12);
    root.set_margin_end(12);

    // Top Menu Bar (PopoverMenuBar with gio::Menu)
    let menubar = crate::ui::components::menu_bar::build_menu_bar(&app);

    // Toolbar with icons: New, Open, Save, Refresh (icon-only buttons)
    let crate::ui::components::toolbar::Toolbar { container: toolbar, btn_new, btn_open, btn_save, btn_refresh } = crate::ui::components::toolbar::build_toolbar();

    // Main area: sidebar + editor tabs
    let main_area = GtkBox::new(Orientation::Horizontal, 12);

    // Sidebar list
    let crate::ui::components::sidebar::Sidebar { container: sidebar_scroller, listbox } = crate::ui::components::sidebar::build_sidebar();


    // Bottom status bar
    let crate::ui::components::status_bar::StatusBar { container: status_bar, label: status_label } = crate::ui::components::status_bar::build_status_bar();

    // Editor (modular): build the entire Basic/Advanced/Source notebook
    let editor = crate::ui::editor::entry_form::build_editor();
    let scroller = ScrolledWindow::builder().hexpand(true).vexpand(true).build();
    scroller.set_child(Some(&editor.notebook));
    // Two-way sync between fields and Source tab
    crate::ui::editor::entry_form::wire_source_sync(&editor);

    // Expose editor widgets locally to reuse existing wiring below
    let type_combo = editor.widgets.type_combo.clone();
    let name_entry = editor.widgets.name_entry.clone();
    let generic_name_entry = editor.widgets.generic_name_entry.clone();
    let comment_entry = editor.widgets.comment_entry.clone();
    let exec_entry = editor.widgets.exec_entry.clone();
    let icon_entry = editor.widgets.icon_entry.clone();
    let terminal_check = editor.widgets.terminal_check.clone();
    let nodisplay_check = editor.widgets.nodisplay_check.clone();
    let startup_check = editor.widgets.startup_check.clone();
    let categories_entry = editor.widgets.categories_entry.clone();
    let mimetype_entry = editor.widgets.mimetype_entry.clone();
    let keywords_entry = editor.widgets.keywords_entry.clone();
    let onlyshowin_entry = editor.widgets.onlyshowin_entry.clone();
    let notshowin_entry = editor.widgets.notshowin_entry.clone();
    let tryexec_entry = editor.widgets.tryexec_entry.clone();
    let path_entry = editor.widgets.path_entry.clone();
    let url_entry = editor.widgets.url_entry.clone();
    let actions_entry = editor.widgets.actions_entry.clone();
    let localized_name = editor.widgets.localized_name.clone();
    let localized_gname = editor.widgets.localized_gname.clone();
    let localized_comment = editor.widgets.localized_comment.clone();
    let extra_kv = editor.widgets.extra_kv.clone();
    let source_view = editor.source_view.clone();

    // Buttons for Preview/Save
    let buttons = GtkBox::new(Orientation::Horizontal, 8);
    buttons.set_halign(Align::End);
    let delete_btn = Button::with_label("Delete");
    delete_btn.add_css_class("destructive-action");
    let preview_btn = Button::with_label("Preview");
    let save_btn = Button::with_label("Save .desktop");
    buttons.append(&delete_btn);
    buttons.append(&preview_btn);
    buttons.append(&save_btn);

    // Main area composition
    main_area.append(&sidebar_scroller);
    main_area.append(&scroller);

    // Build root
    root.append(&menubar);
    root.append(&toolbar);
    root.append(&main_area);
    root.append(&buttons);
    root.append(&status_bar);

    // Put content inside a ToolbarView to fuse the header with the app surface
    let toolbar_view = ToolbarView::new();
    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&root));
    win.set_content(Some(&toolbar_view));

    // Simple app state
    #[derive(Default, Clone)]
    struct UiState { selected_path: Option<PathBuf> }
    let state = Rc::new(RefCell::new(UiState::default()));

    // Helpers

    let set_form_from_entry = {
        let type_combo = type_combo.clone();
        let name_entry = name_entry.clone();
        let generic_name_entry = generic_name_entry.clone();
        let comment_entry = comment_entry.clone();
        let exec_entry = exec_entry.clone();
        let icon_entry = icon_entry.clone();
        let terminal_check = terminal_check.clone();
        let nodisplay_check = nodisplay_check.clone();
        let startup_check = startup_check.clone();
        let categories_entry = categories_entry.clone();
        let mimetype_entry = mimetype_entry.clone();
        let keywords_entry = keywords_entry.clone();
        let onlyshowin_entry = onlyshowin_entry.clone();
        let notshowin_entry = notshowin_entry.clone();
        let tryexec_entry = tryexec_entry.clone();
        let path_entry = path_entry.clone();
        let url_entry = url_entry.clone();
        let actions_entry = actions_entry.clone();
        let localized_name = localized_name.clone();
        let localized_gname = localized_gname.clone();
        let localized_comment = localized_comment.clone();
        let extra_kv = extra_kv.clone();
        move |de: &DesktopEntry| {
            // Type
            let idx = match de.type_field.as_str() { "Application" => 0, "Link" => 1, "Directory" => 2, _ => 0 };
            type_combo.set_active(Some(idx));
            name_entry.set_text(&de.name);
            generic_name_entry.set_text(de.generic_name.as_deref().unwrap_or(""));
            comment_entry.set_text(de.comment.as_deref().unwrap_or(""));
            exec_entry.set_text(&de.exec);
            icon_entry.set_text(de.icon.as_deref().unwrap_or(""));
            terminal_check.set_active(de.terminal);
            nodisplay_check.set_active(de.no_display);
            startup_check.set_active(de.startup_notify);
            categories_entry.set_text(&de.categories.join(";"));
            mimetype_entry.set_text(&de.mime_type.join(";"));
            keywords_entry.set_text(&de.keywords.join(";"));
            onlyshowin_entry.set_text(&de.only_show_in.join(";"));
            notshowin_entry.set_text(&de.not_show_in.join(";"));
            tryexec_entry.set_text(de.try_exec.as_deref().unwrap_or(""));
            path_entry.set_text(de.path.as_deref().unwrap_or(""));
            url_entry.set_text(de.url.as_deref().unwrap_or(""));
            actions_entry.set_text(&de.actions.join(";"));
            // Localized
            let ln: Vec<String> = de.name_localized.iter().map(|(l,v)| format!("{}={}", l, v)).collect();
            let lg: Vec<String> = de.generic_name_localized.iter().map(|(l,v)| format!("{}={}", l, v)).collect();
            let lc: Vec<String> = de.comment_localized.iter().map(|(l,v)| format!("{}={}", l, v)).collect();
            localized_name.buffer().set_text(&ln.join("\n"));
            localized_gname.buffer().set_text(&lg.join("\n"));
            localized_comment.buffer().set_text(&lc.join("\n"));
            // Extra
            let extra: Vec<String> = de.extra.iter().map(|(k,v)| format!("{}={}", k, v)).collect();
            extra_kv.buffer().set_text(&extra.join("\n"));
        }
    };

    let refresh_list = {
        let listbox = listbox.clone();
        let status_label = status_label.clone();
        let state = state.clone();
        move || {
            // Clear existing
            while let Some(child) = listbox.first_child() { listbox.remove(&child); }
            match DesktopReader::list_desktop_files() {
                Ok(paths) => {
                    for path in paths {
                        let (name, icon_str) = match DesktopReader::read_from_path(&path) {
                            Ok(de) => (de.name, de.icon),
                            Err(_) => (path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string(), None),
                        };
                        let row = ListBoxRow::new();
                        let hb = GtkBox::new(Orientation::Horizontal, 6);

                        // Icon image (theme name or file path)
                        let img = if let Some(icon_value) = icon_str.clone() {
                            if icon_value.contains('/') {
                                Image::from_file(icon_value)
                            } else {
                                Image::from_icon_name(&icon_value)
                            }
                        } else {
                            Image::from_icon_name("application-x-executable-symbolic")
                        };
                        img.set_pixel_size(16);
                        hb.append(&img);

                        let lbl = Label::new(Some(&name));
                        lbl.set_xalign(0.0);
                        hb.append(&lbl);
                        row.set_child(Some(&hb));
                        row.set_selectable(true);
                        row.add_css_class("activatable");
                        // store path on row via data
                        row.set_widget_name(&path.to_string_lossy());
                        listbox.append(&row);
                    }
                    status_label.set_text("List refreshed");
                }
                Err(e) => status_label.set_text(&format!("Failed to list: {}", e)),
            }
        }
    };

    // List selection
    {
        let listbox_c = listbox.clone();
        let state_c = state.clone();
        let set_form = set_form_from_entry.clone();
        let status_label = status_label.clone();
        listbox.connect_row_activated(move |_, row| {
            let path_str = row.widget_name();
            let path = PathBuf::from(path_str);
            match DesktopReader::read_from_path(&path) {
                Ok(de) => {
                    set_form(&de);
                    state_c.borrow_mut().selected_path = Some(path.clone());
                    status_label.set_text(&path.to_string_lossy());
                }
                Err(e) => {
                    status_label.set_text(&format!("Open failed: {}", e));
                }
            }
        });
    }

    // Hook toolbar
    {
        let state_c = state.clone();
        let status_label = status_label.clone();
        let set_form = set_form_from_entry.clone();
        btn_new.connect_clicked(move |_| {
            // Clear form by setting empty entry
            set_form(&DesktopEntry { name: String::new(), type_field: "Application".into(), ..Default::default() });
            state_c.borrow_mut().selected_path = None;
            status_label.set_text("New entry");
        });
    }
    {
        let status_label = status_label.clone();
        let set_form = set_form_from_entry.clone();
        btn_open.connect_clicked(move |_| {
            let dialog = FileChooserDialog::new(Some("Open .desktop"), None::<&ApplicationWindow>, FileChooserAction::Open, &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)]);
            let status_label2 = status_label.clone();
            let set_form2 = set_form.clone();
            dialog.connect_response(move |d, resp| {
                if resp == ResponseType::Accept {
                    if let Some(file) = d.file() { if let Some(path) = file.path() {
                        match DesktopReader::read_from_path(&path) {
                            Ok(de) => { set_form2(&de); status_label2.set_text(&path.to_string_lossy()); }
                            Err(e) => status_label2.set_text(&format!("Open failed: {}", e)),
                        }
                    }}
                }
                d.close();
            });
            dialog.show();
        });
    }
    {
        let type_combo = type_combo.clone();
        let name_entry = name_entry.clone();
        let generic_name_entry = generic_name_entry.clone();
        let comment_entry = comment_entry.clone();
        let exec_entry = exec_entry.clone();
        let icon_entry = icon_entry.clone();
        let terminal_check = terminal_check.clone();
        let nodisplay_check = nodisplay_check.clone();
        let startup_check = startup_check.clone();
        let categories_entry = categories_entry.clone();
        let mimetype_entry = mimetype_entry.clone();
        let keywords_entry = keywords_entry.clone();
        let onlyshowin_entry = onlyshowin_entry.clone();
        let notshowin_entry = notshowin_entry.clone();
        let tryexec_entry = tryexec_entry.clone();
        let path_entry = path_entry.clone();
        let url_entry = url_entry.clone();
        let actions_entry = actions_entry.clone();
        let localized_name = localized_name.clone();
        let localized_gname = localized_gname.clone();
        let localized_comment = localized_comment.clone();
        let extra_kv = extra_kv.clone();
        let status_label = status_label.clone();
        btn_save.connect_clicked(move |_| {
            match crate::ui::helpers::collect_entry(&type_combo, &name_entry, &generic_name_entry, &comment_entry, &exec_entry, &icon_entry, &terminal_check, &nodisplay_check, &startup_check, &categories_entry, &mimetype_entry, &keywords_entry, &onlyshowin_entry, &notshowin_entry, &tryexec_entry, &path_entry, &url_entry, &actions_entry, &localized_name, &localized_gname, &localized_comment, &extra_kv) {
                Ok(de) => {
                    let fname = if !de.name.trim().is_empty() { de.name.clone() } else { "desktop-entry".into() };
                    match DesktopWriter::write(&de, &fname, true) {
                        Ok(path) => { status_label.set_text(&format!("Saved: {}", path.display())); }
                        Err(e) => status_label.set_text(&format!("Save failed: {}", e)),
                    }
                }
                Err(e) => status_label.set_text(&format!("Invalid: {}", e)),
            }
        });
    }
    {
        let refresh = refresh_list.clone();
        btn_refresh.connect_clicked(move |_| { refresh(); });
    }

    // Application and Window actions for the menu bar
    {
        // app.new
        let set_form = set_form_from_entry.clone();
        let status_label_new = status_label.clone();
        let app_add_new = app.clone();
        let new_action = SimpleAction::new("new", None);
        new_action.connect_activate(move |_, _| {
            set_form(&DesktopEntry { name: String::new(), type_field: "Application".into(), ..Default::default() });
            status_label_new.set_text("New entry");
        });
        app_add_new.add_action(&new_action);

        // app.open
        let set_form = set_form_from_entry.clone();
        let status_label_open = status_label.clone();
        let app_add_open = app.clone();
        let open_action = SimpleAction::new("open", None);
        open_action.connect_activate(move |_, _| {
            let dialog = FileChooserDialog::new(Some("Open .desktop"), None::<&ApplicationWindow>, FileChooserAction::Open, &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)]);
            let status_label2 = status_label_open.clone();
            let set_form2 = set_form.clone();
            dialog.connect_response(move |d, resp| {
                if resp == ResponseType::Accept {
                    if let Some(file) = d.file() { if let Some(path) = file.path() {
                        match DesktopReader::read_from_path(&path) {
                            Ok(de) => { set_form2(&de); status_label2.set_text(&path.to_string_lossy()); }
                            Err(e) => status_label2.set_text(&format!("Open failed: {}", e)),
                        }
                    }}
                }
                d.close();
            });
            dialog.show();
        });
        app_add_open.add_action(&open_action);

        // app.save
        let type_combo = type_combo.clone();
        let name_entry = name_entry.clone();
        let generic_name_entry = generic_name_entry.clone();
        let comment_entry = comment_entry.clone();
        let exec_entry = exec_entry.clone();
        let icon_entry = icon_entry.clone();
        let terminal_check = terminal_check.clone();
        let nodisplay_check = nodisplay_check.clone();
        let startup_check = startup_check.clone();
        let categories_entry = categories_entry.clone();
        let mimetype_entry = mimetype_entry.clone();
        let keywords_entry = keywords_entry.clone();
        let onlyshowin_entry = onlyshowin_entry.clone();
        let notshowin_entry = notshowin_entry.clone();
        let tryexec_entry = tryexec_entry.clone();
        let path_entry = path_entry.clone();
        let url_entry = url_entry.clone();
        let actions_entry = actions_entry.clone();
        let localized_name = localized_name.clone();
        let localized_gname = localized_gname.clone();
        let localized_comment = localized_comment.clone();
        let extra_kv = extra_kv.clone();
        let status_label = status_label.clone();
        let app_c = app.clone();
        let save_action = SimpleAction::new("save", None);
        save_action.connect_activate(move |_, _| {
            match crate::ui::helpers::collect_entry(&type_combo, &name_entry, &generic_name_entry, &comment_entry, &exec_entry, &icon_entry, &terminal_check, &nodisplay_check, &startup_check, &categories_entry, &mimetype_entry, &keywords_entry, &onlyshowin_entry, &notshowin_entry, &tryexec_entry, &path_entry, &url_entry, &actions_entry, &localized_name, &localized_gname, &localized_comment, &extra_kv) {
                Ok(de) => {
                    let fname = if !de.name.trim().is_empty() { de.name.clone() } else { "desktop-entry".into() };
                    match DesktopWriter::write(&de, &fname, true) {
                        Ok(path) => { status_label.set_text(&format!("Saved: {}", path.display())); }
                        Err(e) => status_label.set_text(&format!("Save failed: {}", e)),
                    }
                }
                Err(e) => status_label.set_text(&format!("Invalid: {}", e)),
            }
        });
        app_c.add_action(&save_action);

        // app.refresh
        let refresh = refresh_list.clone();
        let app_c = app.clone();
        let refresh_action = SimpleAction::new("refresh", None);
        refresh_action.connect_activate(move |_, _| { refresh(); });
        app_c.add_action(&refresh_action);

        // app.quit
        let app_for_add = app.clone();
        let app_for_quit = app.clone();
        let quit_action = SimpleAction::new("quit", None);
        quit_action.connect_activate(move |_, _| { app_for_quit.quit(); });
        app_for_add.add_action(&quit_action);

        // Tools: open system applications dir
        let app_for_add = app.clone();
        let open_sys = SimpleAction::new("open_system_dir", None);
        let win_err = win.clone();
        open_sys.connect_activate(move |_, _| {
            #[cfg(target_os = "linux")]
            {
                let path = std::path::Path::new("/usr/share/applications");
                if let Err(e) = open::that(path) {
                    show_error(&win_err, &format!("Failed to open system dir: {}", e));
                }
            }
        });
        app_for_add.add_action(&open_sys);

        // Tools: open user applications dir
        let app_for_add = app.clone();
        let open_user = SimpleAction::new("open_user_dir", None);
        let win_err2 = win.clone();
        open_user.connect_activate(move |_, _| {
            #[cfg(target_os = "linux")]
            {
                if let Some(base) = directories::BaseDirs::new() {
                    let path = base.data_dir().join("applications");
                    if let Err(e) = open::that(&path) {
                        show_error(&win_err2, &format!("Failed to open user dir: {}", e));
                    }
                } else {
                    show_error(&win_err2, "Cannot resolve user data dir");
                }
            }
        });
        app_for_add.add_action(&open_user);

        // Help: About dialog
        let app_for_add = app.clone();
        let about = SimpleAction::new("about", None);
        let win_for_about = win.clone();
        about.connect_activate(move |_, _| {
            let about_win = AboutWindow::builder()
                .transient_for(&win_for_about)
                .modal(true)
                .application_name("Desktop Entry Manager")
                .developer_name("Arnaud Michel")
                .version(env!("CARGO_PKG_VERSION"))
                .website("https://github.com/")
                .issue_url("https://github.com/")
                .build();
            about_win.present();
        });
        app_for_add.add_action(&about);

        // Credits: simple dialog
        let app_for_add = app.clone();
        let credits = SimpleAction::new("credits", None);
        let win_for_credits = win.clone();
        credits.connect_activate(move |_, _| {
            let text = "Desktop Entry Manager\n\nCredits:\n- Author: Arnaud Michel\n- UI: GTK4 + Libadwaita";
            let dialog = gtk4::MessageDialog::builder()
                .transient_for(&win_for_credits)
                .modal(true)
                .title("Credits")
                .text("Thanks for using Desktop Entry Manager")
                .secondary_text(text)
                .build();
            dialog.add_button("Close", ResponseType::Close);
            dialog.connect_response(|d, _| d.close());
            dialog.show();
        });
        app_for_add.add_action(&credits);

        // win.toggle_fullscreen
        let win_c = win.clone();
        let toggle_fullscreen = SimpleAction::new("toggle_fullscreen", None);
        toggle_fullscreen.connect_activate(move |_, _| {
            if win_c.is_fullscreen() { win_c.unfullscreen(); } else { win_c.fullscreen(); }
        });
        win.add_action(&toggle_fullscreen);
    }

    // Initial population
    refresh_list();

    // Delete handler
    {
        use std::fs;
        let win_del = win.clone();
        let state_del = state.clone();
        let set_form = set_form_from_entry.clone();
        let status_label_del = status_label.clone();
        let refresh = refresh_list.clone();
        delete_btn.connect_clicked(move |_| {
            let maybe_path = state_del.borrow().selected_path.clone();
            if let Some(path) = maybe_path {
                let dialog = gtk4::MessageDialog::builder()
                    .transient_for(&win_del)
                    .modal(true)
                    .title("Confirm deletion")
                    .text("Delete selected .desktop file?")
                    .secondary_text(&format!("This will permanently remove:\n{}", path.display()))
                    .build();
                dialog.add_button("Cancel", ResponseType::Cancel);
                dialog.add_button("Delete", ResponseType::Accept);
                let win_del_c = win_del.clone();
                let set_form_c = set_form.clone();
                let state_del_c = state_del.clone();
                let refresh_c = refresh.clone();
                let status_label_del_c = status_label_del.clone();
                dialog.connect_response(move |d, resp| {
                    if resp == ResponseType::Accept {
                        if let Err(e) = fs::remove_file(&path) {
                            let err = format!("Failed to delete: {}", e);
                            show_error(&win_del_c, &err);
                        } else {
                            // Clear form
                            set_form_c(&DesktopEntry { name: String::new(), type_field: "Application".into(), ..Default::default() });
                            // Reset selection
                            state_del_c.borrow_mut().selected_path = None;
                            // Refresh list
                            refresh_c();
                            // Update status
                            status_label_del_c.set_text("Deleted");
                        }
                    }
                    d.close();
                });
                dialog.show();
            } else {
                // No selected file
                show_error(&win_del, "No file selected to delete");
            }
        });
    }

    // Preview handler
    let type_combo_preview = type_combo.clone();
    let name_entry_preview = name_entry.clone();
    let generic_name_entry_preview = generic_name_entry.clone();
    let comment_entry_preview = comment_entry.clone();
    let exec_entry_preview = exec_entry.clone();
    let icon_entry_preview = icon_entry.clone();
    let terminal_check_preview = terminal_check.clone();
    let nodisplay_check_preview = nodisplay_check.clone();
    let startup_check_preview = startup_check.clone();
    let categories_entry_preview = categories_entry.clone();
    let mimetype_entry_preview = mimetype_entry.clone();
    let keywords_entry_preview = keywords_entry.clone();
    let onlyshowin_entry_preview = onlyshowin_entry.clone();
    let notshowin_entry_preview = notshowin_entry.clone();
    let tryexec_entry_preview = tryexec_entry.clone();
    let path_entry_preview = path_entry.clone();
    let url_entry_preview = url_entry.clone();
    let actions_entry_preview = actions_entry.clone();
    let localized_name_preview = localized_name.clone();
    let localized_gname_preview = localized_gname.clone();
    let localized_comment_preview = localized_comment.clone();
    let extra_kv_preview = extra_kv.clone();
    let win_preview = win.clone();
    preview_btn.connect_clicked(move |_| {
        let entry = crate::ui::helpers::collect_entry(
            &type_combo_preview, &name_entry_preview, &generic_name_entry_preview, &comment_entry_preview, &exec_entry_preview, &icon_entry_preview,
            &terminal_check_preview, &nodisplay_check_preview, &startup_check_preview, &categories_entry_preview, &mimetype_entry_preview,
            &keywords_entry_preview, &onlyshowin_entry_preview, &notshowin_entry_preview, &tryexec_entry_preview, &path_entry_preview,
            &url_entry_preview, &actions_entry_preview, &localized_name_preview, &localized_gname_preview, &localized_comment_preview, &extra_kv_preview
        );
        match entry {
            Ok(de) => {
                let content = de.to_ini_string();
                let dialog = gtk4::MessageDialog::builder()
                    .transient_for(&win_preview)
                    .modal(true)
                    .title("Preview .desktop")
                    .text("This is the generated .desktop content:")
                    .secondary_text(&content)
                    .build();
                dialog.add_button("Close", ResponseType::Close);
                dialog.connect_response(|d, _| d.close());
                dialog.show();
            }
            Err(err) => show_error(&win_preview, &err)
        }
    });

    // Save handler
    let type_combo_save = type_combo.clone();
    let name_entry_save = name_entry.clone();
    let generic_name_entry_save = generic_name_entry.clone();
    let comment_entry_save = comment_entry.clone();
    let exec_entry_save = exec_entry.clone();
    let icon_entry_save = icon_entry.clone();
    let terminal_check_save = terminal_check.clone();
    let nodisplay_check_save = nodisplay_check.clone();
    let startup_check_save = startup_check.clone();
    let categories_entry_save = categories_entry.clone();
    let mimetype_entry_save = mimetype_entry.clone();
    let keywords_entry_save = keywords_entry.clone();
    let onlyshowin_entry_save = onlyshowin_entry.clone();
    let notshowin_entry_save = notshowin_entry.clone();
    let tryexec_entry_save = tryexec_entry.clone();
    let path_entry_save = path_entry.clone();
    let url_entry_save = url_entry.clone();
    let actions_entry_save = actions_entry.clone();
    let localized_name_save = localized_name.clone();
    let localized_gname_save = localized_gname.clone();
    let localized_comment_save = localized_comment.clone();
    let extra_kv_save = extra_kv.clone();
    let win_save = win.clone();
    save_btn.connect_clicked(move |_| {
        let entry = crate::ui::helpers::collect_entry(
            &type_combo_save, &name_entry_save, &generic_name_entry_save, &comment_entry_save, &exec_entry_save, &icon_entry_save,
            &terminal_check_save, &nodisplay_check_save, &startup_check_save, &categories_entry_save, &mimetype_entry_save,
            &keywords_entry_save, &onlyshowin_entry_save, &notshowin_entry_save, &tryexec_entry_save, &path_entry_save,
            &url_entry_save, &actions_entry_save, &localized_name_save, &localized_gname_save, &localized_comment_save, &extra_kv_save
        );
        match entry {
            Ok(de) => {
                let file_name = if !de.name.trim().is_empty() { de.name.clone() } else { "desktop-entry".into() };
                match DesktopWriter::write(&de, &file_name, true) {
                    Ok(path) => {
                        let dialog = gtk4::MessageDialog::builder()
                            .transient_for(&win_save)
                            .modal(true)
                            .title("Saved")
                            .text(".desktop file created")
                            .secondary_text(&format!("Saved to {}", path.display()))
                            .build();
                        dialog.add_button("Open Folder", ResponseType::Accept);
                        dialog.add_button("Close", ResponseType::Close);
                        dialog.connect_response(move |d, resp| {
                            if resp == ResponseType::Accept {
                                #[cfg(target_os = "linux")]
                                {
                                    if let Some(parent) = path.parent() { let _ = open::that(parent); }
                                }
                            }
                            d.close();
                        });
                        dialog.show();
                    }
                    Err(err) => show_error(&win_save, &err.to_string()),
                }
            }
            Err(err) => show_error(&win_save, &err)
        }
    });

    win.present();
}



fn show_error<W: IsA<gtk4::Window>>(parent: &W, msg: &str) {
    let dialog = gtk4::MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Error")
        .text("Operation failed")
        .secondary_text(msg)
        .build();
    dialog.add_button("Close", ResponseType::Close);
    dialog.connect_response(|d, _| d.close());
    dialog.show();
}

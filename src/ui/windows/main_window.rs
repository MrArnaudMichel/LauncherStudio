use gtk4::prelude::*;
use gtk4::{self, Align, Application, ApplicationWindow, Box as GtkBox, Button, CheckButton, ComboBoxText, Entry, FileChooserAction, FileChooserDialog, HeaderBar, Orientation, ResponseType, ScrolledWindow, TextView, Notebook, ListBox, ListBoxRow, Label, PopoverMenuBar, Image};
use gtk4::prelude::FileExt;
use gtk4::gio::{Menu, SimpleAction};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::domain::desktop_entry::DesktopEntry;
use crate::services::desktop_writer::DesktopWriter;
use crate::services::desktop_reader::DesktopReader;

pub fn show_main_window(app: &Application) {
    let win = ApplicationWindow::builder()
        .application(app)
        .title("Desktop Entry Manager")
        .default_width(1000)
        .default_height(700)
        .resizable(true)
        .build();

    // App icon (asset renamed in repo to custom-app-icon.png)
    win.set_icon_name(Some("assets/custom-app-icon.png"));

    // Header bar stays minimal (title), we build our own menu + toolbar below
    let header = HeaderBar::new();
    win.set_titlebar(Some(&header));

    let root = GtkBox::new(Orientation::Vertical, 6);
    root.set_margin_top(12);
    root.set_margin_bottom(12);
    root.set_margin_start(12);
    root.set_margin_end(12);

    // Top Menu Bar (PopoverMenuBar with gio::Menu)
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

    let menubar = PopoverMenuBar::from_model(Some(&menu_model));

    // Toolbar with icons: New, Open, Save, Refresh (icon-only buttons)
    let toolbar = GtkBox::new(Orientation::Horizontal, 6);

    let btn_new = Button::new();
    let img_new = Image::from_icon_name("list-add-symbolic");
    img_new.set_pixel_size(18);
    btn_new.set_child(Some(&img_new));
    btn_new.set_tooltip_text(Some("New .desktop"));

    let btn_open = Button::new();
    let img_open = Image::from_icon_name("document-open-symbolic");
    img_open.set_pixel_size(18);
    btn_open.set_child(Some(&img_open));
    btn_open.set_tooltip_text(Some("Open"));

    let btn_save = Button::new();
    let img_save = Image::from_icon_name("document-save-symbolic");
    img_save.set_pixel_size(18);
    btn_save.set_child(Some(&img_save));
    btn_save.set_tooltip_text(Some("Save"));

    let btn_refresh = Button::new();
    let img_refresh = Image::from_icon_name("view-refresh-symbolic");
    img_refresh.set_pixel_size(18);
    btn_refresh.set_child(Some(&img_refresh));
    btn_refresh.set_tooltip_text(Some("Refresh"));

    toolbar.append(&btn_new);
    toolbar.append(&btn_open);
    toolbar.append(&btn_save);
    toolbar.append(&btn_refresh);

    // Main area: sidebar + editor tabs
    let main_area = GtkBox::new(Orientation::Horizontal, 12);

    // Sidebar list
    let sidebar_scroller = ScrolledWindow::builder().min_content_width(240).vexpand(true).build();
    let listbox = ListBox::new();
    sidebar_scroller.set_child(Some(&listbox));

    // Editor: Notebook with Basic / Advanced / Source
    let notebook = Notebook::new();

    let basic_box = GtkBox::new(Orientation::Vertical, 8);
    let advanced_box = GtkBox::new(Orientation::Vertical, 8);
    let source_view = TextView::new();
    source_view.set_monospace(true);

    // Bottom status bar
    let status_bar = GtkBox::new(Orientation::Horizontal, 6);
    let status_label = Label::new(Some("No file selected"));
    status_label.set_xalign(0.0);
    status_bar.append(&status_label);

    // Form containers (old form split will be inserted into basic/advanced below)
    let scroller = ScrolledWindow::builder().hexpand(true).vexpand(true).build();
    let form = GtkBox::new(Orientation::Vertical, 8);

    // Basic fields
    let (name_row, name_entry) = crate::ui::components::labeled_entry("Name*");
    let (generic_name_row, generic_name_entry) = crate::ui::components::labeled_entry("Generic Name");
    let (comment_row, comment_entry) = crate::ui::components::labeled_entry("Comment");
    let (exec_row, exec_entry) = crate::ui::components::labeled_entry("Exec*");
    let (icon_row, icon_entry) = crate::ui::components::labeled_entry("Icon");

    // File chooser buttons for Exec and Icon
    let exec_btn = Button::with_label("Select...");
    let win_c = win.clone();
    let exec_entry_c = exec_entry.clone();
    exec_btn.connect_clicked(move |_| {
        let dialog = FileChooserDialog::new(Some("Select Executable"), Some(&win_c), FileChooserAction::Open, &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)]);
        let exec_entry_c2 = exec_entry_c.clone();
        dialog.connect_response(move |d, resp| {
            if resp == ResponseType::Accept {
                if let Some(file) = d.file() { if let Some(path) = file.path() { exec_entry_c2.set_text(path.to_string_lossy().as_ref()); } }
            }
            d.close();
        });
        dialog.show();
    });
    exec_row.append(&exec_btn);

    let icon_btn = Button::with_label("Select...");
    let win_c2 = win.clone();
    let icon_entry_c = icon_entry.clone();
    icon_btn.connect_clicked(move |_| {
        let dialog = FileChooserDialog::new(Some("Select Icon"), Some(&win_c2), FileChooserAction::Open, &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)]);
        let icon_entry_c2 = icon_entry_c.clone();
        dialog.connect_response(move |d, resp| {
            if resp == ResponseType::Accept {
                if let Some(file) = d.file() { if let Some(path) = file.path() { icon_entry_c2.set_text(path.to_string_lossy().as_ref()); } }
            }
            d.close();
        });
        dialog.show();
    });
    icon_row.append(&icon_btn);

    // Type
    let type_row = GtkBox::new(Orientation::Horizontal, 8);
    let type_label = gtk4::Label::new(Some("Type*"));
    type_label.set_halign(Align::End);
    type_label.set_xalign(1.0);
    type_label.set_width_chars(18);
    let type_combo = ComboBoxText::new();
    type_combo.append_text("Application");
    type_combo.append_text("Link");
    type_combo.append_text("Directory");
    type_combo.set_active(Some(0));
    type_row.append(&type_label);
    type_row.append(&type_combo);

    // Checkbuttons
    let terminal_check = CheckButton::with_label("Run in Terminal");
    let nodisplay_check = CheckButton::with_label("NoDisplay");
    let startup_check = CheckButton::with_label("StartupNotify");

    // List-like entries
    let (categories_row, categories_entry) = crate::ui::components::labeled_entry("Categories (;) ");
    let (mimetype_row, mimetype_entry) = crate::ui::components::labeled_entry("MimeType (;) ");
    let (keywords_row, keywords_entry) = crate::ui::components::labeled_entry("Keywords (;) ");
    let (onlyshowin_row, onlyshowin_entry) = crate::ui::components::labeled_entry("OnlyShowIn (;) ");
    let (notshowin_row, notshowin_entry) = crate::ui::components::labeled_entry("NotShowIn (;) ");

    // Optional fields
    let (tryexec_row, tryexec_entry) = crate::ui::components::labeled_entry("TryExec");
    let (path_row, path_entry) = crate::ui::components::labeled_entry("Working Dir (Path)");
    let (url_row, url_entry) = crate::ui::components::labeled_entry("URL (Type=Link)");

    // Localized Name/Comment/GenericName additions (simple: text area specifying lang=value per line)
    let localized_label = gtk4::Label::new(Some("Localized fields (one per line: lang=value). Keys: Name, GenericName, Comment. Example: fr=Mon App"));
    localized_label.set_wrap(true);
    let localized_name = TextView::new();
    localized_name.set_monospace(true);
    localized_name.set_size_request(-1, 60);
    let localized_gname = TextView::new();
    localized_gname.set_monospace(true);
    localized_gname.set_size_request(-1, 60);
    let localized_comment = TextView::new();
    localized_comment.set_monospace(true);
    localized_comment.set_size_request(-1, 60);

    let (actions_row, actions_entry) = crate::ui::components::labeled_entry("Actions (names;)");

    // Extra key=value text area
    let extra_label = gtk4::Label::new(Some("Extra key=value lines (advanced)"));
    extra_label.set_wrap(true);
    let extra_kv = TextView::new();
    extra_kv.set_monospace(true);
    extra_kv.set_size_request(-1, 120);

    // Filename entry
    let (filename_row, filename_entry) = crate::ui::components::labeled_entry("File name (without .desktop)");

    // Buttons
    let buttons = GtkBox::new(Orientation::Horizontal, 8);
    buttons.set_halign(Align::End);
    let preview_btn = Button::with_label("Preview");
    let save_btn = Button::with_label("Save .desktop");
    buttons.append(&preview_btn);
    buttons.append(&save_btn);

    // Assemble form into tabs
    // Basic tab minimal fields
    basic_box.append(&type_row);
    basic_box.append(&name_row);
    basic_box.append(&exec_row);
    basic_box.append(&icon_row);
    basic_box.append(&terminal_check);

    // Advanced tab
    advanced_box.append(&generic_name_row);
    advanced_box.append(&comment_row);
    advanced_box.append(&nodisplay_check);
    advanced_box.append(&startup_check);
    advanced_box.append(&categories_row);
    advanced_box.append(&mimetype_row);
    advanced_box.append(&keywords_row);
    advanced_box.append(&onlyshowin_row);
    advanced_box.append(&notshowin_row);
    advanced_box.append(&tryexec_row);
    advanced_box.append(&path_row);
    advanced_box.append(&url_row);

    // Localized sections
    advanced_box.append(&localized_label);
    advanced_box.append(&gtk4::Label::new(Some("Name[lang]=value lines")));
    advanced_box.append(&localized_name);
    advanced_box.append(&gtk4::Label::new(Some("GenericName[lang]=value lines")));
    advanced_box.append(&localized_gname);
    advanced_box.append(&gtk4::Label::new(Some("Comment[lang]=value lines")));
    advanced_box.append(&localized_comment);

    advanced_box.append(&actions_row);
    advanced_box.append(&extra_label);
    advanced_box.append(&extra_kv);
    advanced_box.append(&filename_row);

    // Notebook assembly
    let basic_label = gtk4::Label::new(Some("Basic"));
    let advanced_label = gtk4::Label::new(Some("Advanced"));
    let source_label = gtk4::Label::new(Some("Source"));
    let basic_scroll = ScrolledWindow::builder().hexpand(true).vexpand(true).build();
    basic_scroll.set_child(Some(&basic_box));
    let adv_scroll = ScrolledWindow::builder().hexpand(true).vexpand(true).build();
    adv_scroll.set_child(Some(&advanced_box));
    let source_scroll = ScrolledWindow::builder().hexpand(true).vexpand(true).build();
    source_scroll.set_child(Some(&source_view));
    notebook.append_page(&basic_scroll, Some(&basic_label));
    notebook.append_page(&adv_scroll, Some(&advanced_label));
    notebook.append_page(&source_scroll, Some(&source_label));

    scroller.set_child(Some(&notebook));

    // Main area composition
    main_area.append(&sidebar_scroller);
    main_area.append(&scroller);

    // Build root
    root.append(&menubar);
    root.append(&toolbar);
    root.append(&main_area);
    root.append(&buttons);
    root.append(&status_bar);

    win.set_child(Some(&root));

    // Simple app state
    #[derive(Default, Clone)]
    struct UiState { selected_path: Option<PathBuf> }
    let state = Rc::new(RefCell::new(UiState::default()));

    // Helpers
    let update_source = {
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
        let source_view = source_view.clone();
        move || {
            if let Ok(de) = collect_entry(&type_combo, &name_entry, &generic_name_entry, &comment_entry, &exec_entry, &icon_entry, &terminal_check, &nodisplay_check, &startup_check, &categories_entry, &mimetype_entry, &keywords_entry, &onlyshowin_entry, &notshowin_entry, &tryexec_entry, &path_entry, &url_entry, &actions_entry, &localized_name, &localized_gname, &localized_comment, &extra_kv) {
                let content = de.to_ini_string();
                let buf = source_view.buffer();
                buf.set_text(&content);
            }
        }
    };

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
            match collect_entry(&type_combo, &name_entry, &generic_name_entry, &comment_entry, &exec_entry, &icon_entry, &terminal_check, &nodisplay_check, &startup_check, &categories_entry, &mimetype_entry, &keywords_entry, &onlyshowin_entry, &notshowin_entry, &tryexec_entry, &path_entry, &url_entry, &actions_entry, &localized_name, &localized_gname, &localized_comment, &extra_kv) {
                Ok(de) => {
                    let fname = if !de.name.trim().is_empty() { de.name.clone() } else { "desktop-entry".into() };
                    match DesktopWriter::write(&de, &fname, false) {
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
            match collect_entry(&type_combo, &name_entry, &generic_name_entry, &comment_entry, &exec_entry, &icon_entry, &terminal_check, &nodisplay_check, &startup_check, &categories_entry, &mimetype_entry, &keywords_entry, &onlyshowin_entry, &notshowin_entry, &tryexec_entry, &path_entry, &url_entry, &actions_entry, &localized_name, &localized_gname, &localized_comment, &extra_kv) {
                Ok(de) => {
                    let fname = if !de.name.trim().is_empty() { de.name.clone() } else { "desktop-entry".into() };
                    match DesktopWriter::write(&de, &fname, false) {
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
        let entry = collect_entry(
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
    let filename_entry_save = filename_entry.clone();
    let win_save = win.clone();
    save_btn.connect_clicked(move |_| {
        let entry = collect_entry(
            &type_combo_save, &name_entry_save, &generic_name_entry_save, &comment_entry_save, &exec_entry_save, &icon_entry_save,
            &terminal_check_save, &nodisplay_check_save, &startup_check_save, &categories_entry_save, &mimetype_entry_save,
            &keywords_entry_save, &onlyshowin_entry_save, &notshowin_entry_save, &tryexec_entry_save, &path_entry_save,
            &url_entry_save, &actions_entry_save, &localized_name_save, &localized_gname_save, &localized_comment_save, &extra_kv_save
        );
        let file_name = filename_entry_save.text().to_string();
        match entry {
            Ok(de) => {
                match DesktopWriter::write(&de, &file_name, false) {
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

fn collect_entry(
    type_combo: &ComboBoxText,
    name_entry: &Entry,
    generic_name_entry: &Entry,
    comment_entry: &Entry,
    exec_entry: &Entry,
    icon_entry: &Entry,
    terminal_check: &CheckButton,
    nodisplay_check: &CheckButton,
    startup_check: &CheckButton,
    categories_entry: &Entry,
    mimetype_entry: &Entry,
    keywords_entry: &Entry,
    onlyshowin_entry: &Entry,
    notshowin_entry: &Entry,
    tryexec_entry: &Entry,
    path_entry: &Entry,
    url_entry: &Entry,
    actions_entry: &Entry,
    localized_name: &TextView,
    localized_gname: &TextView,
    localized_comment: &TextView,
    extra_kv: &TextView,
) -> Result<DesktopEntry, String> {
    let type_field = type_combo.active_text().map(|s| s.to_string()).unwrap_or_else(|| "Application".into());
    let name = name_entry.text().to_string();
    let generic_name = opt_text(generic_name_entry);
    let comment = opt_text(comment_entry);
    let exec = exec_entry.text().to_string();
    let icon = opt_text(icon_entry);
    let terminal = terminal_check.is_active();
    let no_display = nodisplay_check.is_active();
    let startup_notify = startup_check.is_active();
    let categories = split_semicolon(categories_entry);
    let mime_type = split_semicolon(mimetype_entry);
    let keywords = split_semicolon(keywords_entry);
    let only_show_in = split_semicolon(onlyshowin_entry);
    let not_show_in = split_semicolon(notshowin_entry);
    let try_exec = opt_text(tryexec_entry);
    let path = opt_text(path_entry);
    let url = opt_text(url_entry);
    let actions = split_semicolon(actions_entry);
    let name_localized = parse_lang_lines(&buffer_text(localized_name));
    let generic_name_localized = parse_lang_lines(&buffer_text(localized_gname));
    let comment_localized = parse_lang_lines(&buffer_text(localized_comment));
    let extra = parse_kv_lines(&buffer_text(extra_kv));

    let de = DesktopEntry {
        type_field,
        name,
        generic_name,
        comment,
        exec,
        icon,
        terminal,
        categories,
        mime_type,
        keywords,
        only_show_in,
        not_show_in,
        no_display,
        startup_notify,
        try_exec,
        path,
        url,
        actions,
        extra,
        name_localized,
        generic_name_localized,
        comment_localized,
    };

    de.validate()?;
    Ok(de)
}

fn split_semicolon(e: &Entry) -> Vec<String> { e.text().split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect() }
fn opt_text(e: &Entry) -> Option<String> { let s = e.text().trim().to_string(); if s.is_empty() { None } else { Some(s) } }

fn buffer_text(tv: &TextView) -> String {
    let buf = tv.buffer();
    buf.text(&buf.start_iter(), &buf.end_iter(), true).to_string()
}

fn parse_lang_lines(s: &str) -> Vec<(String, String)> {
    // Lines in form: lang=value ; ignore empty lines
    s.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() { return None; }
            if let Some((lang, val)) = line.split_once('=') {
                let lang = lang.trim().to_string();
                let val = val.trim().to_string();
                if lang.is_empty() || val.is_empty() { None } else { Some((lang, val)) }
            } else { None }
        })
        .collect()
}

fn parse_kv_lines(s: &str) -> Vec<(String, String)> {
    s.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() { return None; }
            if let Some((k, v)) = line.split_once('=') {
                let k = k.trim().to_string();
                let v = v.trim().to_string();
                if k.is_empty() { None } else { Some((k, v)) }
            } else { None }
        })
        .collect()
}

fn show_error(parent: &ApplicationWindow, msg: &str) {
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

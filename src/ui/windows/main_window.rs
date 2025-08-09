use gtk4::prelude::*;
use gtk4::{self, Align, Application, ApplicationWindow, Box as GtkBox, Button, CheckButton, ComboBoxText, Entry, FileChooserAction, FileChooserDialog, HeaderBar, Orientation, ResponseType, ScrolledWindow, TextView};

use crate::domain::desktop_entry::DesktopEntry;
use crate::services::desktop_writer::DesktopWriter;

pub fn show_main_window(app: &Application) {
    let win = ApplicationWindow::builder()
        .application(app)
        .title("Desktop Entry Creator")
        .default_width(800)
        .default_height(600)
        .build();

    let header = HeaderBar::new();
    win.set_titlebar(Some(&header));

    let root = GtkBox::new(Orientation::Vertical, 12);
    root.set_margin_top(12);
    root.set_margin_bottom(12);
    root.set_margin_start(12);
    root.set_margin_end(12);

    // Form container with scrolling
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
    type_label.set_xalign(0.0);
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

    // Assemble form
    form.append(&type_row);
    form.append(&name_row);
    form.append(&generic_name_row);
    form.append(&comment_row);
    form.append(&exec_row);
    form.append(&icon_row);
    form.append(&terminal_check);
    form.append(&nodisplay_check);
    form.append(&startup_check);
    form.append(&categories_row);
    form.append(&mimetype_row);
    form.append(&keywords_row);
    form.append(&onlyshowin_row);
    form.append(&notshowin_row);
    form.append(&tryexec_row);
    form.append(&path_row);
    form.append(&url_row);

    // Localized sections
    form.append(&localized_label);
    form.append(&gtk4::Label::new(Some("Name[lang]=value lines")));
    form.append(&localized_name);
    form.append(&gtk4::Label::new(Some("GenericName[lang]=value lines")));
    form.append(&localized_gname);
    form.append(&gtk4::Label::new(Some("Comment[lang]=value lines")));
    form.append(&localized_comment);

    form.append(&actions_row);
    form.append(&extra_label);
    form.append(&extra_kv);
    form.append(&filename_row);

    scroller.set_child(Some(&form));

    root.append(&scroller);
    root.append(&buttons);

    win.set_child(Some(&root));

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

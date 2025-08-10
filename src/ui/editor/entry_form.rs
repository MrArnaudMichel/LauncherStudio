use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, Entry, CheckButton, ComboBoxText, TextView, Notebook, ScrolledWindow, Button, FileChooserDialog, FileChooserAction};
use gtk4::{EntryIconPosition};
use gtk4::gio::File;
use gtk4::gdk;

use crate::domain::desktop_entry::DesktopEntry;

#[derive(Clone)]
pub struct EntryWidgets {
    pub type_combo: ComboBoxText,
    pub name_entry: Entry,
    pub generic_name_entry: Entry,
    pub comment_entry: Entry,
    pub exec_entry: Entry,
    pub icon_entry: Entry,
    pub terminal_check: CheckButton,
    pub nodisplay_check: CheckButton,
    pub startup_check: CheckButton,
    pub categories_entry: Entry,
    pub mimetype_entry: Entry,
    pub keywords_entry: Entry,
    pub onlyshowin_entry: Entry,
    pub notshowin_entry: Entry,
    pub tryexec_entry: Entry,
    pub path_entry: Entry,
    pub url_entry: Entry,
    pub actions_entry: Entry,
    pub localized_name: TextView,
    pub localized_gname: TextView,
    pub localized_comment: TextView,
    pub extra_kv: TextView,
}

pub struct Editor {
    pub notebook: Notebook,
    pub source_view: TextView,
    pub widgets: EntryWidgets,
}

pub fn build_editor() -> Editor {
    // Notebook and tab containers with padding
    let notebook = Notebook::new();

    let basic_box = GtkBox::new(Orientation::Vertical, 8);
    basic_box.set_margin_top(12);
    basic_box.set_margin_bottom(12);
    basic_box.set_margin_start(12);
    basic_box.set_margin_end(12);

    let advanced_box = GtkBox::new(Orientation::Vertical, 8);
    advanced_box.set_margin_top(12);
    advanced_box.set_margin_bottom(12);
    advanced_box.set_margin_start(12);
    advanced_box.set_margin_end(12);

    let source_view = TextView::new();
    source_view.set_monospace(true);
    source_view.set_margin_top(12);
    source_view.set_margin_bottom(12);
    source_view.set_margin_start(12);
    source_view.set_margin_end(12);

    // Basic fields
    let type_row = GtkBox::new(Orientation::Horizontal, 8);
    let type_label = Label::new(Some("Type*"));
    type_label.set_halign(gtk4::Align::End);
    type_label.set_xalign(1.0);
    type_label.set_width_chars(18);
    let type_combo = ComboBoxText::new();
    type_combo.append_text("Application");
    type_combo.append_text("Link");
    type_combo.append_text("Directory");
    type_combo.set_active(Some(0));
    type_row.append(&type_label);
    type_row.append(&type_combo);

    let (name_row, name_entry) = crate::ui::components::labeled_entry("Name*");
    let (generic_name_row, generic_name_entry) = crate::ui::components::labeled_entry("Generic Name");
    let (comment_row, comment_entry) = crate::ui::components::labeled_entry("Comment");

    // Exec
    let exec_row = GtkBox::new(Orientation::Horizontal, 8);
    let exec_lbl = Label::new(Some("Exec*"));
    exec_lbl.set_halign(gtk4::Align::End);
    exec_lbl.set_xalign(1.0);
    exec_lbl.set_width_chars(18);
    let exec_entry = Entry::new();
    exec_entry.set_hexpand(true);
    exec_entry.set_icon_from_icon_name(EntryIconPosition::Primary, Some("application-x-executable-symbolic"));
    exec_row.append(&exec_lbl);
    exec_row.append(&exec_entry);
    // File chooser for Exec
    let exec_btn = Button::with_label("Select...");
    exec_btn.connect_clicked({
        let exec_entry_c = exec_entry.clone();
        move |_| {
            let dialog = FileChooserDialog::new(Some("Select Executable"), None::<&gtk4::ApplicationWindow>, FileChooserAction::Open, &[("Cancel", gtk4::ResponseType::Cancel), ("Open", gtk4::ResponseType::Accept)]);
            let exec_entry_c2 = exec_entry_c.clone();
            dialog.connect_response(move |d, resp| {
                if resp == gtk4::ResponseType::Accept {
                    if let Some(file) = d.file() { if let Some(path) = file.path() { exec_entry_c2.set_text(path.to_string_lossy().as_ref()); } }
                }
                d.close();
            });
            dialog.show();
        }
    });
    exec_row.append(&exec_btn);

    // Icon with dynamic preview inside entry (left)
    let icon_row = GtkBox::new(Orientation::Horizontal, 8);
    let icon_lbl = Label::new(Some("Icon"));
    icon_lbl.set_halign(gtk4::Align::End);
    icon_lbl.set_xalign(1.0);
    icon_lbl.set_width_chars(18);
    let icon_entry = Entry::new();
    icon_entry.set_hexpand(true);
    icon_entry.set_icon_from_icon_name(EntryIconPosition::Primary, Some("image-missing"));
    icon_row.append(&icon_lbl);
    icon_row.append(&icon_entry);
    // File chooser for Icon
    let icon_btn = Button::with_label("Select...");
    icon_btn.connect_clicked({
        let icon_entry_c = icon_entry.clone();
        move |_| {
            let dialog = FileChooserDialog::new(Some("Select Icon"), None::<&gtk4::ApplicationWindow>, FileChooserAction::Open, &[("Cancel", gtk4::ResponseType::Cancel), ("Open", gtk4::ResponseType::Accept)]);
            let icon_entry_c2 = icon_entry_c.clone();
            dialog.connect_response(move |d, resp| {
                if resp == gtk4::ResponseType::Accept {
                    if let Some(file) = d.file() { if let Some(path) = file.path() {
                        let txt = path.to_string_lossy().to_string();
                        icon_entry_c2.set_text(&txt);
                    } }
                }
                d.close();
            });
            dialog.show();
        }
    });
    icon_row.append(&icon_btn);

    {
        icon_entry.connect_changed(move |e| {
            let txt = e.text().to_string();
            if txt.trim().is_empty() {
                e.set_icon_from_icon_name(EntryIconPosition::Primary, Some("image-missing"));
            } else if txt.contains('/') {
                let f = File::for_path(&txt);
                match gdk::Texture::from_file(&f) {
                    Ok(tex) => e.set_icon_from_paintable(EntryIconPosition::Primary, Some(&tex)),
                    Err(_) => e.set_icon_from_icon_name(EntryIconPosition::Primary, Some("image-missing")),
                }
            } else {
                e.set_icon_from_icon_name(EntryIconPosition::Primary, Some(&txt));
            }
        });
    }

    // Checkbuttons (aligned)
    let terminal_check = CheckButton::with_label("Run in Terminal");
    let terminal_row = GtkBox::new(Orientation::Horizontal, 8);
    let terminal_spacer = Label::new(None);
    terminal_spacer.set_halign(gtk4::Align::End);
    terminal_spacer.set_xalign(1.0);
    terminal_spacer.set_width_chars(18);
    terminal_row.append(&terminal_spacer);
    terminal_row.append(&terminal_check);

    let nodisplay_check = CheckButton::with_label("NoDisplay");
    let nodisplay_row = GtkBox::new(Orientation::Horizontal, 8);
    let nodisplay_spacer = Label::new(None);
    nodisplay_spacer.set_halign(gtk4::Align::End);
    nodisplay_spacer.set_xalign(1.0);
    nodisplay_spacer.set_width_chars(18);
    nodisplay_row.append(&nodisplay_spacer);
    nodisplay_row.append(&nodisplay_check);

    let startup_check = CheckButton::with_label("StartupNotify");
    let startup_row = GtkBox::new(Orientation::Horizontal, 8);
    let startup_spacer = Label::new(None);
    startup_spacer.set_halign(gtk4::Align::End);
    startup_spacer.set_xalign(1.0);
    startup_spacer.set_width_chars(18);
    startup_row.append(&startup_spacer);
    startup_row.append(&startup_check);

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

    // Localized fields and extras
    let localized_label = Label::new(Some("Localized fields (one per line: lang=value). Keys: Name, GenericName, Comment. Example: fr=Mon App"));
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

    let extra_label = Label::new(Some("Extra key=value lines (advanced)"));
    extra_label.set_wrap(true);
    let extra_kv = TextView::new();
    extra_kv.set_monospace(true);
    extra_kv.set_size_request(-1, 120);

    // Basic tab
    basic_box.append(&type_row);
    basic_box.append(&name_row);
    basic_box.append(&exec_row);
    basic_box.append(&icon_row);
    basic_box.append(&terminal_row);

    // Advanced tab
    advanced_box.append(&generic_name_row);
    advanced_box.append(&comment_row);
    advanced_box.append(&nodisplay_row);
    advanced_box.append(&startup_row);
    advanced_box.append(&categories_row);
    advanced_box.append(&mimetype_row);
    advanced_box.append(&keywords_row);
    advanced_box.append(&onlyshowin_row);
    advanced_box.append(&notshowin_row);
    advanced_box.append(&tryexec_row);
    advanced_box.append(&path_row);
    advanced_box.append(&url_row);

    advanced_box.append(&localized_label);
    advanced_box.append(&Label::new(Some("Name[lang]=value lines")));
    let localized_name_sw = ScrolledWindow::builder().hexpand(true).vexpand(false).build();
    localized_name_sw.add_css_class("frame");
    localized_name_sw.set_child(Some(&localized_name));
    advanced_box.append(&localized_name_sw);
    advanced_box.append(&Label::new(Some("GenericName[lang]=value lines")));
    let localized_gname_sw = ScrolledWindow::builder().hexpand(true).vexpand(false).build();
    localized_gname_sw.add_css_class("frame");
    localized_gname_sw.set_child(Some(&localized_gname));
    advanced_box.append(&localized_gname_sw);
    advanced_box.append(&Label::new(Some("Comment[lang]=value lines")));
    let localized_comment_sw = ScrolledWindow::builder().hexpand(true).vexpand(false).build();
    localized_comment_sw.add_css_class("frame");
    localized_comment_sw.set_child(Some(&localized_comment));
    advanced_box.append(&localized_comment_sw);

    advanced_box.append(&actions_row);
    advanced_box.append(&extra_label);
    let extra_kv_sw = ScrolledWindow::builder().hexpand(true).vexpand(false).build();
    extra_kv_sw.add_css_class("frame");
    extra_kv_sw.set_child(Some(&extra_kv));
    advanced_box.append(&extra_kv_sw);

    // Assemble notebook
    let basic_scroll = ScrolledWindow::builder().hexpand(true).vexpand(true).build();
    basic_scroll.set_child(Some(&basic_box));
    let adv_scroll = ScrolledWindow::builder().hexpand(true).vexpand(true).build();
    adv_scroll.set_child(Some(&advanced_box));
    let source_scroll = ScrolledWindow::builder().hexpand(true).vexpand(true).build();
    source_scroll.add_css_class("frame");
    source_scroll.set_child(Some(&source_view));

    notebook.append_page(&basic_scroll, Some(&Label::new(Some("Basic"))));
    notebook.append_page(&adv_scroll, Some(&Label::new(Some("Advanced"))));
    notebook.append_page(&source_scroll, Some(&Label::new(Some("Source"))));

    let widgets = EntryWidgets {
        type_combo,
        name_entry,
        generic_name_entry,
        comment_entry,
        exec_entry,
        icon_entry,
        terminal_check,
        nodisplay_check,
        startup_check,
        categories_entry,
        mimetype_entry,
        keywords_entry,
        onlyshowin_entry,
        notshowin_entry,
        tryexec_entry,
        path_entry,
        url_entry,
        actions_entry,
        localized_name,
        localized_gname,
        localized_comment,
        extra_kv,
    };

    Editor { notebook, source_view, widgets }
}

#[allow(dead_code)]
pub fn update_source(w: &EntryWidgets, source_view: &TextView) {
    if let Ok(de) = collect_entry(w) {
        let content = de.to_ini_string();
        let buf = source_view.buffer();
        buf.set_text(&content);
    }
}

pub fn set_form_from_entry(w: &EntryWidgets, de: &DesktopEntry) {
    // Type
    let idx = match de.type_field.as_str() { "Application" => 0, "Link" => 1, "Directory" => 2, _ => 0 };
    w.type_combo.set_active(Some(idx));
    w.name_entry.set_text(&de.name);
    w.generic_name_entry.set_text(de.generic_name.as_deref().unwrap_or(""));
    w.comment_entry.set_text(de.comment.as_deref().unwrap_or(""));
    w.exec_entry.set_text(&de.exec);
    w.icon_entry.set_text(de.icon.as_deref().unwrap_or(""));

    // Update icon preview inside entry
    let txt = w.icon_entry.text().to_string();
    if txt.trim().is_empty() {
        w.icon_entry.set_icon_from_icon_name(EntryIconPosition::Primary, Some("image-missing"));
    } else if txt.contains('/') {
        let f = File::for_path(&txt);
        match gdk::Texture::from_file(&f) {
            Ok(tex) => w.icon_entry.set_icon_from_paintable(EntryIconPosition::Primary, Some(&tex)),
            Err(_) => w.icon_entry.set_icon_from_icon_name(EntryIconPosition::Primary, Some("image-missing")),
        }
    } else {
        w.icon_entry.set_icon_from_icon_name(EntryIconPosition::Primary, Some(&txt));
    }

    w.terminal_check.set_active(de.terminal);
    w.nodisplay_check.set_active(de.no_display);
    w.startup_check.set_active(de.startup_notify);
    w.categories_entry.set_text(&de.categories.join(";"));
    w.mimetype_entry.set_text(&de.mime_type.join(";"));
    w.keywords_entry.set_text(&de.keywords.join(";"));
    w.onlyshowin_entry.set_text(&de.only_show_in.join(";"));
    w.notshowin_entry.set_text(&de.not_show_in.join(";"));
    w.tryexec_entry.set_text(de.try_exec.as_deref().unwrap_or(""));
    w.path_entry.set_text(de.path.as_deref().unwrap_or(""));
    w.url_entry.set_text(de.url.as_deref().unwrap_or(""));
    w.actions_entry.set_text(&de.actions.join(";"));

    // Localized
    let ln: Vec<String> = de.name_localized.iter().map(|(l,v)| format!("{}={}", l, v)).collect();
    let lg: Vec<String> = de.generic_name_localized.iter().map(|(l,v)| format!("{}={}", l, v)).collect();
    let lc: Vec<String> = de.comment_localized.iter().map(|(l,v)| format!("{}={}", l, v)).collect();
    w.localized_name.buffer().set_text(&ln.join("\n"));
    w.localized_gname.buffer().set_text(&lg.join("\n"));
    w.localized_comment.buffer().set_text(&lc.join("\n"));

    // Extra
    let extra: Vec<String> = de.extra.iter().map(|(k,v)| format!("{}={}", k, v)).collect();
    w.extra_kv.buffer().set_text(&extra.join("\n"));
}

pub fn collect_entry(w: &EntryWidgets) -> Result<DesktopEntry, String> {
    let type_field = w.type_combo.active_text().map(|s| s.to_string()).unwrap_or_else(|| "Application".into());
    let name = w.name_entry.text().to_string();
    let generic_name = opt_text(&w.generic_name_entry);
    let comment = opt_text(&w.comment_entry);
    let exec = w.exec_entry.text().to_string();
    let icon = opt_text(&w.icon_entry);
    let terminal = w.terminal_check.is_active();
    let no_display = w.nodisplay_check.is_active();
    let startup_notify = w.startup_check.is_active();
    let categories = split_semicolon(&w.categories_entry);
    let mime_type = split_semicolon(&w.mimetype_entry);
    let keywords = split_semicolon(&w.keywords_entry);
    let only_show_in = split_semicolon(&w.onlyshowin_entry);
    let not_show_in = split_semicolon(&w.notshowin_entry);
    let try_exec = opt_text(&w.tryexec_entry);
    let path = opt_text(&w.path_entry);
    let url = opt_text(&w.url_entry);
    let actions = split_semicolon(&w.actions_entry);
    let name_localized = parse_lang_lines(&buffer_text(&w.localized_name));
    let generic_name_localized = parse_lang_lines(&buffer_text(&w.localized_gname));
    let comment_localized = parse_lang_lines(&buffer_text(&w.localized_comment));
    let extra = parse_kv_lines(&buffer_text(&w.extra_kv));

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
fn buffer_text(tv: &TextView) -> String { let buf = tv.buffer(); buf.text(&buf.start_iter(), &buf.end_iter(), true).to_string() }
fn parse_lang_lines(s: &str) -> Vec<(String, String)> {
    s.lines().filter_map(|line| {
        let line = line.trim();
        if line.is_empty() { return None; }
        if let Some((lang, val)) = line.split_once('=') {
            let lang = lang.trim().to_string();
            let val = val.trim().to_string();
            if lang.is_empty() || val.is_empty() { None } else { Some((lang, val)) }
        } else { None }
    }).collect()
}
fn parse_kv_lines(s: &str) -> Vec<(String, String)> {
    s.lines().filter_map(|line| {
        let line = line.trim();
        if line.is_empty() { return None; }
        if let Some((k, v)) = line.split_once('=') {
            let k = k.trim().to_string();
            let v = v.trim().to_string();
            if k.is_empty() { None } else { Some((k, v)) }
        } else { None }
    }).collect()
}


// --- Added: parse from source and wire two-way sync for Source tab ---
pub fn parse_desktop_source(content: &str) -> DesktopEntry {
    let mut entry = DesktopEntry::default();
    let mut in_desktop = false;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') { continue; }
        if line.starts_with('[') && line.ends_with(']') {
            in_desktop = line == "[Desktop Entry]";
            continue;
        }
        if !in_desktop { continue; }
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim();
            let val = v.trim().to_string();
            match key {
                "Type" => entry.type_field = val,
                "Name" => entry.name = val,
                _ if key.starts_with("Name[") && key.ends_with("]") => {
                    let lang = key.trim_start_matches("Name[").trim_end_matches("]").to_string();
                    entry.name_localized.push((lang, val));
                }
                "GenericName" => entry.generic_name = Some(val),
                _ if key.starts_with("GenericName[") && key.ends_with("]") => {
                    let lang = key.trim_start_matches("GenericName[").trim_end_matches("]").to_string();
                    entry.generic_name_localized.push((lang, val));
                }
                "Comment" => entry.comment = Some(val),
                _ if key.starts_with("Comment[") && key.ends_with("]") => {
                    let lang = key.trim_start_matches("Comment[").trim_end_matches("]").to_string();
                    entry.comment_localized.push((lang, val));
                }
                "Exec" => entry.exec = val,
                "TryExec" => entry.try_exec = Some(val),
                "Icon" => entry.icon = Some(val),
                "Path" => entry.path = Some(val),
                "URL" => entry.url = Some(val),
                "Terminal" => entry.terminal = val.eq_ignore_ascii_case("true"),
                "NoDisplay" => entry.no_display = val.eq_ignore_ascii_case("true"),
                "StartupNotify" => entry.startup_notify = val.eq_ignore_ascii_case("true"),
                "Categories" => entry.categories = val.split(';').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect(),
                "MimeType" => entry.mime_type = val.split(';').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect(),
                "Keywords" => entry.keywords = val.split(';').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect(),
                "OnlyShowIn" => entry.only_show_in = val.split(';').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect(),
                "NotShowIn" => entry.not_show_in = val.split(';').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect(),
                "Actions" => entry.actions = val.split(';').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect(),
                _ => entry.extra.push((key.to_string(), val)),
            }
        }
    }
    if entry.type_field.is_empty() { entry.type_field = "Application".into(); }
    entry
}

pub fn wire_source_sync(editor: &Editor) {
    use std::rc::Rc;
    use std::cell::RefCell;

    let widgets = &editor.widgets;
    let source_view = editor.source_view.clone();

    // Guard to avoid infinite loops when programmatically updating
    let guard = Rc::new(RefCell::new(false));

    let update_from_fields = {
        let w = clone_widgets(widgets);
        let source_view = source_view.clone();
        let guard = guard.clone();
        move || {
            if *guard.borrow() { return; }
            if let Ok(de) = collect_entry(&w) {
                *guard.borrow_mut() = true;
                let buf = source_view.buffer();
                buf.set_text(&de.to_ini_string());
                *guard.borrow_mut() = false;
            }
        }
    };

    // Connect field changes
    let connect_entry = |e: &Entry| {
        let cb = update_from_fields.clone();
        e.connect_changed(move |_| cb());
    };
    let connect_check = |c: &CheckButton| {
        let cb = update_from_fields.clone();
        c.connect_toggled(move |_| cb());
    };

    widgets.type_combo.connect_changed({ let cb = update_from_fields.clone(); move |_| cb() });
    connect_entry(&widgets.name_entry);
    connect_entry(&widgets.generic_name_entry);
    connect_entry(&widgets.comment_entry);
    connect_entry(&widgets.exec_entry);
    connect_entry(&widgets.icon_entry);
    connect_check(&widgets.terminal_check);
    connect_check(&widgets.nodisplay_check);
    connect_check(&widgets.startup_check);
    connect_entry(&widgets.categories_entry);
    connect_entry(&widgets.mimetype_entry);
    connect_entry(&widgets.keywords_entry);
    connect_entry(&widgets.onlyshowin_entry);
    connect_entry(&widgets.notshowin_entry);
    connect_entry(&widgets.tryexec_entry);
    connect_entry(&widgets.path_entry);
    connect_entry(&widgets.url_entry);
    connect_entry(&widgets.actions_entry);

    let connect_textview = |tv: &TextView| {
        let cb = update_from_fields.clone();
        tv.buffer().connect_changed(move |_| cb());
    };
    connect_textview(&widgets.localized_name);
    connect_textview(&widgets.localized_gname);
    connect_textview(&widgets.localized_comment);
    connect_textview(&widgets.extra_kv);

    // Connect source buffer changes to parse back into fields
    {
        let w = clone_widgets(widgets);
        let source_buf = source_view.buffer();
        let guard = guard.clone();
        source_buf.connect_changed(move |buf| {
            if *guard.borrow() { return; }
            let text = buf.text(&buf.start_iter(), &buf.end_iter(), true).to_string();
            let de = parse_desktop_source(&text);
            *guard.borrow_mut() = true;
            set_form_from_entry(&w, &de);
            *guard.borrow_mut() = false;
        });
    }

    // Initialize source with current fields
    update_from_fields();
}

fn clone_widgets(w: &EntryWidgets) -> EntryWidgets {
    EntryWidgets {
        type_combo: w.type_combo.clone(),
        name_entry: w.name_entry.clone(),
        generic_name_entry: w.generic_name_entry.clone(),
        comment_entry: w.comment_entry.clone(),
        exec_entry: w.exec_entry.clone(),
        icon_entry: w.icon_entry.clone(),
        terminal_check: w.terminal_check.clone(),
        nodisplay_check: w.nodisplay_check.clone(),
        startup_check: w.startup_check.clone(),
        categories_entry: w.categories_entry.clone(),
        mimetype_entry: w.mimetype_entry.clone(),
        keywords_entry: w.keywords_entry.clone(),
        onlyshowin_entry: w.onlyshowin_entry.clone(),
        notshowin_entry: w.notshowin_entry.clone(),
        tryexec_entry: w.tryexec_entry.clone(),
        path_entry: w.path_entry.clone(),
        url_entry: w.url_entry.clone(),
        actions_entry: w.actions_entry.clone(),
        localized_name: w.localized_name.clone(),
        localized_gname: w.localized_gname.clone(),
        localized_comment: w.localized_comment.clone(),
        extra_kv: w.extra_kv.clone(),
    }
}

use gtk4::{CheckButton, ComboBoxText, Entry, TextView};
use gtk4::prelude::*;
use crate::domain::desktop_entry::DesktopEntry;

pub fn collect_entry(
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

pub fn split_semicolon(e: &Entry) -> Vec<String> { e.text().split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect() }
pub fn opt_text(e: &Entry) -> Option<String> { let s = e.text().trim().to_string(); if s.is_empty() { None } else { Some(s) } }
pub fn buffer_text(tv: &TextView) -> String { let buf = tv.buffer(); buf.text(&buf.start_iter(), &buf.end_iter(), true).to_string() }
pub fn parse_lang_lines(s: &str) -> Vec<(String, String)> {
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
pub fn parse_kv_lines(s: &str) -> Vec<(String, String)> {
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

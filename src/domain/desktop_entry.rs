use std::fmt::Write as _;

#[derive(Debug, Clone, Default)]
pub struct DesktopEntry {
    pub type_field: String,           // Type
    pub name: String,                 // Name
    pub generic_name: Option<String>, // GenericName
    pub comment: Option<String>,      // Comment
    pub exec: String,                 // Exec
    pub icon: Option<String>,         // Icon
    pub terminal: bool,               // Terminal
    pub categories: Vec<String>,      // Categories (semicolon-separated)
    pub mime_type: Vec<String>,       // MimeType (semicolon-separated)
    pub keywords: Vec<String>,        // Keywords (semicolon-separated)
    pub only_show_in: Vec<String>,    // OnlyShowIn
    pub not_show_in: Vec<String>,     // NotShowIn
    pub no_display: bool,             // NoDisplay
    pub startup_notify: bool,         // StartupNotify
    pub try_exec: Option<String>,     // TryExec
    pub path: Option<String>,         // Path (WorkingDirectory)
    pub url: Option<String>,          // For Type=Link
    pub actions: Vec<String>,         // Actions (names)
    pub extra: Vec<(String, String)>, // Any extra key=value

    // Localized variants
    pub name_localized: Vec<(String, String)>,        // (lang, value) => Name[fr]=...
    pub generic_name_localized: Vec<(String, String)>,
    pub comment_localized: Vec<(String, String)>,
}

impl DesktopEntry {
    pub fn validate(&self) -> Result<(), String> {
        if self.type_field.is_empty() {
            return Err("Type is required".into());
        }
        if self.name.trim().is_empty() {
            return Err("Name is required".into());
        }
        match self.type_field.as_str() {
            "Application" => {
                if self.exec.trim().is_empty() {
                    return Err("Exec is required for Type=Application".into());
                }
            }
            "Link" => {
                if self.url.as_deref().unwrap_or("").trim().is_empty() {
                    return Err("URL is required for Type=Link".into());
                }
            }
            "Directory" => {}
            _ => return Err("Type must be one of Application, Link, Directory".into()),
        }
        Ok(())
    }

    pub fn to_ini_string(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(&mut s, "[Desktop Entry]");
        let _ = writeln!(&mut s, "Type={}", self.type_field);
        let _ = writeln!(&mut s, "Name={}", escape(&self.name));
        for (lang, val) in &self.name_localized {
            let _ = writeln!(&mut s, "Name[{}]={}", lang, escape(val));
        }
        if let Some(v) = &self.generic_name {
            let _ = writeln!(&mut s, "GenericName={}", escape(v));
        }
        for (lang, val) in &self.generic_name_localized {
            let _ = writeln!(&mut s, "GenericName[{}]={}", lang, escape(val));
        }
        if let Some(v) = &self.comment {
            let _ = writeln!(&mut s, "Comment={}", escape(v));
        }
        for (lang, val) in &self.comment_localized {
            let _ = writeln!(&mut s, "Comment[{}]={}", lang, escape(val));
        }
        if !self.exec.is_empty() {
            let _ = writeln!(&mut s, "Exec={}", self.exec.trim());
        }
        if let Some(v) = &self.try_exec {
            let _ = writeln!(&mut s, "TryExec={}", v.trim());
        }
        if let Some(v) = &self.icon {
            let _ = writeln!(&mut s, "Icon={}", v.trim());
        }
        if let Some(v) = &self.path {
            let _ = writeln!(&mut s, "Path={}", v.trim());
        }
        if let Some(v) = &self.url {
            let _ = writeln!(&mut s, "URL={}", v.trim());
        }
        let _ = writeln!(&mut s, "Terminal={}", if self.terminal { "true" } else { "false" });
        let _ = writeln!(&mut s, "NoDisplay={}", if self.no_display { "true" } else { "false" });
        let _ = writeln!(
            &mut s,
            "StartupNotify={}",
            if self.startup_notify { "true" } else { "false" }
        );
        if !self.categories.is_empty() {
            let _ = writeln!(&mut s, "Categories={};", self.categories.join(";"));
        }
        if !self.mime_type.is_empty() {
            let _ = writeln!(&mut s, "MimeType={};", self.mime_type.join(";"));
        }
        if !self.keywords.is_empty() {
            let _ = writeln!(&mut s, "Keywords={};", self.keywords.join(";"));
        }
        if !self.only_show_in.is_empty() {
            let _ = writeln!(&mut s, "OnlyShowIn={};", self.only_show_in.join(";"));
        }
        if !self.not_show_in.is_empty() {
            let _ = writeln!(&mut s, "NotShowIn={};", self.not_show_in.join(";"));
        }
        if !self.actions.is_empty() {
            let _ = writeln!(&mut s, "Actions={};", self.actions.join(";"));
        }
        for (k, v) in &self.extra {
            if !k.trim().is_empty() {
                let _ = writeln!(&mut s, "{}={}", k.trim(), v.trim());
            }
        }
        s
    }
}

fn escape(input: &str) -> String {
    // Per .desktop, use plain strings; escape newlines as \n
    input.replace('\n', "\\n")
}

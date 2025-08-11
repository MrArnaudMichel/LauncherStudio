use crate::domain::desktop_entry::DesktopEntry;
use anyhow::{anyhow, Context, Result};
use directories::BaseDirs;
use std::fs;
use std::path::{Path, PathBuf};

pub struct DesktopWriter;

impl DesktopWriter {
    pub fn user_applications_dir() -> Result<PathBuf> {
        if let Some(base) = BaseDirs::new() {
            let dir = base.home_dir().join(".local/share/applications");
            Ok(dir)
        } else {
            Err(anyhow!("Failed to resolve XDG base directories"))
        }
    }

    pub fn write(entry: &DesktopEntry, file_name: &str, overwrite: bool) -> Result<PathBuf> {
        entry.validate().map_err(|e| anyhow!(e))?;
        let dir = Self::user_applications_dir()?;
        fs::create_dir_all(&dir).context("Creating applications directory")?;

        let sanitized = sanitize_file_name(file_name);
        let path = dir.join(format!("{}.desktop", sanitized));
        if path.exists() && !overwrite {
            return Err(anyhow!("File already exists: {}", path.display()));
        }
        let content = entry.to_ini_string();
        fs::write(&path, content).with_context(|| format!("Writing {}", path.display()))?;

        // Try to set sane permissions (not strictly required)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&path, perms)?;
        }

        Ok(path)
    }

    pub fn write_to_path(entry: &DesktopEntry, path: &Path) -> Result<PathBuf> {
        entry.validate().map_err(|e| anyhow!(e))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("Creating directory {}", parent.display()))?;
        }
        let content = entry.to_ini_string();
        fs::write(path, content).with_context(|| format!("Writing {}", path.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o644);
            fs::set_permissions(path, perms)?;
        }
        Ok(path.to_path_buf())
    }
}

fn sanitize_file_name(input: &str) -> String {
    let input = input.trim();
    let fallback = "desktop-entry";
    let s = if input.is_empty() { fallback } else { input };
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => c,
            _ => '-'
        })
        .collect()
}

//! Config file I/O with automatic single-file backup.
//!
//! Per the user's design: when we save `config.yaml`, we copy the existing
//! file to `config.yaml.bak` first (overwriting any previous .bak). We do
//! NOT keep a multi-version history — only the most recent pre-save state.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Read the full text of a config file.
pub fn read_config(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))
}

/// Atomic-ish write: backup current file (overwriting .bak) then write new.
/// We write to a temp file first, then rename — so a crash mid-write
/// doesn't leave a half-written config.
pub fn write_config_with_backup(path: &Path, new_content: &str) -> Result<()> {
    if path.exists() {
        let bak = backup_path_for(path);
        // Copy current → .bak (overwrite any previous backup)
        std::fs::copy(path, &bak)
            .with_context(|| format!("failed to backup {} → {}", path.display(), bak.display()))?;
    }

    // Write to temp file in same dir, then rename
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let tmp = parent.join(format!(
        ".{}.pcm-tmp",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("config")
    ));
    std::fs::write(&tmp, new_content)
        .with_context(|| format!("failed to write temp {}", tmp.display()))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("failed to rename {} → {}", tmp.display(), path.display()))?;
    Ok(())
}

/// The .bak path for a given config. e.g. `/x/config.yaml` → `/x/config.yaml.bak`.
pub fn backup_path_for(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_os_string();
    s.push(".bak");
    PathBuf::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_path_appends_bak() {
        let p = Path::new("/x/config.yaml");
        assert_eq!(backup_path_for(p), PathBuf::from("/x/config.yaml.bak"));
        let p = Path::new("C:\\y\\clash-verge.yaml");
        assert_eq!(
            backup_path_for(p),
            PathBuf::from("C:\\y\\clash-verge.yaml.bak")
        );
    }

    #[test]
    fn roundtrip_backup_and_write() {
        let tmp = std::env::temp_dir().join("pcm-test-write.yaml");
        std::fs::write(&tmp, "original\n").unwrap();

        // First write: creates .bak
        write_config_with_backup(&tmp, "new content\n").unwrap();
        assert_eq!(read_config(&tmp).unwrap(), "new content\n");
        assert!(backup_path_for(&tmp).exists());
        assert_eq!(
            read_config(&backup_path_for(&tmp)).unwrap(),
            "original\n"
        );

        // Second write: overwrites .bak with the (now first) new content
        write_config_with_backup(&tmp, "newer content\n").unwrap();
        assert_eq!(read_config(&tmp).unwrap(), "newer content\n");
        assert_eq!(
            read_config(&backup_path_for(&tmp)).unwrap(),
            "new content\n"
        );

        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(backup_path_for(&tmp));
    }

    #[test]
    fn write_without_existing_file_creates_no_backup() {
        let tmp = std::env::temp_dir().join("pcm-test-write-new.yaml");
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(backup_path_for(&tmp));

        write_config_with_backup(&tmp, "fresh\n").unwrap();
        assert_eq!(read_config(&tmp).unwrap(), "fresh\n");
        assert!(!backup_path_for(&tmp).exists());

        let _ = std::fs::remove_file(&tmp);
    }
}
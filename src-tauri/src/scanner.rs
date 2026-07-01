//! Scanner for Clash-based proxy clients installed on the local computer.
//!
//! MVP Step 1: detect known clients by looking in their default install paths
//! and finding a `config.yaml` / `mihomo.yaml` next to (or in the data dir of)
//! the executable.
//!
//! We deliberately avoid `which`/PATH scanning because Clash clients are
//! typically not on PATH on Windows — they live in `D:\Program Files\FlClash\`
//! or `C:\Users\<user>\AppData\Local\` etc.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A Clash-based client we detected on the user's computer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClashClient {
    /// Display name, e.g. "FlClash", "Clash Verge".
    pub name: String,
    /// Absolute path to the client's `config.yaml` (or `mihomo.yaml`).
    pub config_path: PathBuf,
    /// Path to the install root — used to show "Open in Explorer" and to
    /// re-check detection later.
    pub install_root: PathBuf,
    /// True if the config file is a real mihomo/Clash yaml (has at least
    /// one of: `rules:`, `proxies:`, `proxy-groups:`). False means the file
    /// exists but doesn't look like a Clash config.
    pub looks_valid: bool,
    /// A short human description of where we found this client, shown in the
    /// UI's "detected clients" list.
    pub source: String,
}

/// One "known install location" hint per client. We probe each on every
/// `scan_clash_clients()` call. If the directory exists, we walk it (up to
/// 3 levels deep) and look for config yaml.
struct ClientHint {
    name: &'static str,
    install_root: PathBuf,
    /// Glob-like candidate filenames relative to install_root (we accept the
    /// first one that exists).
    config_filenames: &'static [&'static str],
}

fn user_home() -> PathBuf {
    // USERPROFILE on Windows; fallback to HOME (Git-Bash sets both).
    if let Ok(p) = std::env::var("USERPROFILE") {
        return PathBuf::from(p);
    }
    if let Ok(p) = std::env::var("HOME") {
        return PathBuf::from(p);
    }
    PathBuf::from("C:\\Users\\Public")
}

fn client_hints() -> Vec<ClientHint> {
    let home = user_home();
    let appdata_local = home.join("AppData").join("Local");
    let appdata_roaming = home.join("AppData").join("Roaming");

    // Common Clash client install locations. Add more as users report them.
    //
    // Key lesson from a real scan (2026-07-01): the "install_root" for many
    // Clash clients is NOT where the .exe lives — that's just the binaries.
    // The config dir is a sibling under %APPDATA% or %LOCALAPPDATA%. For
    // example FlClash's binaries live at D:\FlClash but its config.yaml is
    // at %APPDATA%\com.follow\clash\config.yaml. Make sure to probe the
    // data dir, not the install dir, when looking for the config.
    let mut hints = vec![
        // FlClash — config lives at %APPDATA%\com.follow\clash\
        // (the D:\FlClash install root only has the .exe + plugins).
        ClientHint {
            name: "FlClash",
            install_root: appdata_roaming.join("com.follow").join("clash"),
            config_filenames: &["config.yaml", "mihomo.yaml"],
        },
        // FlClash — some installs put data in Local instead of Roaming
        ClientHint {
            name: "FlClash",
            install_root: appdata_local.join("com.follow").join("clash"),
            config_filenames: &["config.yaml", "mihomo.yaml"],
        },
        // FlClash — also probe the binary install root as a fallback in case
        // a portable install drops config.yaml next to FlClash.exe.
        ClientHint {
            name: "FlClash",
            install_root: PathBuf::from("D:\\FlClash"),
            config_filenames: &["config.yaml", "mihomo.yaml"],
        },
        // Clash Verge (rev) — installs under %APPDATA%\io.github.clash-verge-rev.clash-verge-rev\
        // and also the older name clash-verge-rev\
        ClientHint {
            name: "Clash Verge",
            install_root: appdata_roaming
                .join("io.github.clash-verge-rev.clash-verge-rev"),
            config_filenames: &["clash-verge.yaml", "config.yaml"],
        },
        // Clash Verge (rev) — older bundle ID
        ClientHint {
            name: "Clash Verge",
            install_root: appdata_roaming.join("clash-verge-rev"),
            config_filenames: &["clash-verge.yaml", "config.yaml"],
        },
        // Clash Verge (old) — under %LOCALAPPDATA%\clash-verge\
        ClientHint {
            name: "Clash Verge",
            install_root: appdata_local.join("clash-verge"),
            config_filenames: &["clash-verge.yaml", "config.yaml"],
        },
        // Clash for Windows — under %APPDATA%\
        ClientHint {
            name: "Clash for Windows",
            install_root: appdata_roaming.join("Clash for Windows"),
            config_filenames: &["config.yaml", "config.yml"],
        },
        // mihomo standalone — sometimes installed at D:\mihomo, C:\mihomo
        ClientHint {
            name: "mihomo",
            install_root: PathBuf::from("D:\\mihomo"),
            config_filenames: &["config.yaml"],
        },
        ClientHint {
            name: "mihomo",
            install_root: PathBuf::from("C:\\mihomo"),
            config_filenames: &["config.yaml"],
        },
    ];

    // Also probe any "D:\\*" and "C:\\Program Files\\*" that look like
    // Clash clients. This is a best-effort, cheap scan.
    for drive in &[PathBuf::from("D:\\"), PathBuf::from("C:\\")] {
        if let Ok(read) = std::fs::read_dir(drive) {
            for entry in read.flatten().take(50) {
                let name = entry.file_name().to_string_lossy().to_string();
                let lower = name.to_lowercase();
                if lower.contains("clash")
                    || lower.contains("verge")
                    || lower.contains("mihomo")
                    || lower.contains("clash.meta")
                {
                    let root = entry.path();
                    if !hints.iter().any(|h| h.install_root == root) {
                        hints.push(ClientHint {
                            name: "Clash (auto-detected)",
                            install_root: root,
                            config_filenames: &["config.yaml", "mihomo.yaml", "clash-verge.yaml"],
                        });
                    }
                }
            }
        }
    }

    hints
}

/// Walk up to 3 levels under `install_root` looking for the candidate
/// `config_filenames`. Returns the first match.
fn find_config_yaml(root: &Path, candidates: &[&str]) -> Option<PathBuf> {
    if !root.exists() {
        return None;
    }
    // Fast path: any of the candidate filenames live directly in `root`.
    for cand in candidates {
        let direct = root.join(cand);
        if direct.is_file() {
            return Some(direct);
        }
    }
    // Fallback: shallow walk (max depth 3) for the well-known file.
    for entry in WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if candidates.iter().any(|c| *c == name) {
            return Some(entry.into_path());
        }
    }
    None
}

/// Cheap heuristic: does the file at `path` look like a mihomo/Clash config?
/// We just look for any of the canonical top-level keys. Reading 1 KB is
/// enough to find them.
fn looks_like_clash_config(path: &Path) -> bool {
    let Ok(s) = std::fs::read_to_string(path) else {
        return false;
    };
    let head: String = s.chars().take(2048).collect();
    head.contains("rules:")
        || head.contains("proxies:")
        || head.contains("proxy-groups:")
        || head.contains("mixed-port:")
}

pub fn scan_clash_clients() -> Vec<ClashClient> {
    let mut out: Vec<ClashClient> = Vec::new();
    for hint in client_hints() {
        let Some(config_path) = find_config_yaml(&hint.install_root, hint.config_filenames) else {
            continue;
        };
        let looks_valid = looks_like_clash_config(&config_path);
        let source = format!("{}\\…", hint.install_root.display());
        // De-dup: if we already have a client with the same config_path, skip.
        if out.iter().any(|c| c.config_path == config_path) {
            continue;
        }
        out.push(ClashClient {
            name: hint.name.to_string(),
            config_path,
            install_root: hint.install_root,
            looks_valid,
            source,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_config_in_root_dir() {
        let tmp = std::env::temp_dir().join("pcm-test-root");
        let _ = std::fs::create_dir_all(&tmp);
        let cfg = tmp.join("config.yaml");
        std::fs::write(&cfg, "rules: []\n").unwrap();
        let found = find_config_yaml(&tmp, &["config.yaml"]);
        assert_eq!(found, Some(cfg));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn returns_none_for_missing_dir() {
        let missing = std::env::temp_dir().join("pcm-test-does-not-exist-zzz");
        assert_eq!(find_config_yaml(&missing, &["config.yaml"]), None);
    }

    #[test]
    fn heuristic_detects_clash_config() {
        let tmp = std::env::temp_dir().join("pcm-test-heuristic.yaml");
        std::fs::write(&tmp, "mixed-port: 7890\nrules: []\n").unwrap();
        assert!(looks_like_clash_config(&tmp));
        std::fs::write(&tmp, "totally: unrelated\n").unwrap();
        assert!(!looks_like_clash_config(&tmp));
        let _ = std::fs::remove_file(&tmp);
    }
}

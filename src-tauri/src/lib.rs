//! PlaneClash Manage — Rust backend
//!
//! Architecture: Tauri 2 + React frontend, all file I/O and Clash rule
//! parsing/writing happens here. The frontend never touches the filesystem
//! directly except via the dialog plugin (user picks files explicitly).
//!
//! MVP Step 1: scan the computer for Clash-based clients and report their
//! `config.yaml` locations. Subsequent steps will add parse → edit → save.

mod scanner;
mod rules;

use scanner::{scan_clash_clients, ClashClient};

/// Tauri command: scan the local filesystem for installed Clash-based clients
/// and return the ones we know how to handle.
#[tauri::command]
fn scan_clients() -> Vec<ClashClient> {
    scan_clash_clients()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![scan_clients])
        .run(tauri::generate_context!())
        .expect("error while running PlaneClash Manage");
}

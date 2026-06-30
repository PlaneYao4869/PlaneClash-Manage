//! PlaneClash Manage — Rust backend
//!
//! Tauri commands exposed to the React frontend:
//!   - scan_clients()        : detect installed Clash clients on this PC
//!   - load_config(path)     : read a config.yaml and return its rules
//!   - save_rules(path, rs)  : write new rules back, with .bak backup
//!
//! MVP Step 2-6: full rule management (domain + process + IP-CIDR + RULE-SET
//! + MATCH), with single-file backup on save and preserved layout of the rest
//! of the YAML file (proxies / proxy-groups / dns / etc. untouched).

mod config_io;
mod rules;
mod scanner;

use config_io::{read_config, write_config_with_backup};
use rules::{parse_rules, replace_rules_block, Rule};
use scanner::{scan_clash_clients, ClashClient};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

/// One loaded config + its parsed rules. Returned by `load_config`.
#[derive(Debug, Serialize, Deserialize)]
pub struct LoadedConfig {
    pub config_path: PathBuf,
    pub raw_yaml: String,
    pub rules: Vec<Rule>,
}

#[tauri::command]
fn scan_clients() -> Vec<ClashClient> {
    scan_clash_clients()
}

#[tauri::command]
fn load_config(path: PathBuf) -> Result<LoadedConfig, String> {
    let raw_yaml = read_config(&path).map_err(|e| e.to_string())?;
    let rules = parse_rules(&raw_yaml);
    Ok(LoadedConfig {
        config_path: path,
        raw_yaml,
        rules,
    })
}

#[tauri::command]
fn save_rules(path: PathBuf, rules: Vec<Rule>) -> Result<usize, String> {
    // Re-read the latest disk content so concurrent external edits don't get
    // clobbered by our stale in-memory copy.
    let current_yaml = read_config(&path).map_err(|e| e.to_string())?;
    let new_yaml = replace_rules_block(&current_yaml, &rules);
    write_config_with_backup(&path, &new_yaml).map_err(|e| e.to_string())?;
    Ok(rules.len())
}

/// Hold the most recently loaded config so the frontend can ask for it
/// without passing paths through every call. (Optional state.)
#[derive(Default)]
pub struct AppState {
    pub last_config: std::sync::Mutex<Option<LoadedConfig>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![scan_clients, load_config, save_rules])
        .run(tauri::generate_context!())
        .expect("error while running PlaneClash Manage");
}

// Suppress unused warning for State import — we'll use this in Step 7+ if we
// add in-memory undo/redo or session tracking.
#[allow(dead_code)]
fn _state_marker(_s: State<'_, AppState>) {}
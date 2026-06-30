//! Clash rule parsing & serialization.
//!
//! MVP Step 1 ships just the type stubs; Step 2 will fill these in.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClashRule {
    pub raw: String,
}

impl ClashRule {
    pub fn parse(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }
        Some(Self {
            raw: trimmed.to_string(),
        })
    }
}

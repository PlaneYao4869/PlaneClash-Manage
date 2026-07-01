//! Clash rule model + parser/serializer for the YAML `rules:` section.
//!
//! A Clash "rule line" is a comma-separated tuple with 2-4 fields:
//!   TYPE,PAYLOAD,TARGET[,PARAMS]
//!
//! Examples:
//!   - DOMAIN-SUFFIX,baidu.com,DIRECT
//!   - PROCESS-NAME,WeChatApp.exe,PROXY
//!   - IP-CIDR,192.168.0.0/16,DIRECT,no-resolve
//!   - MATCH,DIRECT
//!
//! On save we serialize back to YAML, preserving the original line layout
//! as much as possible so diffs stay minimal.

use serde::{Deserialize, Serialize};
use std::fmt;

/// All rule types we recognize. Order is "user importance" — DOMAIN rules
/// come first in the UI since they're the most common.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum RuleType {
    Domain,
    DomainSuffix,
    DomainKeyword,
    DomainRegex,
    GeoIp,
    /// "Core" types — the MVP focus. Everything else below is "secondary"
    /// but we still parse/serialize them faithfully.
    ProcessName,
    ProcessPath,
    SrcProcessName,
    SrcProcessPath,
    IpCidr,
    IpCidr6,
    SrcIpCidr,
    SrcIpCidr6,
    /// RULE-SET,<name>,<target> — references an external `rule-providers:` entry.
    RuleSet,
    /// Logical operators (less common but valid)
    And,
    Or,
    Not,
    /// Fallback / catch-all
    Match,
}

impl RuleType {
    /// All types the UI can *create*. Read-only types (And/Or/Not) are still
    /// parsed and shown but the Add dialog only offers these.
    pub fn creatable() -> &'static [RuleType] {
        &[
            RuleType::Domain,
            RuleType::DomainSuffix,
            RuleType::DomainKeyword,
            RuleType::DomainRegex,
            RuleType::GeoIp,
            RuleType::ProcessName,
            RuleType::ProcessPath,
            RuleType::IpCidr,
            RuleType::IpCidr6,
            RuleType::SrcIpCidr,
            RuleType::RuleSet,
            RuleType::Match,
        ]
    }

    /// Group label for UI sidebar.
    pub fn group(&self) -> RuleGroup {
        match self {
            RuleType::Domain
            | RuleType::DomainSuffix
            | RuleType::DomainKeyword
            | RuleType::DomainRegex
            | RuleType::GeoIp => RuleGroup::Domain,
            RuleType::ProcessName
            | RuleType::ProcessPath
            | RuleType::SrcProcessName
            | RuleType::SrcProcessPath => RuleGroup::Process,
            RuleType::IpCidr
            | RuleType::IpCidr6
            | RuleType::SrcIpCidr
            | RuleType::SrcIpCidr6 => RuleGroup::IpCidr,
            RuleType::RuleSet => RuleGroup::RuleSet,
            RuleType::Match => RuleGroup::Match,
            RuleType::And | RuleType::Or | RuleType::Not => RuleGroup::Logical,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RuleType::Domain => "DOMAIN",
            RuleType::DomainSuffix => "DOMAIN-SUFFIX",
            RuleType::DomainKeyword => "DOMAIN-KEYWORD",
            RuleType::DomainRegex => "DOMAIN-REGEX",
            RuleType::GeoIp => "GEOIP",
            RuleType::ProcessName => "PROCESS-NAME",
            RuleType::ProcessPath => "PROCESS-PATH",
            RuleType::SrcProcessName => "SRC-PROCESS-NAME",
            RuleType::SrcProcessPath => "SRC-PROCESS-PATH",
            RuleType::IpCidr => "IP-CIDR",
            RuleType::IpCidr6 => "IP-CIDR6",
            RuleType::SrcIpCidr => "SRC-IP-CIDR",
            RuleType::SrcIpCidr6 => "SRC-IP-CIDR6",
            RuleType::RuleSet => "RULE-SET",
            RuleType::And => "AND",
            RuleType::Or => "OR",
            RuleType::Not => "NOT",
            RuleType::Match => "MATCH",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "DOMAIN" => RuleType::Domain,
            "DOMAIN-SUFFIX" => RuleType::DomainSuffix,
            "DOMAIN-KEYWORD" => RuleType::DomainKeyword,
            "DOMAIN-REGEX" => RuleType::DomainRegex,
            "GEOIP" => RuleType::GeoIp,
            "PROCESS-NAME" => RuleType::ProcessName,
            "PROCESS-PATH" => RuleType::ProcessPath,
            "SRC-PROCESS-NAME" => RuleType::SrcProcessName,
            "SRC-PROCESS-PATH" => RuleType::SrcProcessPath,
            "IP-CIDR" => RuleType::IpCidr,
            "IP-CIDR6" => RuleType::IpCidr6,
            "SRC-IP-CIDR" => RuleType::SrcIpCidr,
            "SRC-IP-CIDR6" => RuleType::SrcIpCidr6,
            "RULE-SET" => RuleType::RuleSet,
            "AND" => RuleType::And,
            "OR" => RuleType::Or,
            "NOT" => RuleType::Not,
            "MATCH" => RuleType::Match,
            _ => return None,
        })
    }
}

impl fmt::Display for RuleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RuleGroup {
    Domain,
    Process,
    IpCidr,
    RuleSet,
    Match,
    Logical,
}

impl RuleGroup {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleGroup::Domain => "domain",
            RuleGroup::Process => "process",
            RuleGroup::IpCidr => "ip_cidr",
            RuleGroup::RuleSet => "rule_set",
            RuleGroup::Match => "match",
            RuleGroup::Logical => "logical",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "domain" => RuleGroup::Domain,
            "process" => RuleGroup::Process,
            "ip_cidr" => RuleGroup::IpCidr,
            "rule_set" => RuleGroup::RuleSet,
            "match" => RuleGroup::Match,
            "logical" => RuleGroup::Logical,
            _ => return None,
        })
    }
}

/// One Clash rule. `id` is a stable synthetic id we assign during parsing,
/// used by the UI for React-style keys. Not written back to YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: u32,
    pub rule_type: RuleType,
    pub payload: String,
    pub target: String,
    /// Comma-separated, no surrounding commas. Common values: "no-resolve".
    #[serde(default)]
    pub params: Vec<String>,
    /// True if this line was preceded by a `#` comment in the source YAML.
    /// On save we preserve that prefix so user comments survive round-trips.
    #[serde(default)]
    pub disabled_in_source: bool,
}

impl Rule {
    /// Parse a single rule line. Returns None for blank/comment-only lines.
    /// Skips lines that start with `-` (it's just YAML list syntax, we don't
    /// want it in the payload).
    pub fn parse(line: &str) -> Option<Self> {
        let trimmed = line.trim_start();

        // Detect "commented-out" lines: they start with `#`. The rule body
        // we still want to parse lives after `# - ` (YAML "commented-out list
        // item") or sometimes `# TYPE,...` (commented without list marker).
        let (body_after_marker, disabled_in_source) = if let Some(rest) = trimmed.strip_prefix("# - ") {
            (rest, true)
        } else if let Some(rest) = trimmed.strip_prefix("# ") {
            (rest, true)
        } else {
            (trimmed, false)
        };

        // Strip a YAML list marker (`- `) if present.
        let body = body_after_marker
            .strip_prefix("- ")
            .or_else(|| body_after_marker.strip_prefix("-"))
            .unwrap_or(body_after_marker)
            .trim();

        if body.is_empty() {
            return None;
        }

        let parts: Vec<&str> = body.split(',').map(|s| s.trim()).collect();
        if parts.len() < 2 {
            return None;
        }
        let rule_type = RuleType::from_str(parts[0])?;
        let payload = parts.get(1).copied().unwrap_or("").to_string();
        let target = parts.get(2).copied().unwrap_or("").to_string();
        let params: Vec<String> = if parts.len() >= 4 {
            parts[3..]
                .iter()
                .flat_map(|p| p.split(','))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            Vec::new()
        };
        Some(Rule {
            id: 0, // assigned later by parse_rules()
            rule_type,
            payload,
            target,
            params,
            disabled_in_source,
        })
    }

    /// Serialize back to a YAML list item line (without the `- ` prefix).
    pub fn to_yaml_line(&self) -> String {
        let mut s = format!("{},{},{}", self.rule_type, self.payload, self.target);
        if !self.params.is_empty() {
            s.push(',');
            s.push_str(&self.params.join(","));
        }
        if self.disabled_in_source {
            format!("# - {}", s)
        } else {
            s
        }
    }
}

/// Parse the `rules:` block out of a YAML config text.
/// Returns the rules (with stable `id`) plus a count of comments/blank lines
/// that were skipped (useful for diagnostics).
pub fn parse_rules(yaml_text: &str) -> Vec<Rule> {
    let mut out = Vec::new();
    let mut next_id: u32 = 1;
    let mut in_rules = false;
    let mut rules_indent: Option<usize> = None;

    for line in yaml_text.lines() {
        // Detect `rules:` start
        if !in_rules {
            let trimmed = line.trim_start();
            if trimmed == "rules:" || trimmed.starts_with("rules: ") {
                in_rules = true;
                rules_indent = Some(line.len() - trimmed.len());
                continue;
            }
            continue;
        }

        // We're inside rules:. Stop if we hit a line that's not indented
        // enough (top-level key like `proxies:` again).
        if line.is_empty() {
            continue;
        }
        let leading = line.len() - line.trim_start_matches(' ').len();
        let indent = rules_indent.unwrap_or(0) + 2; // rule lines are 2 spaces deeper
        if leading < indent && line.trim_start().ends_with(':') {
            // Hit another top-level key like `proxies:`, `proxy-groups:`, etc.
            break;
        }

        if let Some(mut rule) = Rule::parse(line) {
            rule.id = next_id;
            next_id += 1;
            out.push(rule);
        }
    }
    out
}

/// Serialize rules back to YAML list-item lines. Caller is responsible
/// for prefixing `rules:\n` and the proper indentation.
#[allow(dead_code)]
pub fn rules_to_yaml_lines(rules: &[Rule]) -> Vec<String> {
    rules.iter().map(|r| r.to_yaml_line()).collect()
}

/// Locate the byte range of the `rules:` block (including the `rules:` line
/// itself, up to but not including the next top-level key) so the caller
/// can replace just that slice without touching proxies/proxy-groups/etc.
///
/// Returns (start, end) byte offsets in `yaml_text`. If `rules:` is not
/// found, returns None.
pub fn locate_rules_block(yaml_text: &str) -> Option<(usize, usize)> {
    let mut in_rules = false;
    let mut start: Option<usize> = None;
    let mut rules_indent: Option<usize> = None;

    let mut pos = 0usize;
    for line in yaml_text.split_inclusive('\n') {
        // split_inclusive keeps the trailing '\n', which means raw equality
        // checks like `== "rules:"` would fail because of the trailing \n.
        // Use trim_end() to strip both leading whitespace and the newline
        // before comparing.
        let trimmed_start = line.trim_end().trim_start();
        let leading = line.len() - line.trim_start().len();

        if !in_rules {
            if trimmed_start == "rules:" || trimmed_start.starts_with("rules: ") {
                start = Some(pos);
                in_rules = true;
                rules_indent = Some(leading);
            }
        } else {
            // Empty line — keep going, may be inside rules block
            if trimmed_start.is_empty() {
                // continue
            } else {
                let indent = rules_indent.unwrap_or(0) + 2;
                // Detect a "next top-level key" line: it has less indent than
                // the rule lines, AND it looks like a key (contains exactly
                // one ':' that isn't followed by YAML collection braces/brackets
                // and isn't at the very start of a list item).
                //
                // Examples we treat as the next key:
                //   "proxies:"
                //   "proxy-groups:"
                //   "rule-providers:"
                //   "dns:"            (also followed by anything)
                //
                // Examples we DON'T treat as a new key (still inside rules):
                //   "- DOMAIN,foo,DIRECT"            (list item)
                //   "# comment"                       (commented out)
                //   "  - foo: bar"                    (nested mapping)
                if leading < indent && is_top_level_key(trimmed_start) {
                    // End of rules block — return up to this line's start
                    return Some((start.unwrap(), pos));
                }
            }
        }

        pos += line.len();
    }

    // Reached EOF while still inside rules block
    start.map(|s| (s, yaml_text.len()))
}

/// Heuristic: is this line a top-level YAML key (e.g. `proxies:` or
/// `proxy-groups: []`)? Returns false for list items (`- foo`) and
/// for indented mappings (`foo: bar` with leading spaces).
fn is_top_level_key(trimmed: &str) -> bool {
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') {
        return false;
    }
    // Find the FIRST ':' that isn't at the start — that's the key/value split.
    // YAML allows `:` inside unquoted values, so we look for one that has
    // either end-of-line, a space, or `[`/`{` after it.
    if let Some(idx) = trimmed.find(':') {
        if idx == 0 {
            return false; // ":foo" — not a valid key
        }
        let after = &trimmed[idx + 1..];
        // After the colon we want either end-of-line or a separator char.
        // Inline collections like `proxies: []` have a space then `[`.
        return after.is_empty()
            || after.starts_with(' ')
            || after.starts_with('[')
            || after.starts_with('{')
            || after.starts_with('"')
            || after.starts_with('\'');
    }
    false
}

/// Replace the `rules:` block in `yaml_text` with the new rules.
/// Preserves every other key (proxies, proxy-groups, dns, rule-providers, etc.).
///
/// Strategy: find the block, splice in our replacement which keeps `rules:`
/// as the header. If the new rules list is empty, leave just `rules: []`.
pub fn replace_rules_block(yaml_text: &str, new_rules: &[Rule]) -> String {
    let Some((start, end)) = locate_rules_block(yaml_text) else {
        // No existing rules: block — append one at the end
        let mut out = yaml_text.trim_end().to_string();
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
        out.push_str("rules:\n");
        for r in new_rules {
            out.push_str("  - ");
            out.push_str(&r.to_yaml_line());
            out.push('\n');
        }
        return out;
    };

    let mut out = String::with_capacity(yaml_text.len());
    out.push_str(&yaml_text[..start]);

    if new_rules.is_empty() {
        out.push_str("rules: []\n");
    } else {
        out.push_str("rules:\n");
        for r in new_rules {
            out.push_str("  - ");
            out.push_str(&r.to_yaml_line());
            out.push('\n');
        }
    }

    // Append everything after the rules block
    out.push_str(&yaml_text[end..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_type_roundtrip() {
        for t in [
            RuleType::Domain,
            RuleType::DomainSuffix,
            RuleType::ProcessName,
            RuleType::IpCidr,
            RuleType::Match,
            RuleType::RuleSet,
        ] {
            assert_eq!(RuleType::from_str(t.as_str()), Some(t));
        }
    }

    #[test]
    fn parse_simple_rule() {
        let r = Rule::parse("  - DOMAIN-SUFFIX,baidu.com,DIRECT").unwrap();
        assert_eq!(r.rule_type, RuleType::DomainSuffix);
        assert_eq!(r.payload, "baidu.com");
        assert_eq!(r.target, "DIRECT");
        assert!(r.params.is_empty());
        assert!(!r.disabled_in_source);
    }

    #[test]
    fn parse_rule_with_params() {
        let r = Rule::parse("- IP-CIDR,192.168.0.0/16,DIRECT,no-resolve").unwrap();
        assert_eq!(r.rule_type, RuleType::IpCidr);
        assert_eq!(r.payload, "192.168.0.0/16");
        assert_eq!(r.target, "DIRECT");
        assert_eq!(r.params, vec!["no-resolve"]);
    }

    #[test]
    fn parse_commented_rule() {
        let r = Rule::parse("  # - DOMAIN,a.com,DIRECT").unwrap();
        assert!(r.disabled_in_source);
        assert_eq!(r.rule_type, RuleType::Domain);
    }

    #[test]
    fn parse_blank_returns_none() {
        assert!(Rule::parse("").is_none());
        assert!(Rule::parse("   ").is_none());
        assert!(Rule::parse("# full line comment").is_none());
    }

    #[test]
    fn to_yaml_line_roundtrip() {
        let r = Rule {
            id: 1,
            rule_type: RuleType::ProcessName,
            payload: "WeChatApp.exe".into(),
            target: "DIRECT".into(),
            params: vec![],
            disabled_in_source: false,
        };
        assert_eq!(r.to_yaml_line(), "PROCESS-NAME,WeChatApp.exe,DIRECT");
    }

    #[test]
    fn parse_rules_full_block() {
        let yaml = r#"
mixed-port: 7890
rules:
  - DOMAIN-SUFFIX,baidu.com,DIRECT
  - PROCESS-NAME,WeChatApp.exe,DIRECT
  - IP-CIDR,192.168.0.0/16,DIRECT,no-resolve
proxies:
  - name: test
    type: ss
"#;
        let rules = parse_rules(yaml);
        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].rule_type, RuleType::DomainSuffix);
        assert_eq!(rules[1].rule_type, RuleType::ProcessName);
        assert_eq!(rules[2].rule_type, RuleType::IpCidr);
        assert_eq!(rules[2].params, vec!["no-resolve"]);
        // IDs are unique
        let mut ids: Vec<u32> = rules.iter().map(|r| r.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn locate_rules_block_basic() {
        let yaml = "mixed-port: 7890\nrules:\n  - A,B,C\nproxies:\n  - x\n";
        let (s, e) = locate_rules_block(yaml).unwrap();
        let block = &yaml[s..e];
        // The block should be just the `rules:` section: header + list
        // items only, NOT including the next top-level key (proxies:).
        assert!(block.starts_with("rules:"));
        assert!(block.contains("A,B,C"));
        assert!(!block.contains("proxies:"));
        // After the block, the original "proxies:" must be preserved
        // (replace_rules_block appends yaml[end..] verbatim).
        assert!(yaml[e..].starts_with("proxies:"));
    }

    #[test]
    fn replace_rules_block_preserves_other_keys() {
        let yaml = "mixed-port: 7890\nrules:\n  - A,B,C\nproxies:\n  - name: test\n";
        let new_rules = vec![Rule {
            id: 1,
            rule_type: RuleType::Domain,
            payload: "x.com".into(),
            target: "REJECT".into(),
            params: vec![],
            disabled_in_source: false,
        }];
        let out = replace_rules_block(yaml, &new_rules);
        assert!(out.contains("mixed-port: 7890"));
        assert!(out.contains("DOMAIN,x.com,REJECT"));
        assert!(!out.contains("A,B,C"));
        assert!(out.contains("name: test"));
    }

    #[test]
    fn replace_rules_block_empty_list() {
        let yaml = "rules:\n  - A,B,C\nproxies: []\n";
        let out = replace_rules_block(yaml, &[]);
        assert!(out.contains("rules: []"));
        assert!(!out.contains("A,B,C"));
        assert!(out.contains("proxies: []"));
    }

    #[test]
    fn replace_rules_block_no_existing_block() {
        let yaml = "mixed-port: 7890\n";
        let new_rules = vec![Rule {
            id: 1,
            rule_type: RuleType::DomainSuffix,
            payload: "a.com".into(),
            target: "DIRECT".into(),
            params: vec![],
            disabled_in_source: false,
        }];
        let out = replace_rules_block(yaml, &new_rules);
        assert!(out.contains("mixed-port: 7890"));
        assert!(out.contains("rules:"));
        assert!(out.contains("DOMAIN-SUFFIX,a.com,DIRECT"));
    }

    #[test]
    fn creatable_includes_core_types() {
        let types: Vec<RuleType> = RuleType::creatable().to_vec();
        assert!(types.contains(&RuleType::Domain));
        assert!(types.contains(&RuleType::DomainSuffix));
        assert!(types.contains(&RuleType::ProcessName));
        assert!(types.contains(&RuleType::IpCidr));
        assert!(types.contains(&RuleType::Match));
        // Logical operators are NOT creatable (read-only)
        assert!(!types.contains(&RuleType::And));
    }
}
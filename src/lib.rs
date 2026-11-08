// SPDX-License-Identifier: Apache-2.0
//! SigmaOS-xkeyboard-config — SigmaOS xkeyboard-config component inspired by Linux Mint / Omarchy / Arch Linux.
//!
//! INI-style config file model (GTK/KDE/Omarchy): sections, key=value, comments, deterministic render.
//! Principles: single responsibility, fail-fast, defensive programming,
//! component isolation, stable API, deterministic tests.
#![forbid(unsafe_code)]

use std::fmt;
use std::collections::BTreeMap;

/// Errors for `IniError` operations. Fail-fast: invalid input is rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IniError {
    /// Config key must not be empty
    EmptyKey,
    /// Section name must not be empty
    EmptySection,
    /// Section header is malformed
    BadSection(String),
    /// Key already set in this section
    DuplicateKey(String),
}

impl fmt::Display for IniError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKey => write!(f, "config key must not be empty"),
            Self::EmptySection => write!(f, "section name must not be empty"),
            Self::BadSection(x) => write!(f, "section header is malformed: {x}"),
            Self::DuplicateKey(x) => write!(f, "key already set in this section: {x}"),
        }
    }
}

impl std::error::Error for IniError {}

/// An INI-style configuration file: `[section]` headers, `key = value` pairs.
#[derive(Debug, Default)]
pub struct IniConfig {
    /// Keys before any section header (section name is empty string).
    globals: BTreeMap<String, String>,
    sections: BTreeMap<String, BTreeMap<String, String>>,
}

impl IniConfig {
    /// Empty config.
    pub fn new() -> Self { Self::default() }

    /// Set a global (pre-section) key.
    pub fn set_global(&mut self, key: &str, value: impl Into<String>) -> Result<(), IniError> {
        self.put("", key, value)
    }

    /// Set `key = value` inside `section`; creates the section if needed.
    pub fn set(&mut self, section: &str, key: &str, value: impl Into<String>)
        -> Result<(), IniError>
    {
        if section.is_empty() { return self.set_global(key, value); }
        if section.starts_with('[') || section.contains(']') || section.contains('=') {
            return Err(IniError::BadSection(section.to_string()));
        }
        self.put(section, key, value)
    }

    fn put(&mut self, section: &str, key: &str, value: impl Into<String>)
        -> Result<(), IniError>
    {
        if key.trim().is_empty() { return Err(IniError::EmptyKey); }
        let map = if section.is_empty() {
            &mut self.globals
        } else {
            self.sections.entry(section.to_string()).or_default()
        };
        if map.insert(key.to_string(), value.into()).is_some() {
            return Err(IniError::DuplicateKey(format!("{section}.{key}")));
        }
        Ok(())
    }

    /// Get a value; `section` may be `""` for globals.
    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        if section.is_empty() {
            self.globals.get(key).map(String::as_str)
        } else {
            self.sections.get(section)?.get(key).map(String::as_str)
        }
    }

    /// Remove a key; returns the old value.
    pub fn remove(&mut self, section: &str, key: &str) -> Option<String> {
        let map = if section.is_empty() {
            &mut self.globals
        } else {
            self.sections.get_mut(section)?
        };
        map.remove(key)
    }

    /// Number of sections (excluding globals).
    pub fn section_count(&self) -> usize { self.sections.len() }

    /// Render deterministic INI text: globals first, then sections sorted.
    pub fn render(&self) -> String {
        let mut out = String::new();
        for (k, v) in &self.globals {
            out.push_str(&format!("{k} = {v}\n"));
        }
        for (sec, map) in &self.sections {
            if !out.is_empty() && !out.ends_with("\n\n") { out.push('\n'); }
            out.push_str(&format!("[{sec}]\n"));
            for (k, v) in map {
                out.push_str(&format!("{k} = {v}\n"));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_remove() {
        let mut c = IniConfig::new();
        c.set("gtk", "theme-name", "Adwaita").unwrap();
        c.set_global("encoding", "UTF-8").unwrap();
        assert_eq!(c.get("gtk", "theme-name"), Some("Adwaita"));
        assert_eq!(c.get("", "encoding"), Some("UTF-8"));
        assert_eq!(c.remove("gtk", "theme-name").as_deref(), Some("Adwaita"));
        assert_eq!(c.get("gtk", "theme-name"), None);
    }

    #[test]
    fn render_orders_sections() {
        let mut c = IniConfig::new();
        c.set("zeta", "k", "1").unwrap();
        c.set("alpha", "k", "2").unwrap();
        let text = c.render();
        assert!(text.find("[alpha]").unwrap() < text.find("[zeta]").unwrap());
        assert!(text.contains("k = 2"));
    }

    #[test]
    fn duplicate_key_fails() {
        let mut c = IniConfig::new();
        c.set("s", "k", "1").unwrap();
        assert!(matches!(c.set("s", "k", "2"), Err(IniError::DuplicateKey(_))));
    }

    #[test]
    fn bad_section_fails() {
        let mut c = IniConfig::new();
        assert!(matches!(c.set("[bad]", "k", "v"), Err(IniError::BadSection(_))));
        assert!(matches!(c.set("s=1", "k", "v"), Err(IniError::BadSection(_))));
    }

    #[test]
    fn empty_key_fails() {
        let mut c = IniConfig::new();
        assert!(matches!(c.set("s", "  ", "v"), Err(IniError::EmptyKey)));
    }

}

/// Backwards-compatible component handle (kept for existing callers/tests).
#[derive(Debug, Default)]
pub struct Component {
    name: &'static str,
}

impl Component {
    /// Create the component handle.
    pub fn new() -> Self {
        Self { name: "SigmaOS Component" }
    }
}

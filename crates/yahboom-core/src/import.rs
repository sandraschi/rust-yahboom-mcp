//! Zero-config import of `../yahboom-mcp/.env`.
//! Order: process env, then the legacy dotenv file, then built-in defaults.
//! Secret values are never written to logs.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub gateway_url: String,
    pub robot_ip: String,
    pub origin: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            gateway_url: "http://127.0.0.1:10892".to_string(),
            robot_ip: "192.168.1.11".to_string(),
            origin: "default".to_string(),
        }
    }
}

pub fn parse_env_text(text: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        let value = value.trim().trim_matches('"').trim_matches('\'');
        map.insert(key.to_string(), expand_default(value, &map));
    }
    map
}

fn expand_default(value: &str, prior: &BTreeMap<String, String>) -> String {
    let Some(rest) = value.strip_prefix("${") else {
        return value.to_string();
    };
    let Some(body) = rest.strip_suffix('}') else {
        return value.to_string();
    };
    let (name, default) = body.split_once(":-").unwrap_or((body, ""));
    if let Ok(env) = std::env::var(name) {
        if !env.is_empty() {
            return env;
        }
    }
    if let Some(found) = prior.get(name) {
        if !found.is_empty() {
            return found.clone();
        }
    }
    default.to_string()
}

pub fn discover_legacy_dir() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var("YAHBOOM_LEGACY_DIR") {
        let path = PathBuf::from(explicit.trim());
        if path.join(".env").is_file() || path.join(".env.example").is_file() {
            return Some(path);
        }
    }
    let candidates = [
        PathBuf::from("../yahboom-mcp"),
        PathBuf::from("D:/Dev/repos/yahboom-mcp"),
    ];
    candidates
        .into_iter()
        .find(|path| path.join(".env").is_file() || path.join(".env.example").is_file())
}

pub fn load_env_file(dir: &Path) -> BTreeMap<String, String> {
    let path = if dir.join(".env").is_file() {
        dir.join(".env")
    } else {
        dir.join(".env.example")
    };
    let text = std::fs::read_to_string(path).unwrap_or_default();
    parse_env_text(&text)
}

pub fn fleet_secret(name: &str) -> Option<String> {
    if let Ok(value) = std::env::var(name) {
        let trimmed = value.trim().to_string();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }
    let dir = discover_legacy_dir()?;
    let value = load_env_file(&dir).get(name)?.trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

pub fn load_settings() -> Settings {
    let mut settings = Settings::default();
    if let Some(dir) = discover_legacy_dir() {
        let file = load_env_file(&dir);
        if let Some(ip) = file.get("YAHBOOM_IP") {
            if !ip.trim().is_empty() {
                settings.robot_ip = ip.trim().to_string();
                settings.origin = "legacy-env".to_string();
            }
        }
        if let Some(url) = file.get("YAHBOOM_GATEWAY_URL") {
            if !url.trim().is_empty() {
                settings.gateway_url = url.trim().trim_end_matches('/').to_string();
                settings.origin = "legacy-env".to_string();
            }
        }
    }
    if let Ok(ip) = std::env::var("YAHBOOM_IP") {
        if !ip.trim().is_empty() {
            settings.robot_ip = ip.trim().to_string();
            settings.origin = "process-env".to_string();
        }
    }
    if let Ok(url) = std::env::var("YAHBOOM_GATEWAY_URL") {
        if !url.trim().is_empty() {
            settings.gateway_url = url.trim().trim_end_matches('/').to_string();
            settings.origin = "process-env".to_string();
        }
    }
    settings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_comments_and_default_expansion() {
        let map = parse_env_text("# note\nYAHBOOM_IP=10.0.0.5\nOTHER=${MISSING:-fallback}\n");
        assert_eq!(map.get("YAHBOOM_IP").map(String::as_str), Some("10.0.0.5"));
        assert_eq!(map.get("OTHER").map(String::as_str), Some("fallback"));
    }

    #[test]
    fn empty_file_yields_empty_map() {
        assert!(parse_env_text("").is_empty());
    }
}

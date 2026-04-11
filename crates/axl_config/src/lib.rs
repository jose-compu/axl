use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AxlConfig {
    #[serde(default)]
    pub extends: Vec<String>,
    #[serde(default)]
    pub components: HashMap<String, String>,
    #[serde(default)]
    pub rules: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub ignore: Vec<String>,
}

impl AxlConfig {
    pub fn load(path: Option<&str>) -> Result<Self> {
        let selected = path.unwrap_or("axl.config.json");
        if !Path::new(selected).exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(selected)?;
        let cfg: AxlConfig = serde_json::from_str(&raw)?;
        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::AxlConfig;
    use std::fs;

    #[test]
    fn returns_default_when_file_missing() {
        let path = std::env::temp_dir().join("axl_config_missing.json");
        if path.exists() {
            fs::remove_file(&path).expect("cleanup existing file");
        }
        let cfg = AxlConfig::load(Some(path.to_string_lossy().as_ref())).expect("load should work");
        assert!(cfg.extends.is_empty());
        assert!(cfg.ignore.is_empty());
    }

    #[test]
    fn loads_config_from_json_file() {
        let path = std::env::temp_dir().join(format!(
            "axl_config_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock should be after epoch")
                .as_nanos()
        ));
        fs::write(
            &path,
            r#"{
  "extends": ["recommended"],
  "components": { "Link": "a" },
  "rules": { "focus/tabindex-positive": "warn" },
  "ignore": ["**/*.test.tsx"]
}"#,
        )
        .expect("write config file");

        let cfg = AxlConfig::load(Some(path.to_string_lossy().as_ref())).expect("config should parse");
        assert_eq!(cfg.extends, vec!["recommended"]);
        assert_eq!(cfg.components.get("Link").map(String::as_str), Some("a"));
        assert!(cfg.rules.contains_key("focus/tabindex-positive"));
        assert_eq!(cfg.ignore, vec!["**/*.test.tsx"]);

        let _ = fs::remove_file(path);
    }
}

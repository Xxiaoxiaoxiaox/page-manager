use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub exe_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerConfig {
    pub apps: Vec<AppEntry>,
}

impl ManagerConfig {
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_else(|_| Self::default())
        } else {
            let config = Self::default();
            config.save();
            config
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self).unwrap();
        let _ = fs::write(&path, content);
    }

    fn config_path() -> PathBuf {
        let mut path = dirs::home_dir().unwrap_or_default();
        path.push(".pake-manager");
        path.push("apps.json");
        path
    }

    pub fn default() -> Self {
        let default_apps = vec![
            AppEntry { name: "DeepSeek".into(), url: "https://chat.deepseek.com".into(), exe_path: "".into() },
            AppEntry { name: "Grok".into(), url: "https://grok.com".into(), exe_path: "".into() },
            AppEntry { name: "Gemini".into(), url: "https://gemini.google.com".into(), exe_path: "".into() },
            AppEntry { name: "Claude".into(), url: "https://claude.ai".into(), exe_path: "".into() },
            AppEntry { name: "ChatGPT".into(), url: "https://chatgpt.com".into(), exe_path: "".into() },
            AppEntry { name: "YouTube".into(), url: "https://youtube.com".into(), exe_path: "".into() },
        ];
        ManagerConfig { apps: default_apps }
    }
}

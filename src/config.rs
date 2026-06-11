use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub name: String,
    pub server: String,
    pub username: Option<String>,
    pub domain: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub fullscreen: Option<bool>,
    #[serde(default)]
    pub dynamic_resolution: Option<bool>,
    #[serde(default)]
    pub scale_desktop: Option<u32>,
    #[serde(default)]
    pub smart_sizing: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub connections: Vec<ConnectionConfig>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content =
                std::fs::read_to_string(&config_path).context("Failed to read config file")?;
            serde_json::from_str(&content).context("Failed to parse config file")
        } else {
            Ok(Config {
                connections: Vec::new(),
            })
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let content = serde_json::to_string_pretty(self).context("Failed to serialize config")?;
        std::fs::write(&config_path, content).context("Failed to write config file")?;

        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Could not find config directory")?;
        Ok(config_dir.join("rrdp").join("connections.json"))
    }

    pub fn save_connection(
        &mut self,
        name: &str,
        server: &str,
        username: Option<String>,
        domain: Option<String>,
        description: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
        fullscreen: Option<bool>,
        dynamic_resolution: Option<bool>,
        scale_desktop: Option<u32>,
        smart_sizing: Option<bool>,
    ) {
        // Remove existing connection with same name
        self.connections.retain(|c| c.name != name);

        self.connections.push(ConnectionConfig {
            name: name.to_string(),
            server: server.to_string(),
            username,
            domain,
            description,
            width,
            height,
            fullscreen,
            dynamic_resolution,
            scale_desktop,
            smart_sizing,
        });
    }

    pub fn list_connections(&self) -> &[ConnectionConfig] {
        &self.connections
    }

    pub fn get_connection(&self, name: &str) -> Option<&ConnectionConfig> {
        self.connections.iter().find(|c| c.name == name)
    }

    pub fn delete_connection(&mut self, name: &str) -> bool {
        let initial_len = self.connections.len();
        self.connections.retain(|c| c.name != name);
        self.connections.len() < initial_len
    }
}

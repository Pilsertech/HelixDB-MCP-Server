// Configuration module for AI Memory Layer MCP Server
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub helix: HelixConfig,
    pub embedding: EmbeddingConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HelixConfig {
    pub endpoint: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmbeddingConfig {
    pub mode: EmbeddingMode,
    #[serde(default)]
    pub provider: Option<EmbeddingProvider>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub openai_api_url: Option<String>,
    #[serde(default)]
    pub gemini_api_url: Option<String>,
    #[serde(default)]
    pub local_api_url: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default = "default_dimensions")]
    pub dimensions: usize,
    // TCP embedding server configuration
    #[serde(default)]
    pub tcp_address: Option<String>,
    #[serde(default = "default_tcp_timeout")]
    pub tcp_timeout_secs: u64,
}

fn default_tcp_timeout() -> u64 {
    30
}

fn default_dimensions() -> usize {
    1536
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingMode {
    Mcp,      // MCP server generates embeddings
    Helixdb,  // HelixDB generates embeddings via Embed()
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingProvider {
    OpenAI,
    Gemini,
    Local,
    Tcp,  // Direct TCP connection to EmbeddingServer
}

impl Config {
    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Load configuration with fallback to default path
    pub fn load() -> Result<Self> {
        // Try mcpconfig.toml in current directory
        if Path::new("mcpconfig.toml").exists() {
            return Self::from_file("mcpconfig.toml");
        }

        // Fallback to default configuration
        Ok(Self::default())
    }

    /// Get API key from config or environment
    pub fn get_api_key(&self) -> Option<String> {
        // Check config first
        if let Some(ref key) = self.embedding.api_key {
            if !key.is_empty() {
                return Some(key.clone());
            }
        }

        // Check environment based on provider
        match self.embedding.provider {
            Some(EmbeddingProvider::OpenAI) => {
                std::env::var("OPENAI_API_KEY").ok()
            }
            Some(EmbeddingProvider::Gemini) => {
                std::env::var("GEMINI_API_KEY").ok()
            }
            Some(EmbeddingProvider::Local) | Some(EmbeddingProvider::Tcp) | None => None,
        }
    }

    /// Check if MCP server should handle embedding generation
    pub fn is_mcp_embedding_enabled(&self) -> bool {
        self.embedding.mode == EmbeddingMode::Mcp
    }

    /// Check if HelixDB should handle embedding generation
    pub fn is_helixdb_embedding_enabled(&self) -> bool {
        self.embedding.mode == EmbeddingMode::Helixdb
    }

    /// Get the appropriate embedding API URL based on the provider
    pub fn get_embedding_api_url(&self) -> Option<String> {
        match self.embedding.provider {
            Some(EmbeddingProvider::OpenAI) => {
                self.embedding.openai_api_url.clone()
            }
            Some(EmbeddingProvider::Gemini) => {
                self.embedding.gemini_api_url.clone()
            }
            Some(EmbeddingProvider::Local) => {
                self.embedding.local_api_url.clone()
            }
            Some(EmbeddingProvider::Tcp) => {
                self.embedding.tcp_address.clone()
            }
            None => None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                name: "AI Memory Layer MCP Server".to_string(),
                version: "0.1.0".to_string(),
            },
            helix: HelixConfig {
                endpoint: "127.0.0.1".to_string(),
                port: 6969,
            },
            embedding: EmbeddingConfig {
                mode: EmbeddingMode::Helixdb, // Default to simpler mode
                provider: None,
                model: None,
                openai_api_url: None,
                gemini_api_url: None,
                local_api_url: None,
                api_key: None,
                dimensions: 1536,
                tcp_address: None,
                tcp_timeout_secs: 30,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.embedding.mode, EmbeddingMode::Helixdb);
        assert!(config.is_helixdb_embedding_enabled());
        assert!(!config.is_mcp_embedding_enabled());
    }

    #[test]
    fn test_api_key_from_env() {
        std::env::set_var("OPENAI_API_KEY", "test-key");
        let config = Config::default();
        assert_eq!(config.get_api_key(), Some("test-key".to_string()));
        std::env::remove_var("OPENAI_API_KEY");
    }
}

use anyhow::{Context, Result};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

/// HelixDB HTTP client for MCP endpoints
#[derive(Clone)]
pub struct HelixClient {
    base_url: String,
    http_client: HttpClient,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRequest {
    pub connection_id: Option<String>,
    pub data: Option<Value>,
}

impl HelixClient {
    /// Create a new HelixDB client
    pub fn new(endpoint: &str, port: u16) -> Self {
        let base_url = format!("http://{}:{}", endpoint, port);
        Self {
            base_url,
            http_client: HttpClient::new(),
        }
    }

    /// Execute a HelixDB MCP query
    pub async fn query(&self, endpoint: &str, payload: Value) -> Result<Value> {
        let url = format!("{}/{}", self.base_url, endpoint);
        
        debug!("Querying HelixDB: {} with payload: {}", url, payload);
        
        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send request to HelixDB")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("HelixDB query failed with status {}: {}", status, error_text);
        }

        let result = response
            .json::<Value>()
            .await
            .context("Failed to parse HelixDB response")?;

        Ok(result)
    }

    /// Initialize a new MCP session
    pub async fn init(&self) -> Result<String> {
        let result = self.query("mcp/init", serde_json::json!({})).await?;
        
        // Extract connection_id from response
        if let Some(connection_id) = result.as_str() {
            Ok(connection_id.to_string())
        } else if let Some(connection_id) = result.get("connection_id").and_then(|v| v.as_str()) {
            Ok(connection_id.to_string())
        } else {
            anyhow::bail!("Invalid response from mcp/init: missing connection_id")
        }
    }

    /// Get next item from session
    pub async fn next(&self, connection_id: &str) -> Result<Value> {
        self.query(
            "mcp/next",
            serde_json::json!({
                "connection_id": connection_id
            })
        ).await
    }

    /// Collect all items from session
    pub async fn collect(&self, connection_id: &str, range: Option<(usize, usize)>, drop: bool) -> Result<Value> {
        let mut payload = serde_json::json!({
            "connection_id": connection_id,
            "drop": drop
        });

        if let Some((start, end)) = range {
            payload["range"] = serde_json::json!({
                "start": start,
                "end": end
            });
        }

        self.query("mcp/collect", payload).await
    }

    /// Reset a session
    pub async fn reset(&self, connection_id: &str) -> Result<String> {
        let result = self.query(
            "mcp/reset",
            serde_json::json!({
                "connection_id": connection_id
            })
        ).await?;

        Ok(result.as_str().unwrap_or("Reset successful").to_string())
    }

    /// Get schema for connection
    pub async fn schema_resource(&self, connection_id: &str) -> Result<Value> {
        self.query(
            "mcp/schema_resource",
            serde_json::json!({
                "connection_id": connection_id
            })
        ).await
    }

    /// Get nodes by type
    pub async fn n_from_type(&self, connection_id: &str, node_type: &str) -> Result<Value> {
        self.query(
            "mcp/n_from_type",
            serde_json::json!({
                "connection_id": connection_id,
                "data": {
                    "node_type": node_type
                }
            })
        ).await
    }

    /// Get edges by type
    pub async fn e_from_type(&self, connection_id: &str, edge_type: &str) -> Result<Value> {
        self.query(
            "mcp/e_from_type",
            serde_json::json!({
                "connection_id": connection_id,
                "data": {
                    "edge_type": edge_type
                }
            })
        ).await
    }

    /// Traverse outward to nodes/vectors
    pub async fn out_step(&self, connection_id: &str, edge_label: &str, edge_type: &str) -> Result<Value> {
        self.query(
            "mcp/out_step",
            serde_json::json!({
                "connection_id": connection_id,
                "data": {
                    "edge_label": edge_label,
                    "edge_type": edge_type
                }
            })
        ).await
    }

    /// Traverse outward to edges
    pub async fn out_e_step(&self, connection_id: &str, edge_label: &str) -> Result<Value> {
        self.query(
            "mcp/out_e_step",
            serde_json::json!({
                "connection_id": connection_id,
                "data": {
                    "edge_label": edge_label
                }
            })
        ).await
    }

    /// Traverse inward to nodes/vectors
    pub async fn in_step(&self, connection_id: &str, edge_label: &str, edge_type: &str) -> Result<Value> {
        self.query(
            "mcp/in_step",
            serde_json::json!({
                "connection_id": connection_id,
                "data": {
                    "edge_label": edge_label,
                    "edge_type": edge_type
                }
            })
        ).await
    }

    /// Traverse inward to edges
    pub async fn in_e_step(&self, connection_id: &str, edge_label: &str) -> Result<Value> {
        self.query(
            "mcp/in_e_step",
            serde_json::json!({
                "connection_id": connection_id,
                "data": {
                    "edge_label": edge_label
                }
            })
        ).await
    }

    /// Filter items in traversal
    pub async fn filter_items(&self, connection_id: &str, filter: Value) -> Result<Value> {
        self.query(
            "mcp/filter_items",
            serde_json::json!({
                "connection_id": connection_id,
                "data": {
                    "filter": filter
                }
            })
        ).await
    }

    /// Vector similarity search
    pub async fn search_vector(&self, connection_id: &str, vector: Vec<f32>, k: usize, min_score: Option<f32>) -> Result<Value> {
        let mut data = serde_json::json!({
            "vector": vector,
            "k": k
        });

        if let Some(score) = min_score {
            data["min_score"] = serde_json::json!(score);
        }

        self.query(
            "mcp/search_vector",
            serde_json::json!({
                "connection_id": connection_id,
                "data": data
            })
        ).await
    }

    /// Text-based vector search (server-side embedding)
    pub async fn search_vector_text(&self, connection_id: &str, query: &str, label: &str) -> Result<Value> {
        self.query(
            "mcp/search_vector_text",
            serde_json::json!({
                "connection_id": connection_id,
                "data": {
                    "query": query,
                    "label": label
                }
            })
        ).await
    }

    /// BM25 keyword search
    pub async fn search_keyword(&self, connection_id: &str, query: &str, label: &str, limit: usize) -> Result<Value> {
        self.query(
            "mcp/search_keyword",
            serde_json::json!({
                "connection_id": connection_id,
                "data": {
                    "query": query,
                    "label": label,
                    "limit": limit
                }
            })
        ).await
    }

    /// Test connection to HelixDB
    pub async fn test_connection(&self) -> Result<()> {
        // Try to initialize a connection as a health check
        let _connection_id = self.init().await?;
        Ok(())
    }
}

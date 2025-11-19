//! TCP Embedding Client for MCP Server
//!
//! High-performance TCP client using OVNT protocol to communicate
//! with the standalone EmbeddingServer

use serde::{Deserialize, Serialize};
use std::io;
use tokio::net::TcpStream;
use uuid::Uuid;

/// OVNT Protocol magic bytes
const MAGIC_BYTES: [u8; 4] = [0x4F, 0x56, 0x4E, 0x54]; // "OVNT"
const VERSION: u8 = 0x01;
const MSG_TYPE_DATA: u8 = 4;

/// Embedding request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedRequest {
    pub text: String,
    pub model: Option<String>,
}

/// Embedding response - supports multiple formats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbedResponse {
    /// Direct array: [0.1, 0.2, ...]
    DirectArray(Vec<f32>),
    /// Wrapped: {"embedding": [...]}
    Wrapped { embedding: Vec<f32> },
    /// Alternative: {"vector": [...]}
    VectorWrapped { vector: Vec<f32> },
}

impl EmbedResponse {
    pub fn get_embedding(&self) -> &Vec<f32> {
        match self {
            EmbedResponse::DirectArray(v) => v,
            EmbedResponse::Wrapped { embedding } => embedding,
            EmbedResponse::VectorWrapped { vector } => vector,
        }
    }
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// TCP Embedding Client
pub struct EmbeddingClient {
    server_address: String,
    client_id: Uuid,
    timeout: std::time::Duration,
}

impl EmbeddingClient {
    /// Create a new embedding client
    pub fn new(server_address: String, timeout_secs: u64) -> Self {
        Self {
            server_address,
            client_id: Uuid::new_v4(),
            timeout: std::time::Duration::from_secs(timeout_secs),
        }
    }

    /// Validate text before generating embedding
    fn validate_text_for_embedding(&self, text: &str) -> Result<(), String> {
        if text.trim().is_empty() {
            return Err("Cannot generate embedding for empty text".to_string());
        }
        if text.len() < 3 {
            return Err("Text too short for meaningful embedding".to_string()); 
        }
        Ok(())
    }

    /// Validate embedding dimensions
    async fn validate_embedding(&self, embedding: &[f32]) -> Result<(), String> {
        // Check vector dimensions
        let expected_dim = 384; // Should match model output
        if embedding.len() != expected_dim {
            return Err(format!("Invalid embedding dimension: got {}, expected {}", 
                             embedding.len(), expected_dim));
        }
        Ok(())
    }

    /// Generate embedding for text using default model
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        self.embed_text_with_model(text, None).await
    }

    /// Generate embedding for text with specific model
    pub async fn embed_text_with_model(
        &self,
        text: &str,
        model: Option<String>,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // Validate input text
        self.validate_text_for_embedding(text)
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)) as Box<dyn std::error::Error>)?;

        // Connect to server
        let mut stream = tokio::time::timeout(
            self.timeout,
            TcpStream::connect(&self.server_address),
        )
        .await??;

        // Create request
        let request = EmbedRequest {
            text: text.to_string(),
            model,
        };

        // Serialize request
        let payload = rmp_serde::to_vec(&request)?;

        // Write OVNT protocol message
        self.write_protocol_message(&mut stream, payload).await?;

        // Read response
        let response_payload = self.read_protocol_message(&mut stream).await?;

        // Try to deserialize as EmbedResponse first
        if let Ok(response) = rmp_serde::from_slice::<EmbedResponse>(&response_payload) {
            let embedding = response.get_embedding().clone();
            // Validate embedding before returning
            self.validate_embedding(&embedding).await
                .map_err(|e| format!("Embedding validation failed: {}", e))?;
            Ok(embedding)
        } else if let Ok(error) = rmp_serde::from_slice::<ErrorResponse>(&response_payload) {
            Err(format!("Server error: {}", error.error).into())
        } else {
            Err("Invalid response format".into())
        }
    }

    /// Write OVNT protocol message
    async fn write_protocol_message(
        &self,
        stream: &mut TcpStream,
        payload: Vec<u8>,
    ) -> io::Result<()> {
        use tokio::io::AsyncWriteExt;

        // Magic bytes
        stream.write_all(&MAGIC_BYTES).await?;

        // Version
        stream.write_u8(VERSION).await?;

        // Message type
        stream.write_u8(MSG_TYPE_DATA).await?;

        // Length
        stream.write_u32_le(payload.len() as u32).await?;

        // Sender ID (client ID)
        stream.write_all(self.client_id.as_bytes()).await?;

        // Target ID (None for server)
        stream.write_u8(0).await?;

        // Message ID
        let message_id = Uuid::new_v4();
        stream.write_all(message_id.as_bytes()).await?;

        // Payload
        stream.write_all(&payload).await?;

        stream.flush().await?;
        Ok(())
    }

    /// Read OVNT protocol message
    async fn read_protocol_message(&self, stream: &mut TcpStream) -> io::Result<Vec<u8>> {
        use tokio::io::AsyncReadExt;

        // Read magic bytes
        let mut magic = [0u8; 4];
        stream.read_exact(&mut magic).await?;
        if magic != MAGIC_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid magic bytes: {:?}", magic),
            ));
        }

        // Read version
        let version = stream.read_u8().await?;
        if version != VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported version: {}", version),
            ));
        }

        // Read message type
        let _msg_type = stream.read_u8().await?;

        // Read length
        let length = stream.read_u32_le().await?;

        // Read sender ID
        let mut sender_bytes = [0u8; 16];
        stream.read_exact(&mut sender_bytes).await?;

        // Read target ID option
        let target_tag = stream.read_u8().await?;
        if target_tag == 1 {
            let mut target_bytes = [0u8; 16];
            stream.read_exact(&mut target_bytes).await?;
        }

        // Read message ID
        let mut message_bytes = [0u8; 16];
        stream.read_exact(&mut message_bytes).await?;

        // Read payload
        let mut payload = vec![0u8; length as usize];
        stream.read_exact(&mut payload).await?;

        Ok(payload)
    }

    /// Test connection to embedding server
    pub async fn test_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
        tokio::time::timeout(
            self.timeout,
            TcpStream::connect(&self.server_address),
        )
        .await??;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_embed_text() {
        // Load from config or use default
        let address = std::env::var("TCP_EMBEDDING_ADDRESS")
            .unwrap_or_else(|_| "127.0.0.1:8787".to_string());
        
        let client = EmbeddingClient::new(address, 30);
        let result = client.embed_text("Hello, world!").await;
        assert!(result.is_ok());
        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 384); // all-MiniLM-L6-v2 dimension
    }
}

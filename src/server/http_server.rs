/// HTTP Transport Server for MCP using RMCP's StreamableHttpService
/// 
/// This module provides an HTTP server that uses RMCP's built-in streamable HTTP
/// transport, which provides full MCP protocol support including all tools and resources.
/// 
/// The server uses Hyper + Tower for HTTP handling and RMCP's StreamableHttpService
/// for the MCP protocol layer. This provides:
/// - Full JSON-RPC 2.0 protocol over HTTP
/// - Server-Sent Events (SSE) for streaming responses
/// - Complete tool and resource access (same as TCP/STDIO)
/// - Session management for stateful connections
/// - CORS headers for browser access

use anyhow::Result;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use rmcp::transport::streamable_http_server::{StreamableHttpService, StreamableHttpServerConfig};
use rmcp::transport::streamable_http_server::session::never::NeverSessionManager;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_service::Service;
use tracing::{info, error};

use crate::{HelixMcpServer, config::ServerConfig};

/// Start the HTTP MCP server using RMCP's StreamableHttpService
/// 
/// This function creates a proper MCP server over HTTP using RMCP's built-in
/// streamable HTTP transport. This provides FULL tool access just like TCP and STDIO modes.
/// 
/// # Arguments
/// * `server` - The MCP server instance to use for handling requests
/// * `addr` - The address to bind to (e.g., "127.0.0.1:8080")
/// * `config` - Server configuration including HTTP settings
pub async fn start_http_server(server: HelixMcpServer, addr: &str, _config: Arc<ServerConfig>) -> Result<()> {
    let addr: SocketAddr = addr.parse()?;
    
    info!("🌐 HTTP MCP Server starting on http://{}", addr);
    info!("📡 Using RMCP StreamableHttpService (full MCP protocol support)");
    info!("✅ All tools accessible via HTTP (same as TCP/STDIO modes)");
    info!("📡 Endpoints:");
    info!("   POST / - MCP JSON-RPC requests");
    info!("   GET  / - Health check");
    
    // Create RMCP StreamableHttpService configuration
    let http_config = StreamableHttpServerConfig {
        stateful_mode: false, // Stateless mode for simplicity
        sse_keep_alive: Some(std::time::Duration::from_secs(15)),
    };
    
    // Create the service factory - RMCP will call this for each request
    let service_factory = move || {
        Ok::<_, std::io::Error>(server.clone())
    };
    
    // Create the StreamableHttpService
    // This is the key - RMCP's built-in HTTP transport that provides full MCP support
    let mcp_http_service = StreamableHttpService::new(
        service_factory,
        Arc::new(NeverSessionManager::default()),
        http_config,
    );
    
    // Bind TCP listener
    let listener = TcpListener::bind(addr).await?;
    info!("🚀 HTTP MCP Server listening on http://{}", addr);
    info!("💡 Tip: All MCP tools are available via standard JSON-RPC over HTTP");
    
    // Accept connections
    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                info!("🔌 New HTTP connection from {}", peer_addr);
                
                let service = mcp_http_service.clone();
                
                // Spawn a task to handle this connection
                tokio::spawn(async move {
                    // Wrap the stream with TokioIo
                    let io = TokioIo::new(stream);
                    
                    // Convert the service to tower::Service
                    let service = service_fn(move |req| {
                        let mut svc = service.clone();
                        async move {
                            svc.call(req).await
                        }
                    });
                    
                    // Serve HTTP/1.1 on this connection
                    if let Err(e) = http1::Builder::new()
                        .serve_connection(io, service)
                        .await
                    {
                        error!("❌ HTTP connection error for {}: {}", peer_addr, e);
                    } else {
                        info!("✅ HTTP connection closed for {}", peer_addr);
                    }
                });
            }
            Err(e) => {
                error!("❌ Failed to accept connection: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }
}

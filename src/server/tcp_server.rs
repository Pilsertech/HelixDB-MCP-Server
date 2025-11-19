/// TCP Transport Server for MCP
/// 
/// This module provides a simple TCP server that accepts connections
/// and serves the MCP protocol over TCP without any authentication.
/// 
/// Each connection is handled independently, making it easy for clients
/// to connect and start using MCP tools immediately.
/// 
/// Performance optimizations are configurable via mcpconfig.toml:
/// - tcp_nodelay: Disable Nagle's algorithm for low latency
/// - tcp_keepalive: Detect broken connections automatically

use anyhow::Result;
use rmcp::{serve_server};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, error};

use crate::{HelixMcpServer, config::ServerConfig};

/// Start the TCP MCP server
/// 
/// This function binds to the specified address and accepts connections.
/// Each connection spawns a new task to handle the MCP protocol.
/// 
/// # Arguments
/// * `server` - The MCP server instance to use for handling requests
/// * `addr` - The address to bind to (e.g., "127.0.0.1:8765")
/// * `config` - Server configuration including TCP optimization settings
pub async fn start_tcp_server(server: HelixMcpServer, addr: &str, config: Arc<ServerConfig>) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    info!("🌐 TCP MCP Server listening on {}", addr);
    info!("✅ Ready to accept connections (no authentication required)");
    info!("⚙️  TCP_NODELAY: {}", config.tcp_nodelay);
    info!("⚙️  TCP_KEEPALIVE: {}", config.tcp_keepalive);
    
    // Keep server alive in Arc for sharing across connections
    let server = Arc::new(server);
    
    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                info!("🔌 New connection from {}", peer_addr);
                
                // Clone the server and config for this connection
                let server_clone = server.clone();
                let config_clone = config.clone();
                
                // Spawn a task to handle this connection
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(server_clone, stream, peer_addr, config_clone).await {
                        error!("❌ Connection error for {}: {}", peer_addr, e);
                    } else {
                        info!("✅ Connection closed for {}", peer_addr);
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

/// Handle a single TCP connection
/// 
/// This function serves the MCP protocol over the given TCP stream.
/// The connection is kept alive until either side closes it.
/// 
/// Performance optimizations are applied based on config:
/// - TCP_NODELAY: Disables Nagle's algorithm (configurable)
/// - TCP_KEEPALIVE: Detects broken connections (configurable)
/// - Efficient async I/O: Uses tokio's zero-copy operations
async fn handle_connection(
    server: Arc<HelixMcpServer>,
    stream: TcpStream,
    peer_addr: std::net::SocketAddr,
    config: Arc<ServerConfig>,
) -> Result<()> {
    info!("📡 Serving MCP protocol to {}", peer_addr);
    
    // ============================================================================
    // TCP PERFORMANCE OPTIMIZATIONS (Configurable via mcpconfig.toml)
    // ============================================================================
    
    // 1. TCP_NODELAY: Disable Nagle's algorithm
    // Nagle buffers small packets for 40-200ms to batch them together
    // For interactive protocols like MCP, this adds unacceptable latency
    // Disabling it sends packets immediately = much lower latency
    if config.tcp_nodelay {
        stream.set_nodelay(true)?;
        info!("✅ TCP_NODELAY enabled for {} (zero buffering delay)", peer_addr);
    } else {
        info!("⚠️  TCP_NODELAY disabled for {} (Nagle's algorithm active)", peer_addr);
    }
    
    // 2. TCP_KEEPALIVE: Detect broken connections
    // Without this, dead connections can hang indefinitely
    // With this, the OS automatically detects and closes dead connections
    #[cfg(unix)]
    {
        if config.tcp_keepalive {
            use socket2::{Socket, TcpKeepalive};
            use std::time::Duration;
            
            // Convert to std::net::TcpStream to use socket2
            let std_stream = stream.into_std()?;
            let socket = Socket::from(std_stream);
            
            let keepalive = TcpKeepalive::new()
                .with_time(Duration::from_secs(config.tcp_keepalive_idle))
                .with_interval(Duration::from_secs(config.tcp_keepalive_interval));
            
            socket.set_tcp_keepalive(&keepalive)?;
            info!("✅ TCP_KEEPALIVE enabled for {} (idle={}s, interval={}s)", 
                  peer_addr, config.tcp_keepalive_idle, config.tcp_keepalive_interval);
            
            // Convert back to tokio TcpStream
            let stream = TcpStream::from_std(socket.into())?;
            
            // 3. Serve the MCP protocol
            let running_service = serve_server((*server).clone(), stream).await?;
            running_service.waiting().await?;
        } else {
            // Serve without keepalive
            let running_service = serve_server((*server).clone(), stream).await?;
            running_service.waiting().await?;
        }
    }
    
    #[cfg(windows)]
    {
        // Windows doesn't support socket2 TcpKeepalive the same way
        // Use basic settings or skip keepalive configuration
        if config.tcp_keepalive {
            info!("✅ TCP_KEEPALIVE using system defaults for {} (Windows)", peer_addr);
        }
        
        // Serve the MCP protocol
        let running_service = serve_server((*server).clone(), stream).await?;
        running_service.waiting().await?;
    }
    
    info!("🔌 Connection from {} closed normally", peer_addr);
    Ok(())
}

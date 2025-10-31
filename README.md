# HelixDB MCP Server (Rust Implementation)

A Rust implementation of the HelixDB MCP (Model Context Protocol) server, fully compatible with the Python [helix-py](https://github.com/HelixDB/helix-py) MCP server.

## Overview

This MCP server exposes HelixDB's graph traversal and search capabilities through the Model Context Protocol, allowing LLMs to interact with your HelixDB instance.

**Based on:** [HelixDB MCP Documentation](https://docs.helix-db.com/features/mcp/helix-mcp)

## Features

### Session Management Tools
- `init` - Initialize a new MCP traversal connection
- `next` - Get the next item in traversal results
- `collect` - Collect all items in traversal results
- `reset` - Reset the MCP traversal connection
- `schema_resource` - Get schema for the connection

### Graph Traversal Tools
- `n_from_type` - Retrieve all nodes of a given type
- `e_from_type` - Retrieve all edges of a given type
- `out_step` - Traverse outward from current nodes/vectors
- `out_e_step` - Traverse outward to edges
- `in_step` - Traverse inward to nodes/vectors
- `in_e_step` - Traverse inward to edges

### Filter and Search Tools
- `filter_items` - Filter current traversal state
- `search_vector_text` - Similarity search using text query
- `search_keyword` - BM25 keyword search

## Prerequisites

- Rust 1.70 or higher
- A running HelixDB instance (local or remote)

## Installation

### Build from source

```bash
cd helix_mcp_server
cargo build --release
```

The binary will be at `target/release/helix-mcp-server`

## Configuration

### Environment Variables

- `HELIX_ENDPOINT` - HelixDB server endpoint (default: `127.0.0.1`)
- `HELIX_PORT` - HelixDB server port (default: `6969`)
- `RUST_LOG` - Logging level (default: `info`, options: `debug`, `info`, `warn`, `error`)

## Usage

### Standalone Mode (stdio transport)

The server runs on stdio transport by default, which is compatible with Claude Desktop, Cursor, and other MCP clients:

```bash
# Set environment variables
$env:HELIX_ENDPOINT="127.0.0.1"
$env:HELIX_PORT="6969"

# Run the server
./target/release/helix-mcp-server
```

### Claude Desktop Configuration

Add to `%APPDATA%\Claude\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "helix-mcp": {
      "command": "E:\\path\\to\\helix-mcp-server.exe",
      "env": {
        "HELIX_ENDPOINT": "127.0.0.1",
        "HELIX_PORT": "6969"
      }
    }
  }
}
```

### Cursor Configuration

In Cursor Settings > MCP & Integrations > New MCP Server:

```json
{
  "mcpServers": {
    "helix-mcp": {
      "command": "E:\\path\\to\\helix-mcp-server.exe",
      "env": {
        "HELIX_ENDPOINT": "127.0.0.1",
        "HELIX_PORT": "6969"
      }
    }
  }
}
```

### Windsurf Configuration

Go to Cascade settings > MCP Servers > Manage MCPs > View raw config:

```json
{
  "mcpServers": {
    "helix-mcp": {
      "command": "E:\\path\\to\\helix-mcp-server.exe",
      "env": {
        "HELIX_ENDPOINT": "127.0.0.1",
        "HELIX_PORT": "6969"
      }
    }
  }
}
```

## Example Workflow

1. **Initialize a connection:**
   ```
   Call init() → Returns connection_id
   ```

2. **Retrieve nodes of a specific type:**
   ```
   Call n_from_type(connection_id, "User") → Returns first user node
   ```

3. **Traverse relationships:**
   ```
   Call out_step(connection_id, "Follows", "node") → Returns connected nodes
   ```

4. **Get more results:**
   ```
   Call next(connection_id) → Returns next item in traversal
   ```

5. **Collect all results:**
   ```
   Call collect(connection_id) → Returns all remaining items
   ```

6. **Filter results:**
   ```
   Call filter_items(connection_id, {filter_spec}) → Returns filtered items
   ```

7. **Search operations:**
   ```
   Call search_keyword(connection_id, "query text", "label", 10)
   Call search_vector_text(connection_id, "search text", "vec_label")
   ```

## Architecture

The server is structured as follows:

- `main.rs` - Server initialization and tool router setup
- `helix_client.rs` - HTTP client for HelixDB MCP endpoints
- `tools.rs` - MCP tool handler implementations
- `session.rs` - Session management (minimal, as HelixDB handles sessions)

## Comparison with Python Implementation

This Rust implementation provides:
- ✅ All tools from helix-py MCP server
- ✅ Same API and behavior
- ✅ Compatible with all MCP clients
- ✅ Better performance and lower memory usage
- ✅ Standalone binary (no Python runtime needed)
- ⚠️ Note: `search_vector` (with embedder) not yet implemented - use `search_vector_text` instead

## Troubleshooting

### "Failed to connect to HelixDB"

Make sure HelixDB is running:
```bash
# Check if HelixDB is running
curl http://127.0.0.1:6969/health
```

### Enable debug logging

```bash
$env:RUST_LOG="debug"
./helix-mcp-server
```

### MCP client not connecting

1. Check the path to the binary in your MCP client configuration
2. Ensure environment variables are set correctly
3. Look at stderr output for error messages

## Development

### Run in development mode

```bash
cargo run
```

### Run with debug logging

```bash
RUST_LOG=debug cargo run
```

### Run tests

```bash
cargo test
```

## Contributing

This implementation follows the official HelixDB MCP specification from the Python reference implementation.

## License

Same as HelixDB project

## Resources

- [HelixDB MCP Documentation](https://docs.helix-db.com/features/mcp/helix-mcp)
- [HelixDB Python SDK](https://github.com/HelixDB/helix-py)
- [Model Context Protocol](https://modelcontextprotocol.io/)

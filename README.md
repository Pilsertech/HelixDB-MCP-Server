# AI Memory Layer MCP Server

Business and customer intelligence system with semantic search and relationship tracking.

## What It Does

Stores and retrieves business information (products, services, locations, hours, policies, events) and customer data (behaviors, preferences, desires, rules, feedback) with AI-powered search.

**Built on:** HelixDB graph database with vector embeddings

## Key Features

- **Two Search Modes**: Keyword search (fast, exact) and semantic search (meaning-based)
- **Customer Interactions**: Track product/service engagement with reasons
- **Navigation System**: Store directions with compass bearings and accessibility info
- **Smart Updates**: Automatically maintains search indexes when data changes
- **Relationship Discovery**: Find connections between customers and products/services

## Quick Start

### 1. Prerequisites
- HelixDB server running (default: `127.0.0.1:6969`)
- Rust 1.70+ (for building from source)

### 2. Configuration
Create `mcpconfig.toml` (see `mcpconfig.example.toml`):

```toml
[helix]
endpoint = "127.0.0.1"
port = 6969

[embedding]
# Option 1: Let HelixDB generate embeddings (recommended)
enabled = false

# Option 2: Generate embeddings yourself
enabled = true
provider = "openai"  # or "gemini", "local", "tcp"
model = "text-embedding-3-small"
openai_api_key = "sk-..."
```

### 3. Build & Run

```bash
cargo build --release
./target/release/helix-mcp-server
```

Or use pre-built binary from releases.

## LLM Integration

### Claude Desktop
`%APPDATA%\Claude\claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "ai-memory": {
      "command": "E:\\path\\to\\helix-mcp-server.exe"
    }
  }
}
```

### Cursor / Windsurf
Add to MCP servers configuration:
```json
{
  "mcpServers": {
    "ai-memory": {
      "command": "E:\\path\\to\\helix-mcp-server.exe"
    }
  }
}
```

## Usage Examples

### Store Business Info
```
LLM: create_business_memory(
  business_id: "biz123",
  memory_type: "product",
  data: {
    product_id: "prod_001",
    name: "Wireless Headphones",
    price: 99.99,
    description: "Noise-canceling bluetooth headphones"
  }
)
```

### Search
```
# Keyword search (fast, exact)
LLM: search_bm25(
  query: "headphones",
  memory_types: ["products"]
)

# Semantic search (meaning-based)
LLM: search_semantic(
  query: "audio equipment for music lovers",
  memory_types: ["products"]
)
```

### Track Customer Interaction
```
LLM: create_customer_product_interaction(
  customer_id: "cust456",
  product_id: "prod_001",
  interaction_id: "int_789",
  interaction_type: "purchased",
  text_reason: "Looking for quality headphones for commute"
)
```

### Update with Auto-Refresh
```
LLM: update_business_memory(
  memory_id: "prod_001",
  memory_type: "product",
  updates: {
    composite_text: "Premium noise-canceling wireless headphones with 30hr battery"
  }
)
# Automatically updates search indexes
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

## Available Tools (22 total)

**Query & Search**
- `query_business_memory` / `query_customer_memory` - Filter by criteria
- `search_semantic` - Find by meaning
- `search_bm25` - Find by keywords (use for exact matches/IDs)
- `find_customer_insights` - Discover relationships

**Create**
- `create_business_memory` / `create_customer_memory` - Add memories
- `create_customer_product_interaction` / `create_customer_service_interaction` - Track interactions
- `create_navigation_hub` / `create_navigation_waypoint` / `create_direction_path` - Add directions

**Update**
- `update_business_memory` / `update_customer_memory` - Modify memories
- `update_interaction` / `update_navigation` - Modify interactions/directions

**Query Specialized**
- `query_customer_interactions` / `search_customer_interactions` - Find interactions
- `query_navigation` / `search_navigation` - Get directions

**Delete**
- `delete_memory` - Remove any memory type

**Advanced**
- `do_query` - Direct database queries (use primary tools first)

## Search Strategy

1. **Exact match?** Use `search_bm25` (product IDs, phone numbers, exact terms)
2. **Conceptual?** Use `search_semantic` (find similar products, related ideas)
3. **Not sure?** Try `search_bm25` first, then `search_semantic`

## Troubleshooting

**Connection fails:**
- Verify HelixDB is running: `netstat -an | findstr :6969` (Windows)
- Check `mcpconfig.toml` settings

**Search returns nothing:**
- Try both `search_bm25` and `search_semantic`
- Verify data exists with `query_business_memory` or `query_customer_memory`

**LLM doesn't see tools:**
- Restart LLM client after config changes
- Check server logs for errors

## Architecture

- **22 unified tools** route to **146+ database queries**
- Smart routing by parameters (no tool explosion)
- Automatic search index maintenance on updates
- Two embedding modes: self-managed or HelixDB-generated

## Documentation

- `IMPLEMENTATION_COMPLETE.md` - Full technical details
- `QUICK_REFERENCE.md` - Testing checklist and commands
- `SYSTEM_VALIDATION.md` - Current status report

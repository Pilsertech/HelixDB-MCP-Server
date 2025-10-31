use anyhow::Result;
use rmcp::{tool_router, tool, tool_handler, ServerHandler, serve_server, schemars, transport::stdio};
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ServerCapabilities, ServerInfo, AnnotateAble};
use rmcp::ErrorData as McpError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, error, warn};

mod helix_client;
mod session;
mod config;
mod embedding_client;

use helix_client::HelixClient;
use config::Config;

// ============================================================================
// HIGH-LEVEL DOMAIN-SPECIFIC TOOL PARAMETERS
// ============================================================================

// Query parameters
#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct QueryBusinessMemoryParam {
    business_id: String,
    memory_type: String,  // "products", "services", "locations", "hours", "social", "policies", "events", "all"
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct QueryCustomerMemoryParam {
    customer_id: String,
    memory_type: String,  // "behaviors", "preferences", "desires", "rules", "feedback", "all"
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<serde_json::Value>,
}

// Create parameters
#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct CreateBusinessMemoryParam {
    business_id: String,
    memory_type: String,  // "product", "service", "location", "hours", "social", "policy", "event"
    data: serde_json::Value,  // JSON object with memory-specific fields
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct CreateCustomerMemoryParam {
    customer_id: String,
    memory_type: String,  // "behavior", "preference", "desire", "rule", "feedback"
    data: serde_json::Value,  // JSON object with memory-specific fields
}

// Update parameters
#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct UpdateBusinessMemoryParam {
    memory_id: String,  // product_id, service_id, location_id, etc.
    memory_type: String,  // "product", "service", "location", "hours", "social", "policy", "event"
    updates: serde_json::Value,  // JSON object with fields to update
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct UpdateCustomerMemoryParam {
    memory_id: String,  // behavior_id, preference_id, desire_id, etc.
    memory_type: String,  // "behavior", "preference", "desire", "rule", "feedback"
    updates: serde_json::Value,  // JSON object with fields to update
}

// Delete parameters
#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct DeleteMemoryParam {
    memory_id: String,
    memory_type: String,  // "product", "service", "behavior", "preference", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    delete_embedding: Option<bool>,  // Whether to also delete embedding (default: true)
}

// Advanced: Direct query execution
#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct DoQueryParam {
    endpoint: String,  // Query name from queries.hx (e.g., "get_business_products")
    payload: serde_json::Value,  // JSON object with query parameters
}

// Search and insights parameters
#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct SearchSemanticParam {
    query: String,
    memory_types: Vec<String>,  // e.g., ["products", "preferences"]
    #[serde(skip_serializing_if = "Option::is_none")]
    business_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    customer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<i32>,
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct FindCustomerInsightsParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    customer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    product_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    service_id: Option<String>,
    relationship_type: String,  // "liked", "disliked", "used_service", "visited_location", "all"
}

#[derive(Clone)]
pub struct HelixMcpServer {
    helix_client: Arc<HelixClient>,
    config: Arc<Config>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl HelixMcpServer {
    fn new(helix_client: Arc<HelixClient>, config: Arc<Config>) -> Self {
        Self {
            helix_client,
            config,
            tool_router: Self::tool_router(),
        }
    }

    // ========================================================================
    // HIGH-LEVEL DOMAIN-SPECIFIC TOOLS FOR AI MEMORY LAYER
    // ========================================================================

    #[tool(description = "Query business memories - unified access to products, services, locations, hours, social media, policies, and events for a specific business")]
    async fn query_business_memory(&self, params: Parameters<QueryBusinessMemoryParam>) -> Result<CallToolResult, McpError> {
        let business_id = &params.0.business_id;
        let memory_type = &params.0.memory_type;
        
        info!("query_business_memory: business_id={}, type={}", business_id, memory_type);

        // Determine which query to execute based on memory_type
        let query_name = match memory_type.as_str() {
            "products" => "get_business_products",
            "services" => "get_business_services",
            "locations" => "get_business_locations",
            "hours" => "get_business_hours",
            "social" => "get_business_social_media",
            "policies" => "get_business_policies",
            "events" => "get_business_events",
            "all" => {
                // Return all business memory types
                let mut all_memories = json!({});
                
                // Query each type and collect results
                if let Ok(products) = self.helix_client.query(
                    "get_business_products",
                    json!({"business_id": business_id})
                ).await {
                    all_memories["products"] = products;
                }
                
                if let Ok(services) = self.helix_client.query(
                    "get_business_services",
                    json!({"business_id": business_id})
                ).await {
                    all_memories["services"] = services;
                }
                
                if let Ok(locations) = self.helix_client.query(
                    "get_business_locations",
                    json!({"business_id": business_id})
                ).await {
                    all_memories["locations"] = locations;
                }
                
                if let Ok(hours) = self.helix_client.query(
                    "get_business_hours",
                    json!({"business_id": business_id})
                ).await {
                    all_memories["hours"] = hours;
                }
                
                if let Ok(social) = self.helix_client.query(
                    "get_business_social_media",
                    json!({"business_id": business_id})
                ).await {
                    all_memories["social"] = social;
                }
                
                if let Ok(policies) = self.helix_client.query(
                    "get_business_policies",
                    json!({"business_id": business_id})
                ).await {
                    all_memories["policies"] = policies;
                }
                
                if let Ok(events) = self.helix_client.query(
                    "get_business_events",
                    json!({"business_id": business_id})
                ).await {
                    all_memories["events"] = events;
                }
                
                return Ok(CallToolResult::structured(all_memories));
            }
            _ => {
                return Ok(CallToolResult::structured_error(json!({
                    "error": format!("Invalid memory_type: {}. Valid types: products, services, locations, hours, social, policies, events, all", memory_type)
                })));
            }
        };

        // Execute the query
        let payload = json!({"business_id": business_id});
        
        match self.helix_client.query(query_name, payload).await {
            Ok(mut results) => {
                // Apply filters if provided
                if let Some(filters) = &params.0.filters {
                    results = self.apply_filters(results, filters);
                }
                
                Ok(CallToolResult::structured(json!({
                    "business_id": business_id,
                    "memory_type": memory_type,
                    "count": results.as_array().map(|a| a.len()).unwrap_or(0),
                    "data": results
                })))
            }
            Err(e) => {
                error!("query_business_memory failed: {}", e);
                Ok(CallToolResult::structured_error(json!({"error": e.to_string()})))
            }
        }
    }

    #[tool(description = "Query customer memories - unified access to behaviors, preferences, desires, rules, and feedback for a specific customer")]
    async fn query_customer_memory(&self, params: Parameters<QueryCustomerMemoryParam>) -> Result<CallToolResult, McpError> {
        let customer_id = &params.0.customer_id;
        let memory_type = &params.0.memory_type;
        
        info!("query_customer_memory: customer_id={}, type={}", customer_id, memory_type);

        // Determine which query to execute based on memory_type
        let query_name = match memory_type.as_str() {
            "behaviors" => "get_customer_behaviors",
            "preferences" => "get_customer_preferences",
            "desires" => "get_customer_desires",
            "rules" => "get_customer_rules",
            "feedback" => "get_customer_feedback",
            "all" => {
                // Return all customer memory types
                let mut all_memories = json!({});
                
                if let Ok(behaviors) = self.helix_client.query(
                    "get_customer_behaviors",
                    json!({"customer_id": customer_id})
                ).await {
                    all_memories["behaviors"] = behaviors;
                }
                
                if let Ok(preferences) = self.helix_client.query(
                    "get_customer_preferences",
                    json!({"customer_id": customer_id})
                ).await {
                    all_memories["preferences"] = preferences;
                }
                
                if let Ok(desires) = self.helix_client.query(
                    "get_customer_desires",
                    json!({"customer_id": customer_id})
                ).await {
                    all_memories["desires"] = desires;
                }
                
                if let Ok(rules) = self.helix_client.query(
                    "get_customer_rules",
                    json!({"customer_id": customer_id})
                ).await {
                    all_memories["rules"] = rules;
                }
                
                if let Ok(feedback) = self.helix_client.query(
                    "get_customer_feedback",
                    json!({"customer_id": customer_id})
                ).await {
                    all_memories["feedback"] = feedback;
                }
                
                return Ok(CallToolResult::structured(all_memories));
            }
            _ => {
                return Ok(CallToolResult::structured_error(json!({
                    "error": format!("Invalid memory_type: {}. Valid types: behaviors, preferences, desires, rules, feedback, all", memory_type)
                })));
            }
        };

        // Execute the query
        let payload = json!({"customer_id": customer_id});
        
        match self.helix_client.query(query_name, payload).await {
            Ok(mut results) => {
                // Apply filters if provided
                if let Some(filters) = &params.0.filters {
                    results = self.apply_filters(results, filters);
                }
                
                Ok(CallToolResult::structured(json!({
                    "customer_id": customer_id,
                    "memory_type": memory_type,
                    "count": results.as_array().map(|a| a.len()).unwrap_or(0),
                    "data": results
                })))
            }
            Err(e) => {
                error!("query_customer_memory failed: {}", e);
                Ok(CallToolResult::structured_error(json!({"error": e.to_string()})))
            }
        }
    }

    // Helper function to apply filters to results
    fn apply_filters(&self, results: serde_json::Value, filters: &serde_json::Value) -> serde_json::Value {
        // If results is not an array, return as-is
        let Some(array) = results.as_array() else {
            return results;
        };

        // Filter the array based on criteria
        let filtered: Vec<serde_json::Value> = array
            .iter()
            .filter(|item| self.matches_filters(item, filters))
            .cloned()
            .collect();

        json!(filtered)
    }

    fn matches_filters(&self, item: &serde_json::Value, filters: &serde_json::Value) -> bool {
        // Simple filter matching - can be extended
        let Some(filter_obj) = filters.as_object() else {
            return true;
        };

        for (key, value) in filter_obj {
            match item.get(key) {
                Some(item_value) => {
                    // Handle range filters for numbers
                    if let Some(range) = value.as_object() {
                        if let Some(lte) = range.get("lte") {
                            if let (Some(item_num), Some(filter_num)) = (item_value.as_f64(), lte.as_f64()) {
                                if item_num > filter_num {
                                    return false;
                                }
                            }
                        }
                        if let Some(gte) = range.get("gte") {
                            if let (Some(item_num), Some(filter_num)) = (item_value.as_f64(), gte.as_f64()) {
                                if item_num < filter_num {
                                    return false;
                                }
                            }
                        }
                    } else if item_value != value {
                        // Exact match for non-range filters
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }

    #[tool(description = "Semantic search across business and customer memories using AI embeddings - finds memories by meaning, not just keywords")]
    async fn search_semantic(&self, params: Parameters<SearchSemanticParam>) -> Result<CallToolResult, McpError> {
        let query = &params.0.query;
        let memory_types = &params.0.memory_types;
        let limit = params.0.limit.unwrap_or(10);
        
        info!("search_semantic: query='{}', types={:?}, limit={}", query, memory_types, limit);

        // Check embedding mode from config
        if self.config.is_helixdb_embedding_enabled() {
            // HelixDB mode: Just pass text query, HelixDB generates embedding via Embed()
            info!("Using HelixDB embedding mode (Embed() function in queries)");
            
            let mut all_results = Vec::new();
            
            // Search across requested memory types
            for memory_type in memory_types {
                let query_name = match memory_type.as_str() {
                    "products" => "search_business_products_semantic",
                    "preferences" => "search_customer_preferences_semantic",
                    _ => {
                        info!("Skipping unsupported memory type for semantic search: {}", memory_type);
                        continue;
                    }
                };

                // Build search payload - just text, HelixDB will call Embed()
                let mut payload = json!({
                    "query_text": query,
                    "k": limit,
                });

                // Add filters based on optional parameters
                if let Some(business_id) = &params.0.business_id {
                    payload["business_id"] = json!(business_id);
                }

                if let Some(customer_id) = &params.0.customer_id {
                    payload["customer_id"] = json!(customer_id);
                }

                // Execute query
                match self.helix_client.query(query_name, payload).await {
                    Ok(results) => {
                        if let Some(array) = results.as_array() {
                            all_results.extend(array.iter().cloned());
                        }
                    }
                    Err(e) => {
                        error!("Semantic search failed for {}: {}", memory_type, e);
                    }
                }
            }

            return Ok(CallToolResult::structured(json!({
                "query": query,
                "memory_types": memory_types,
                "total_results": all_results.len(),
                "limit": limit,
                "embedding_mode": "helixdb",
                "results": all_results
            })));
        }

        // MCP mode: MCP server generates embedding via API
        if self.config.is_mcp_embedding_enabled() {
            info!("Using MCP embedding mode (API calls in MCP server)");
            info!("Provider: {:?}", self.config.embedding.provider);
            
            // Check if we have API key (for cloud providers)
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            // Validate based on provider
            match self.config.embedding.provider {
                Some(config::EmbeddingProvider::OpenAI) | Some(config::EmbeddingProvider::Gemini) => {
                    if api_key.is_empty() {
                        error!("API key missing for cloud provider");
                        return Ok(CallToolResult::structured_error(json!({
                            "error": "API key not configured for cloud embedding provider",
                            "provider": format!("{:?}", self.config.embedding.provider),
                            "suggestion": "Set OPENAI_API_KEY or GEMINI_API_KEY environment variable, or add api_key to mcpconfig.toml"
                        })));
                    }
                }
                Some(config::EmbeddingProvider::Local) => {
                    info!("Using local HTTP embedding provider (no API key needed)");
                }
                Some(config::EmbeddingProvider::Tcp) => {
                    info!("Using TCP embedding server (no API key needed)");
                }
                None => {
                    error!("Embedding provider not configured");
                    return Ok(CallToolResult::structured_error(json!({
                        "error": "Embedding provider not configured",
                        "suggestion": "Set 'provider' in mcpconfig.toml to: openai, gemini, local, or tcp"
                    })));
                }
            }

            // Generate embedding from query text
            info!("Generating embedding for query: {}", query);
            let query_embedding = match self.generate_embedding(query, &api_key).await {
                Ok(embedding) => {
                    info!("? Generated embedding vector with {} dimensions", embedding.len());
                    embedding
                }
                Err(e) => {
                    error!("? Failed to generate embedding: {}", e);
                    return Ok(CallToolResult::structured_error(json!({
                        "error": format!("Embedding generation failed: {}", e),
                        "provider": format!("{:?}", self.config.embedding.provider),
                        "suggestion": "Check API key, network connection, and local server status"
                    })));
                }
            };

            let mut all_results = Vec::new();
            
            // Search across requested memory types using generated embedding
            for memory_type in memory_types {
                let query_name = match memory_type.as_str() {
                    "products" => "search_business_products_hybrid",
                    "preferences" => "search_customer_preferences_hybrid",
                    _ => {
                        info!("Skipping unsupported memory type for semantic search: {}", memory_type);
                        continue;
                    }
                };

                // Build search payload with embedding vector
                let mut payload = json!({
                    "query_embedding": query_embedding,
                    "limit": limit,
                });

                // Add filters based on optional parameters
                if let Some(business_id) = &params.0.business_id {
                    if memory_type == "products" {
                        payload["business_id"] = json!(business_id);
                        payload["min_price"] = json!(0.0);
                        payload["max_price"] = json!(1000000.0);
                    }
                }

                if let Some(customer_id) = &params.0.customer_id {
                    if memory_type == "preferences" {
                        payload["customer_id"] = json!(customer_id);
                    }
                }

                // Execute query with embedding
                match self.helix_client.query(query_name, payload).await {
                    Ok(results) => {
                        if let Some(array) = results.as_array() {
                            all_results.extend(array.iter().cloned());
                        }
                    }
                    Err(e) => {
                        error!("Semantic search failed for {}: {}", memory_type, e);
                    }
                }
            }

            return Ok(CallToolResult::structured(json!({
                "query": query,
                "memory_types": memory_types,
                "total_results": all_results.len(),
                "limit": limit,
                "embedding_mode": "mcp",
                "provider": format!("{:?}", self.config.embedding.provider),
                "model": self.config.embedding.model,
                "results": all_results
            })));
        }

        // Fallback (should never reach here)
        Ok(CallToolResult::structured_error(json!({
            "error": "Invalid embedding configuration. Check mcpconfig.toml"
        })))
    }

    #[tool(description = "Find customer insights - discover relationships between customers and products/services (likes, dislikes, usage) with reasons")]
    async fn find_customer_insights(&self, params: Parameters<FindCustomerInsightsParam>) -> Result<CallToolResult, McpError> {
        let relationship_type = &params.0.relationship_type;
        
        info!("find_customer_insights: relationship_type={}", relationship_type);

        // Build query based on what we're looking for
        let mut insights = json!({});

        match relationship_type.as_str() {
            "liked" | "disliked" | "used_service" | "visited_location" | "all" => {
                // These require traversing edges with embedded reasons
                // The schema has: CustomerLikedProduct, CustomerDislikedProduct, CustomerUsedService, CustomerVisitedLocation
                
                if let Some(customer_id) = &params.0.customer_id {
                    // Find what this customer liked/disliked/used
                    insights["customer_id"] = json!(customer_id);
                    
                    if relationship_type == "liked" || relationship_type == "all" {
                        // Get customer preferences
                        if let Ok(preferences) = self.helix_client.query(
                            "get_customer_preferences",
                            json!({"customer_id": customer_id})
                        ).await {
                            insights["liked_products"] = json!({
                                "note": "Customer preferences found. To get actual product relationships, traverse CustomerLikedProduct edges.",
                                "preferences": preferences
                            });
                        }
                    }

                    if relationship_type == "disliked" || relationship_type == "all" {
                        // Get negative feedback
                        if let Ok(feedback) = self.helix_client.query(
                            "get_customer_feedback",
                            json!({"customer_id": customer_id})
                        ).await {
                            // Filter for negative sentiment
                            if let Some(feedback_array) = feedback.as_array() {
                                let negative: Vec<_> = feedback_array
                                    .iter()
                                    .filter(|f| {
                                        f.get("sentiment")
                                            .and_then(|s| s.as_str())
                                            .map(|s| s == "negative")
                                            .unwrap_or(false)
                                    })
                                    .cloned()
                                    .collect();
                                
                                insights["disliked_items"] = json!({
                                    "count": negative.len(),
                                    "negative_feedback": negative
                                });
                            }
                        }
                    }

                    if relationship_type == "used_service" || relationship_type == "all" {
                        // Get behaviors related to service usage
                        if let Ok(behaviors) = self.helix_client.query(
                            "get_customer_behaviors",
                            json!({"customer_id": customer_id})
                        ).await {
                            if let Some(behaviors_array) = behaviors.as_array() {
                                let service_usage: Vec<_> = behaviors_array
                                    .iter()
                                    .filter(|b| {
                                        b.get("behavior_type")
                                            .and_then(|bt| bt.as_str())
                                            .map(|bt| bt == "interaction" || bt == "purchase")
                                            .unwrap_or(false)
                                    })
                                    .cloned()
                                    .collect();
                                
                                insights["service_usage"] = json!({
                                    "count": service_usage.len(),
                                    "behaviors": service_usage
                                });
                            }
                        }
                    }

                } else if let Some(product_id) = &params.0.product_id {
                    // Find which customers liked/disliked this product
                    insights["product_id"] = json!(product_id);
                    insights["note"] = json!("To find customers who liked/disliked this product, you need to traverse incoming CustomerLikedProduct/CustomerDislikedProduct edges. This requires graph traversal queries.");
                    
                    // Suggestion: Use low-level tools to traverse
                    insights["suggestion"] = json!({
                        "approach": "Use in_step tool to traverse CustomerLikedProduct edges from this product node",
                        "example": "in_step(connection_id, edge_label='CustomerLikedProduct', edge_type='...')"
                    });

                } else if let Some(service_id) = &params.0.service_id {
                    // Find which customers used this service
                    insights["service_id"] = json!(service_id);
                    insights["note"] = json!("To find customers who used this service, traverse incoming CustomerUsedService edges.");
                    
                    insights["suggestion"] = json!({
                        "approach": "Use in_step tool to traverse CustomerUsedService edges from this service node"
                    });

                } else {
                    return Ok(CallToolResult::structured_error(json!({
                        "error": "Must provide at least one of: customer_id, product_id, or service_id"
                    })));
                }

                Ok(CallToolResult::structured(insights))
            }
            _ => {
                Ok(CallToolResult::structured_error(json!({
                    "error": format!("Invalid relationship_type: {}. Valid types: liked, disliked, used_service, visited_location, all", relationship_type)
                })))
            }
        }
    }

    // ========================================================================
    // CREATE TOOLS - Add new memories
    // ========================================================================

    #[tool(description = "Create new business memory - add products, services, locations, hours, social media, policies, or events")]
    async fn create_business_memory(&self, params: Parameters<CreateBusinessMemoryParam>) -> Result<CallToolResult, McpError> {
        let business_id = &params.0.business_id;
        let memory_type = &params.0.memory_type;
        let mut data = params.0.data.clone();
        
        info!("create_business_memory: business_id={}, type={}", business_id, memory_type);

        // Check if embedding needs to be generated (MCP mode)
        if self.config.is_mcp_embedding_enabled() {
            // Get text_description from data
            let text_description = data.get("text_description")
                .and_then(|v| v.as_str())
                .ok_or_else(|| McpError::invalid_params("Missing text_description field", None))?;

            info!("Generating embedding for text_description...");
            
            // Get API key (empty for local provider)
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            // Generate embedding
            match self.generate_embedding(text_description, &api_key).await {
                Ok(embedding) => {
                    info!("? Generated {} dimensional embedding", embedding.len());
                    
                    // Add embedding to data
                    data["embedding"] = json!(embedding);
                    
                    // Add embedding model info
                    let model_name = match self.config.embedding.provider {
                        Some(config::EmbeddingProvider::OpenAI) | Some(config::EmbeddingProvider::Gemini) => {
                            self.config.embedding.model.clone().unwrap_or_else(|| "unknown".to_string())
                        }
                        Some(config::EmbeddingProvider::Local) => "local".to_string(),
                        Some(config::EmbeddingProvider::Tcp) => "tcp-local".to_string(),
                        None => "unknown".to_string()
                    };
                    data["embedding_model"] = json!(model_name);
                }
                Err(e) => {
                    error!("? Failed to generate embedding: {}", e);
                    return Ok(CallToolResult::structured_error(json!({
                        "error": format!("Failed to generate embedding: {}", e),
                        "suggestion": "Check embedding configuration and API connectivity"
                    })));
                }
            }
        } else {
            // HelixDB mode - embeddings should be in the data already or will be generated by HelixDB
            info!("Using HelixDB embedding mode - expecting embedding in data or HelixDB will generate it");
        }

        // Determine which query to execute based on memory_type
        let query_name = match memory_type.as_str() {
            "product" => "add_business_product_memory",
            "service" => "add_business_service_memory",
            "location" => "add_business_location_memory",
            "hours" => "add_business_hours_memory",
            "social" => "add_business_social_media_memory",
            "policy" => "add_business_policy_memory",
            "event" => "add_business_event_memory",
            _ => {
                return Ok(CallToolResult::structured_error(json!({
                    "error": format!("Invalid memory_type: {}. Valid types: product, service, location, hours, social, policy, event", memory_type)
                })));
            }
        };

        // Execute the query
        match self.helix_client.query(query_name, data).await {
            Ok(result) => {
                Ok(CallToolResult::structured(json!({
                    "success": true,
                    "memory_type": memory_type,
                    "business_id": business_id,
                    "embedding_mode": if self.config.is_mcp_embedding_enabled() { "mcp" } else { "helixdb" },
                    "result": result
                })))
            }
            Err(e) => {
                error!("create_business_memory failed: {}", e);
                Ok(CallToolResult::structured_error(json!({
                    "error": format!("Failed to create {} memory: {}", memory_type, e)
                })))
            }
        }
    }

    #[tool(description = "Create new customer memory - add behaviors, preferences, desires, rules, or feedback")]
    async fn create_customer_memory(&self, params: Parameters<CreateCustomerMemoryParam>) -> Result<CallToolResult, McpError> {
        let customer_id = &params.0.customer_id;
        let memory_type = &params.0.memory_type;
        let mut data = params.0.data.clone();
        
        info!("create_customer_memory: customer_id={}, type={}", customer_id, memory_type);

        // Check if embedding needs to be generated (MCP mode)
        if self.config.is_mcp_embedding_enabled() {
            // Get text_description from data
            let text_description = data.get("text_description")
                .and_then(|v| v.as_str())
                .ok_or_else(|| McpError::invalid_params("Missing text_description field", None))?;

            info!("Generating embedding for text_description...");
            
            // Get API key (empty for local provider)
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            // Generate embedding
            match self.generate_embedding(text_description, &api_key).await {
                Ok(embedding) => {
                    info!("? Generated {} dimensional embedding", embedding.len());
                    
                    // Add embedding to data
                    data["embedding"] = json!(embedding);
                    
                    // Add embedding model info
                    let model_name = match self.config.embedding.provider {
                        Some(config::EmbeddingProvider::OpenAI) | Some(config::EmbeddingProvider::Gemini) => {
                            self.config.embedding.model.clone().unwrap_or_else(|| "unknown".to_string())
                        }
                        Some(config::EmbeddingProvider::Local) => "local".to_string(),
                        Some(config::EmbeddingProvider::Tcp) => "tcp-local".to_string(),
                        None => "unknown".to_string()
                    };
                    data["embedding_model"] = json!(model_name);
                }
                Err(e) => {
                    error!("? Failed to generate embedding: {}", e);
                    return Ok(CallToolResult::structured_error(json!({
                        "error": format!("Failed to generate embedding: {}", e),
                        "suggestion": "Check embedding configuration and API connectivity"
                    })));
                }
            }
        } else {
            info!("Using HelixDB embedding mode - expecting embedding in data or HelixDB will generate it");
        }

        // Determine which query to execute based on memory_type
        let query_name = match memory_type.as_str() {
            "behavior" => "add_customer_behavior_memory",
            "preference" => "add_customer_preference_memory",
            "desire" => "add_customer_desire_memory",
            "rule" => "add_customer_rule_memory",
            "feedback" => "add_customer_feedback_memory",
            _ => {
                return Ok(CallToolResult::structured_error(json!({
                    "error": format!("Invalid memory_type: {}. Valid types: behavior, preference, desire, rule, feedback", memory_type)
                })));
            }
        };

        // Execute the query
        match self.helix_client.query(query_name, data).await {
            Ok(result) => {
                Ok(CallToolResult::structured(json!({
                    "success": true,
                    "memory_type": memory_type,
                    "customer_id": customer_id,
                    "embedding_mode": if self.config.is_mcp_embedding_enabled() { "mcp" } else { "helixdb" },
                    "result": result
                })))
            }
            Err(e) => {
                error!("create_customer_memory failed: {}", e);
                Ok(CallToolResult::structured_error(json!({
                    "error": format!("Failed to create {} memory: {}", memory_type, e)
                })))
            }
        }
    }

    // ========================================================================
    // UPDATE TOOLS - Modify existing memories
    // ========================================================================

    #[tool(description = "Update existing business memory - modify products, services, locations, hours, social media, policies, or events")]
    async fn update_business_memory(&self, params: Parameters<UpdateBusinessMemoryParam>) -> Result<CallToolResult, McpError> {
        let memory_id = &params.0.memory_id;
        let memory_type = &params.0.memory_type;
        let updates = &params.0.updates;
        
        info!("update_business_memory: memory_id={}, type={}", memory_id, memory_type);

        // Check if description is being updated and we need to regenerate embedding
        let description_changed = updates.get("description").is_some() || 
                                 updates.get("text_description").is_some();
        
        let mut embedding_regenerated = false;
        
        // If description changed and we're in MCP mode, regenerate embedding
        if description_changed && self.config.is_mcp_embedding_enabled() {
            info!("Description changed in MCP mode - regenerating embedding for {} {}", memory_type, memory_id);
            
            // Get the new description text
            let description_text = updates.get("description")
                .or(updates.get("text_description"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| McpError::invalid_request("Description field must be a string", None))?;
            
            // Get API key for embedding generation
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            // Generate new embedding
            match self.generate_embedding(description_text, &api_key).await {
                Ok(embedding) => {
                    info!("Generated new embedding, deleting old embedding node...");
                    
                    // Delete old embedding (follows pattern from delete queries)
                    let delete_query = match memory_type.as_str() {
                        "product" => "delete_product_embedding_edge_only",
                        "service" => "delete_service_embedding_edge_only",
                        "location" => "delete_location_embedding_edge_only",
                        "hours" => "delete_hours_embedding_edge_only",
                        "social" => "delete_social_embedding_edge_only",
                        "policy" => "delete_policy_embedding_edge_only",
                        "event" => "delete_event_embedding_edge_only",
                        _ => {
                            warn!("No embedding delete query for type: {}", memory_type);
                            ""
                        }
                    };
                    
                    if !delete_query.is_empty() {
                        let delete_payload = json!({
                            format!("{}_id", memory_type): memory_id
                        });
                        
                        match self.helix_client.query(delete_query, delete_payload).await {
                            Ok(_) => info!("Deleted old embedding edge"),
                            Err(e) => warn!("Failed to delete old embedding (may not exist): {}", e)
                        }
                    }
                    
                    // Create new embedding node and link it
                    // Note: This follows the same pattern as create_business_memory
                    let embedding_id = format!("{}_{}_emb_{}", memory_type, memory_id, chrono::Utc::now().timestamp());
                    let model_name = match self.config.embedding.provider {
                        Some(_) => {
                            self.config.embedding.model.clone().unwrap_or_else(|| "unknown".to_string())
                        }
                        None => "unknown".to_string()
                    };
                    
                    let add_embedding_payload = json!({
                        "embedding_id": embedding_id,
                        "embedding": embedding,
                        "model": model_name,
                        "created_at": chrono::Utc::now().timestamp()
                    });
                    
                    // Add vector node
                    let add_vector_query = format!("add_{}_embedding_vector", memory_type);
                    if let Err(e) = self.helix_client.query(&add_vector_query, add_embedding_payload).await {
                        error!("Failed to create new embedding vector: {}", e);
                        // Continue anyway - update the main node even if embedding fails
                    } else {
                        // Link the embedding to the memory node
                        let link_payload = json!({
                            format!("{}_id", memory_type): memory_id,
                            "embedding_id": embedding_id
                        });
                        
                        let link_query = format!("link_{}_to_embedding", memory_type);
                        if let Err(e) = self.helix_client.query(&link_query, link_payload).await {
                            error!("Failed to link new embedding: {}", e);
                        } else {
                            info!("Successfully regenerated and linked new embedding");
                            embedding_regenerated = true;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to generate new embedding: {}", e);
                    // Continue with update even if embedding generation fails
                }
            }
        }

        // Determine which update query to use based on memory_type and fields
        let query_name = match memory_type.as_str() {
            "product" => {
                // Check which fields are being updated
                if updates.get("price").is_some() && updates.as_object().map(|o| o.len()) == Some(1) {
                    "update_product_price"
                } else if updates.get("availability").is_some() && updates.as_object().map(|o| o.len()) == Some(1) {
                    "update_product_availability"
                } else {
                    "update_product_full"
                }
            },
            "service" => {
                if updates.get("price").is_some() && updates.as_object().map(|o| o.len()) == Some(1) {
                    "update_service_price"
                } else if updates.get("availability").is_some() && updates.as_object().map(|o| o.len()) == Some(1) {
                    "update_service_availability"
                } else {
                    "update_service_full"
                }
            },
            "location" => "update_location_address",
            "hours" => "update_business_hours_monday",
            "social" => "update_social_stats",
            "policy" => "update_policy_content",
            "event" => "update_event_dates",
            _ => {
                return Ok(CallToolResult::structured_error(json!({
                    "error": format!("Invalid memory_type: {}. Valid types: product, service, location, hours, social, policy, event", memory_type)
                })));
            }
        };

        // Merge memory_id with updates
        let mut payload = updates.clone();
        if let Some(obj) = payload.as_object_mut() {
            obj.insert(format!("{}_id", memory_type), json!(memory_id));
            obj.insert("updated_at".to_string(), json!(chrono::Utc::now().timestamp()));
        }

        // Execute the query
        match self.helix_client.query(query_name, payload).await {
            Ok(result) => {
                Ok(CallToolResult::structured(json!({
                    "success": true,
                    "memory_type": memory_type,
                    "memory_id": memory_id,
                    "embedding_regenerated": embedding_regenerated,
                    "result": result
                })))
            }
            Err(e) => {
                error!("update_business_memory failed: {}", e);
                Ok(CallToolResult::structured_error(json!({
                    "error": format!("Failed to update {} memory: {}", memory_type, e)
                })))
            }
        }
    }

    #[tool(description = "Update existing customer memory - modify behaviors, preferences, desires, rules, or feedback")]
    async fn update_customer_memory(&self, params: Parameters<UpdateCustomerMemoryParam>) -> Result<CallToolResult, McpError> {
        let memory_id = &params.0.memory_id;
        let memory_type = &params.0.memory_type;
        let updates = &params.0.updates;
        
        info!("update_customer_memory: memory_id={}, type={}", memory_id, memory_type);

        // Check if text fields are being updated and we need to regenerate embedding
        let text_changed = updates.get("text_description").is_some() || 
                          updates.get("context").is_some() ||
                          updates.get("subject").is_some();
        
        let mut embedding_regenerated = false;
        
        // If text changed and we're in MCP mode, regenerate embedding
        if text_changed && self.config.is_mcp_embedding_enabled() {
            info!("Text content changed in MCP mode - regenerating embedding for {} {}", memory_type, memory_id);
            
            // Get the new text content
            let text_content = updates.get("text_description")
                .or(updates.get("context"))
                .or(updates.get("subject"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| McpError::invalid_request("Text field must be a string", None))?;
            
            // Get API key for embedding generation
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            // Generate new embedding
            match self.generate_embedding(text_content, &api_key).await {
                Ok(embedding) => {
                    info!("Generated new embedding, deleting old embedding node...");
                    
                    // Delete old embedding (follows pattern from delete queries)
                    let delete_query = match memory_type.as_str() {
                        "behavior" => "delete_behavior_embedding_edge_only",
                        "preference" => "delete_preference_embedding_edge_only",
                        "desire" => "delete_desire_embedding_edge_only",
                        "rule" => "delete_rule_embedding_edge_only",
                        "feedback" => "delete_feedback_embedding_edge_only",
                        _ => {
                            warn!("No embedding delete query for type: {}", memory_type);
                            ""
                        }
                    };
                    
                    if !delete_query.is_empty() {
                        let delete_payload = json!({
                            format!("{}_id", memory_type): memory_id
                        });
                        
                        match self.helix_client.query(delete_query, delete_payload).await {
                            Ok(_) => info!("Deleted old embedding edge"),
                            Err(e) => warn!("Failed to delete old embedding (may not exist): {}", e)
                        }
                    }
                    
                    // Create new embedding node and link it
                    let embedding_id = format!("{}_{}_emb_{}", memory_type, memory_id, chrono::Utc::now().timestamp());
                    let model_name = match self.config.embedding.provider {
                        Some(_) => {
                            self.config.embedding.model.clone().unwrap_or_else(|| "unknown".to_string())
                        }
                        None => "unknown".to_string()
                    };
                    
                    let add_embedding_payload = json!({
                        "embedding_id": embedding_id,
                        "embedding": embedding,
                        "model": model_name,
                        "created_at": chrono::Utc::now().timestamp()
                    });
                    
                    // Add vector node
                    let add_vector_query = format!("add_{}_embedding_vector", memory_type);
                    if let Err(e) = self.helix_client.query(&add_vector_query, add_embedding_payload).await {
                        error!("Failed to create new embedding vector: {}", e);
                        // Continue anyway - update the main node even if embedding fails
                    } else {
                        // Link the embedding to the memory node
                        let link_payload = json!({
                            format!("{}_id", memory_type): memory_id,
                            "embedding_id": embedding_id
                        });
                        
                        let link_query = format!("link_{}_to_embedding", memory_type);
                        if let Err(e) = self.helix_client.query(&link_query, link_payload).await {
                            error!("Failed to link new embedding: {}", e);
                        } else {
                            info!("Successfully regenerated and linked new embedding");
                            embedding_regenerated = true;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to generate new embedding: {}", e);
                    // Continue with update even if embedding generation fails
                }
            }
        }

        // Determine which update query to use
        let query_name = match memory_type.as_str() {
            "behavior" => "update_behavior_context",
            "preference" => "update_preference_strength",
            "desire" => "update_desire_priority",
            "rule" => "update_rule_enforcement",
            "feedback" => "update_feedback_rating",
            _ => {
                return Ok(CallToolResult::structured_error(json!({
                    "error": format!("Invalid memory_type: {}. Valid types: behavior, preference, desire, rule, feedback", memory_type)
                })));
            }
        };

        // Merge memory_id with updates
        let mut payload = updates.clone();
        if let Some(obj) = payload.as_object_mut() {
            obj.insert(format!("{}_id", memory_type), json!(memory_id));
            obj.insert("updated_at".to_string(), json!(chrono::Utc::now().timestamp()));
        }

        // Execute the query
        match self.helix_client.query(query_name, payload).await {
            Ok(result) => {
                Ok(CallToolResult::structured(json!({
                    "success": true,
                    "memory_type": memory_type,
                    "memory_id": memory_id,
                    "embedding_regenerated": embedding_regenerated,
                    "result": result
                })))
            }
            Err(e) => {
                error!("update_customer_memory failed: {}", e);
                Ok(CallToolResult::structured_error(json!({
                    "error": format!("Failed to update {} memory: {}", memory_type, e)
                })))
            }
        }
    }

    // ========================================================================
    // DELETE TOOLS - Remove memories
    // ========================================================================

    #[tool(description = "Delete memory - remove products, services, locations, behaviors, preferences, etc.")]
    async fn delete_memory(&self, params: Parameters<DeleteMemoryParam>) -> Result<CallToolResult, McpError> {
        let memory_id = &params.0.memory_id;
        let memory_type = &params.0.memory_type;
        let delete_embedding = params.0.delete_embedding.unwrap_or(true);
        
        info!("delete_memory: memory_id={}, type={}, delete_embedding={}", memory_id, memory_type, delete_embedding);

        // Determine which delete query to use
        let query_name = match memory_type.as_str() {
            "product" => if delete_embedding { "delete_product_with_embedding" } else { "delete_product" },
            "service" => if delete_embedding { "delete_service_with_embedding" } else { "delete_service" },
            "location" => if delete_embedding { "delete_location_with_embedding" } else { "delete_location" },
            "hours" => if delete_embedding { "delete_hours_with_embedding" } else { "delete_hours" },
            "social" => if delete_embedding { "delete_social_with_embedding" } else { "delete_social" },
            "policy" => if delete_embedding { "delete_policy_with_embedding" } else { "delete_policy" },
            "event" => if delete_embedding { "delete_event_with_embedding" } else { "delete_event" },
            "behavior" => if delete_embedding { "delete_behavior_with_embedding" } else { "delete_behavior" },
            "preference" => if delete_embedding { "delete_preference_with_embedding" } else { "delete_preference" },
            "desire" => if delete_embedding { "delete_desire_with_embedding" } else { "delete_desire" },
            "rule" => if delete_embedding { "delete_rule_with_embedding" } else { "delete_rule" },
            "feedback" => if delete_embedding { "delete_feedback_with_embedding" } else { "delete_feedback" },
            _ => {
                return Ok(CallToolResult::structured_error(json!({
                    "error": format!("Invalid memory_type: {}. Valid types: product, service, location, hours, social, policy, event, behavior, preference, desire, rule, feedback", memory_type)
                })));
            }
        };

        // Create payload with memory_id
        let payload = json!({
            format!("{}_id", memory_type): memory_id
        });

        // Execute the query
        match self.helix_client.query(query_name, payload).await {
            Ok(result) => {
                Ok(CallToolResult::structured(json!({
                    "success": true,
                    "memory_type": memory_type,
                    "memory_id": memory_id,
                    "deleted_embedding": delete_embedding,
                    "result": result
                })))
            }
            Err(e) => {
                error!("delete_memory failed: {}", e);
                Ok(CallToolResult::structured_error(json!({
                    "error": format!("Failed to delete {} memory: {}", memory_type, e)
                })))
            }
        }
    }

    // ========================================================================
    // ADVANCED TOOL - Direct query execution (last resort)
    // ========================================================================

    #[tool(description = "ADVANCED: Execute any HelixDB query directly - USE ONLY when high-level tools (query_business_memory, create_customer_memory, etc.) cannot accomplish the task. This is a last resort for complex queries not covered by standard tools.")]
    async fn do_query(&self, params: Parameters<DoQueryParam>) -> Result<CallToolResult, McpError> {
        let endpoint = &params.0.endpoint;
        let payload = &params.0.payload;
        
        info!("do_query: endpoint={}", endpoint);

        // Whitelist of allowed queries (security - prevent dangerous operations)
        let allowed_queries = vec![
            // Business product queries
            "add_business_product_memory",
            "get_business_products",
            "search_business_products",
            "search_business_products_hybrid",
            "update_product_price",
            "update_product_availability",
            "update_product_full",
            "delete_product",
            "delete_product_with_embedding",
            
            // Business service queries
            "add_business_service_memory",
            "get_business_services",
            "search_business_services",
            "update_service_price",
            "update_service_availability",
            "update_service_full",
            "delete_service",
            "delete_service_with_embedding",
            
            // Business location queries
            "add_business_location_memory",
            "get_business_locations",
            "update_location_address",
            "delete_location",
            "delete_location_with_embedding",
            
            // Business hours queries
            "add_business_hours_memory",
            "get_business_hours",
            "update_business_hours_monday",
            "delete_hours",
            "delete_hours_with_embedding",
            
            // Business social media queries
            "add_business_social_media_memory",
            "get_business_social_media",
            "update_social_stats",
            "delete_social",
            "delete_social_with_embedding",
            
            // Business policy queries
            "add_business_policy_memory",
            "get_business_policies",
            "update_policy_content",
            "delete_policy",
            "delete_policy_with_embedding",
            
            // Business event queries
            "add_business_event_memory",
            "get_business_events",
            "update_event_dates",
            "delete_event",
            "delete_event_with_embedding",
            
            // Customer behavior queries
            "add_customer_behavior_memory",
            "get_customer_behaviors",
            "update_behavior_context",
            "delete_behavior",
            "delete_behavior_with_embedding",
            
            // Customer preference queries
            "add_customer_preference_memory",
            "get_customer_preferences",
            "search_customer_preferences",
            "search_customer_preferences_hybrid",
            "update_preference_strength",
            "delete_preference",
            "delete_preference_with_embedding",
            
            // Customer desire queries
            "add_customer_desire_memory",
            "get_customer_desires",
            "update_desire_priority",
            "delete_desire",
            "delete_desire_with_embedding",
            
            // Customer rule queries
            "add_customer_rule_memory",
            "get_customer_rules",
            "update_rule_enforcement",
            "delete_rule",
            "delete_rule_with_embedding",
            
            // Customer feedback queries
            "add_customer_feedback_memory",
            "get_customer_feedback",
            "update_feedback_rating",
            "delete_feedback",
            "delete_feedback_with_embedding",
        ];

        // Validate endpoint is allowed
        if !allowed_queries.contains(&endpoint.as_str()) {
            return Ok(CallToolResult::structured_error(json!({
                "error": format!("Query '{}' is not allowed", endpoint),
                "allowed_queries": allowed_queries,
                "suggestion": "Use high-level tools (query_business_memory, create_customer_memory, etc.) instead"
            })));
        }

        // Execute query directly
        match self.helix_client.query(endpoint, payload.clone()).await {
            Ok(result) => {
                Ok(CallToolResult::structured(json!({
                    "endpoint": endpoint,
                    "result": result
                })))
            }
            Err(e) => {
                error!("do_query failed: {}", e);
                Ok(CallToolResult::structured_error(json!({
                    "error": format!("Query execution failed: {}", e),
                    "endpoint": endpoint
                })))
            }
        }
    }

    // ========================================================================
    // EMBEDDING GENERATION (MCP Mode)
    // ========================================================================

    /// Generate embedding vector from text using configured provider
    async fn generate_embedding(&self, text: &str, api_key: &str) -> Result<Vec<f32>, String> {
        use config::{EmbeddingProvider, EmbeddingMode};

        // Only generate embeddings in MCP mode
        if self.config.embedding.mode != EmbeddingMode::Mcp {
            return Err("generate_embedding called in non-MCP mode".to_string());
        }

        let provider = self.config.embedding.provider.as_ref()
            .ok_or("No embedding provider configured")?;

        match provider {
            EmbeddingProvider::OpenAI => {
                self.generate_openai_embedding(text, api_key).await
            }
            EmbeddingProvider::Gemini => {
                // Gemini now uses OpenAI-compatible format
                self.generate_openai_embedding(text, api_key).await
            }
            EmbeddingProvider::Local => {
                self.generate_local_embedding(text).await
            }
            EmbeddingProvider::Tcp => {
                self.generate_tcp_embedding(text).await
            }
        }
    }

    /// Generate embedding using OpenAI-compatible API
    /// Works with: OpenAI, Novita AI, Together AI, OpenRouter, Gemini (via OpenAI proxy), etc.
    async fn generate_openai_embedding(&self, text: &str, api_key: &str) -> Result<Vec<f32>, String> {
        let model = self.config.embedding.model.as_ref()
            .ok_or("Embedding model not configured")?;

        let api_url = self.config.embedding.openai_api_url.as_ref()
            .ok_or("OpenAI API URL not configured in mcpconfig.toml")?;

        info!("Generating embedding with model: {} at {}", model, api_url);

        let client = reqwest::Client::new();
        let response = client
            .post(api_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": model,
                "input": text,
                "encoding_format": "float"
            }))
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, error_text));
        }

        let json_response: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse API response: {}", e))?;

        // Extract embedding from OpenAI-compatible response
        let embedding = json_response["data"][0]["embedding"]
            .as_array()
            .ok_or("Invalid response: missing data[0].embedding array")?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect::<Vec<f32>>();

        if embedding.is_empty() {
            return Err("API returned empty embedding".to_string());
        }

        Ok(embedding)
    }

    /// Generate embedding using local embedding model (simple mode)
    /// Sends: {"text": "your text"}
    /// Expects: [0.1, 0.2, 0.3, ...] or {"embedding": [0.1, 0.2, ...]}
    async fn generate_local_embedding(&self, text: &str) -> Result<Vec<f32>, String> {
        let api_url = self.config.embedding.local_api_url.as_ref()
            .ok_or("Local API URL not configured in mcpconfig.toml")?;

        info!("Generating local embedding at {}", api_url);

        let client = reqwest::Client::new();
        
        // Send simple JSON: just the text
        let response = client
            .post(api_url)
            .header("Content-Type", "application/json")
            .json(&json!({
                "text": text
            }))
            .send()
            .await
            .map_err(|e| format!("Local API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Local API error {}: {}", status, error_text));
        }

        let json_response: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse local API response: {}", e))?;

        // Try direct array format first: [0.1, 0.2, 0.3, ...]
        if let Some(embedding_array) = json_response.as_array() {
            let embedding = embedding_array
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect::<Vec<f32>>();
            
            if !embedding.is_empty() {
                info!("Parsed direct array format, {} dimensions", embedding.len());
                return Ok(embedding);
            }
        }

        // Try wrapped format: {"embedding": [0.1, 0.2, ...]}
        if let Some(embedding_array) = json_response["embedding"].as_array() {
            let embedding = embedding_array
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect::<Vec<f32>>();
            
            if !embedding.is_empty() {
                info!("Parsed wrapped format, {} dimensions", embedding.len());
                return Ok(embedding);
            }
        }

        // Try {"vector": [...]} format
        if let Some(embedding_array) = json_response["vector"].as_array() {
            let embedding = embedding_array
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect::<Vec<f32>>();
            
            if !embedding.is_empty() {
                info!("Parsed vector format, {} dimensions", embedding.len());
                return Ok(embedding);
            }
        }

        Err(format!(
            "Invalid local API response. Expected: [0.1, 0.2, ...] or {{\"embedding\": [...]}} or {{\"vector\": [...]}}. Got: {}",
            serde_json::to_string(&json_response).unwrap_or_default().chars().take(200).collect::<String>()
        ))
    }

    async fn generate_tcp_embedding(&self, text: &str) -> Result<Vec<f32>, String> {
        let tcp_addr = self.config.embedding.tcp_address.as_ref()
            .ok_or("TCP address not configured in mcpconfig.toml")?;

        info!("Generating TCP embedding at {}", tcp_addr);

        // Use the embedding_client module
        let client = embedding_client::EmbeddingClient::new(
            tcp_addr.clone(), 
            self.config.embedding.tcp_timeout_secs
        );

        // Connect and generate embedding
        let embedding = client.embed_text(text).await
            .map_err(|e| format!("TCP embedding request failed: {}", e))?;

        info!("TCP embedding generated: {} dimensions", embedding.len());
        Ok(embedding)
    }
}


#[tool_handler]
impl ServerHandler for HelixMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            instructions: Some(
                "AI Memory Layer MCP Server - Provides intelligent access to business and customer memories.\n\n\
                PRIMARY TOOLS (Use These First):\n\
                - query_business_memory: Get products, services, locations, hours, social media, policies, or events\n\
                - query_customer_memory: Get behaviors, preferences, desires, rules, or feedback\n\
                - create_business_memory: Add products, services, locations, hours, social media, policies, or events\n\
                - create_customer_memory: Add behaviors, preferences, desires, rules, or feedback\n\
                - update_business_memory: Update products, services, locations, hours, social media, policies, or events\n\
                - update_customer_memory: Update behaviors, preferences, desires, rules, or feedback\n\
                - delete_memory: Remove any memory type (business or customer)\n\
                - search_semantic: AI-powered semantic search across memories\n\
                - find_customer_insights: Discover customer-product/service relationships\n\n\
                ADVANCED TOOL (Last Resort Only):\n\
                - do_query: Direct HelixDB query execution - use ONLY when primary tools cannot accomplish the task\n\n\
                Total: 10 tools (9 primary + 1 advanced).".to_string()
            ),
            ..Default::default()
        }
    }

    async fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListResourcesResult, McpError> {
        use rmcp::model::RawResource;
        
        let mut about = RawResource::new("meta://about", "About AI Memory Layer");
        about.description = Some("Information about the AI Memory Layer MCP Server and its capabilities".to_string());
        about.mime_type = Some("text/plain".to_string());

        let mut instructions = RawResource::new("meta://instructions", "Usage Instructions");
        instructions.description = Some("Detailed instructions for using the AI Memory Layer MCP Server tools".to_string());
        instructions.mime_type = Some("text/plain".to_string());

        let mut schema = RawResource::new("meta://schema", "Memory Schema");
        schema.description = Some("Complete schema documentation for business and customer memories".to_string());
        schema.mime_type = Some("text/plain".to_string());
        
        Ok(rmcp::model::ListResourcesResult {
            resources: vec![
                about.no_annotation(),
                instructions.no_annotation(),
                schema.no_annotation(),
            ],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        request: rmcp::model::ReadResourceRequestParam,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ReadResourceResult, McpError> {
        use rmcp::model::ResourceContents;
        
        let uri = &request.uri;
        let content = match uri.as_str() {
            "meta://about" => {
                "# AI Memory Layer MCP Server\n\n\
                Version: 0.1.0\n\n\
                ## Overview\n\
                MCP server providing access to business and customer memories stored in HelixDB.\n\n\
                ## Tools\n\
                - 4 Query Tools: Read business/customer data\n\
                - 2 Create Tools: Add new memories\n\
                - 2 Update Tools: Modify existing memories\n\
                - 1 Delete Tool: Remove memories\n\
                - 1 Advanced Tool: Direct query execution (last resort)\n\n\
                ## Memory Types\n\
                **Business**: Products, Services, Locations, Hours, Social Media, Policies, Events\n\
                **Customer**: Behaviors, Preferences, Desires, Rules, Feedback\n\n\
                ## Features\n\
                - Filtering support\n\
                - Vector embeddings for semantic search\n\
                - Relationship queries"
            },
            "meta://instructions" => {
                "# AI Memory Layer - Usage Instructions\n\n\
                ## Tool Hierarchy\n\n\
                **USE PRIMARY TOOLS FIRST** (covers 95% of use cases):\n\
                - query_business_memory, query_customer_memory\n\
                - create_business_memory, create_customer_memory\n\
                - update_business_memory, update_customer_memory\n\
                - delete_memory\n\
                - search_semantic, find_customer_insights\n\n\
                **USE ADVANCED TOOL ONLY AS LAST RESORT**:\n\
                - do_query: When primary tools cannot accomplish the task\n\n\
                ## Examples\n\n\
                ### Query Business Memories\n\
                ```json\n\
                {\n\
                  \"business_id\": \"BIZ123\",\n\
                  \"memory_type\": \"products\",\n\
                  \"filters\": { \"price\": { \"lte\": 100.0 } }\n\
                }\n\
                ```\n\n\
                ### Create New Memory\n\
                ```json\n\
                {\n\
                  \"business_id\": \"BIZ123\",\n\
                  \"memory_type\": \"product\",\n\
                  \"data\": {\n\
                    \"product_id\": \"PROD001\",\n\
                    \"product_name\": \"Eco Bottle\",\n\
                    \"price\": 25.99,\n\
                    \"currency\": \"USD\"\n\
                  }\n\
                }\n\
                ```\n\n\
                ### Update Memory\n\
                ```json\n\
                {\n\
                  \"memory_id\": \"PROD001\",\n\
                  \"memory_type\": \"product\",\n\
                  \"updates\": { \"price\": 29.99 }\n\
                }\n\
                ```\n\n\
                ### Delete Memory\n\
                ```json\n\
                {\n\
                  \"memory_id\": \"PROD001\",\n\
                  \"memory_type\": \"product\",\n\
                  \"delete_embedding\": true\n\
                }\n\
                ```\n\n\
                ### Advanced: Direct Query (Last Resort)\n\
                ```json\n\
                {\n\
                  \"endpoint\": \"get_business_products\",\n\
                  \"payload\": { \"business_id\": \"BIZ123\" }\n\
                }\n\
                ```\n\n\
                ## Filtering\n\
                Range queries:\n\
                ```json\n\
                { \"price\": { \"lte\": 100.0, \"gte\": 50.0 } }\n\
                ```\n\n\
                Exact matches:\n\
                ```json\n\
                { \"availability\": \"in_stock\" }\n\
                ```"
            },
            "meta://schema" => {
                "# AI Memory Layer Schema\n\n\
                ## Business Memory Types\n\n\
                ### 1. BusinessProductMemory\n\
                - product_id, product_name, product_category\n\
                - price, currency, availability\n\
                - description, features, specifications\n\
                - tags, seo_keywords\n\
                - Vector embedding for semantic search\n\n\
                ### 2. BusinessServiceMemory\n\
                - service_id, service_name, service_category\n\
                - price, currency, availability\n\
                - duration, description, benefits\n\
                - requirements, tags\n\
                - Vector embedding\n\n\
                ### 3. BusinessLocationMemory\n\
                - location_id, location_name, location_type\n\
                - address, city, state, country, postal_code\n\
                - latitude, longitude\n\
                - phone, email, website\n\
                - Vector embedding\n\n\
                ### 4. BusinessHoursMemory\n\
                - hours_id, location_id\n\
                - monday_open, monday_close\n\
                - (tuesday through sunday)\n\
                - special_hours, holidays\n\
                - Vector embedding\n\n\
                ### 5. BusinessSocialMediaMemory\n\
                - social_id, platform, handle\n\
                - profile_url, follower_count, post_count\n\
                - engagement_rate, description\n\
                - Vector embedding\n\n\
                ### 6. BusinessPolicyMemory\n\
                - policy_id, policy_type, policy_name\n\
                - content, version, effective_date\n\
                - is_active\n\
                - Vector embedding\n\n\
                ### 7. BusinessEventMemory\n\
                - event_id, event_name, event_type\n\
                - description, start_date, end_date\n\
                - location, capacity, registration_url\n\
                - Vector embedding\n\n\
                ## Customer Memory Types\n\n\
                ### 1. CustomerBehaviorMemory\n\
                - behavior_id, customer_id\n\
                - behavior_type, action, context\n\
                - timestamp, channel, duration_seconds\n\
                - Vector embedding\n\n\
                ### 2. CustomerPreferenceMemory\n\
                - preference_id, customer_id\n\
                - preference_type, subject, strength\n\
                - evidence_count, text_description\n\
                - Vector embedding\n\n\
                ### 3. CustomerDesireMemory\n\
                - desire_id, customer_id\n\
                - desire_type, goal, priority\n\
                - timeline, budget_range\n\
                - Vector embedding\n\n\
                ### 4. CustomerRuleMemory\n\
                - rule_id, customer_id\n\
                - rule_type, condition, action\n\
                - priority, enforcement, is_active\n\
                - Vector embedding\n\n\
                ### 5. CustomerFeedbackMemory\n\
                - feedback_id, customer_id\n\
                - subject, rating, sentiment\n\
                - text_description, tags\n\
                - Vector embedding\n\n\
                ## Relationship Edges\n\
                - CustomerLikedProduct (with text_reason)\n\
                - CustomerDislikedProduct (with text_reason)\n\
                - CustomerUsedService (with text_feedback)\n\
                - CustomerVisitedLocation (with text_notes)\n\n\
                ## Embedding Format\n\
                - Model: text-embedding-3-small (OpenAI) or equivalent\n\
                - Dimensions: 1536 (configurable)\n\
                - Stored as F32 vector arrays\n\
                - Used for hybrid search (vector + keyword)"
            },
            _ => {
                return Err(McpError::resource_not_found(
                    "Resource not found",
                    Some(json!({"uri": uri}))
                ));
            }
        };

        Ok(rmcp::model::ReadResourceResult {
            contents: vec![ResourceContents::text(content, uri.clone())],
        })
    }
}

fn main() -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}

/// Test local embedding server connection
async fn test_local_embedding_connection(url: &str) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    
    // Try to send a simple test request
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&json!({
            "text": "connection test"
        }))
        .send()
        .await?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Server returned status: {}", response.status()))
    }
}

async fn test_tcp_embedding_connection(addr: &str, timeout_secs: u64) -> Result<()> {
    let timeout = timeout_secs.min(10); // Cap at 10 seconds for connection test
    
    // Try to connect to TCP server and send a test request
    let client = embedding_client::EmbeddingClient::new(addr.to_string(), timeout);
    
    // Try a simple test embedding
    let _result = client.embed_text("connection test").await
        .map_err(|e| anyhow::anyhow!("TCP test request failed: {}", e))?;
    
    Ok(())
}

async fn async_main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .with_writer(std::io::stderr)
        .init();

    info!("");
    info!("   HelixDB MCP Server (Rust)");
    info!("");

    // Load configuration
    let config = Config::load().unwrap_or_else(|e| {
        error!("Failed to load config: {}, using defaults", e);
        Config::default()
    });

    info!(" Configuration loaded:");
    info!("   Embedding Mode: {:?}", config.embedding.mode);
    
    if config.is_mcp_embedding_enabled() {
        info!("   Provider: {:?}", config.embedding.provider);
        info!("   Model: {:?}", config.embedding.model);
        
        // Validate provider configuration
        match config.embedding.provider {
            Some(config::EmbeddingProvider::OpenAI) | Some(config::EmbeddingProvider::Gemini) => {
                // Check API key for cloud providers
                let api_key = config.get_api_key();
                if api_key.is_some() {
                    info!("   API Key: ? Configured");
                    let api_url = config.embedding.openai_api_url.as_ref()
                        .or(config.embedding.gemini_api_url.as_ref());
                    if let Some(url) = api_url {
                        info!("   API URL: {}", url);
                    }
                } else {
                    error!("   API Key: ? MISSING!");
                    error!("   Please set OPENAI_API_KEY or GEMINI_API_KEY environment variable");
                    error!("   Or add 'api_key' to mcpconfig.toml [embedding] section");
                    anyhow::bail!("API key required for cloud embedding providers");
                }
            }
            Some(config::EmbeddingProvider::Local) => {
                // Check local API connection
                let local_url = config.embedding.local_api_url.as_ref();
                if let Some(url) = local_url {
                    info!("   Local API URL: {}", url);
                    info!("   Testing local embedding server connection...");
                    
                    // Test connection to local server
                    match test_local_embedding_connection(url).await {
                        Ok(()) => {
                            info!("   ? Local embedding server is reachable");
                        }
                        Err(e) => {
                            error!("   ? Cannot connect to local embedding server: {}", e);
                            error!("   Make sure your local embedding server is running at {}", url);
                            error!("   Example: python local_server.py or ollama serve");
                            anyhow::bail!("Local embedding server not reachable");
                        }
                    }
                } else {
                    error!("   Local API URL: ? NOT CONFIGURED");
                    error!("   Please set 'local_api_url' in mcpconfig.toml [embedding] section");
                    anyhow::bail!("Local API URL required for local embedding provider");
                }
            }
            Some(config::EmbeddingProvider::Tcp) => {
                // Check TCP connection
                let tcp_addr = config.embedding.tcp_address.as_ref();
                if let Some(addr) = tcp_addr {
                    info!("   TCP Address: {}", addr);
                    info!("   Testing TCP embedding server connection...");
                    
                    // Test connection to TCP server
                    match test_tcp_embedding_connection(addr, config.embedding.tcp_timeout_secs).await {
                        Ok(()) => {
                            info!("   ? TCP embedding server is reachable");
                        }
                        Err(e) => {
                            error!("   ? Cannot connect to TCP embedding server: {}", e);
                            error!("   Make sure EmbeddingServer is running at {}", addr);
                            error!("   Example: cd EmbeddingServer && cargo run");
                            anyhow::bail!("TCP embedding server not reachable");
                        }
                    }
                } else {
                    error!("   TCP Address: ? NOT CONFIGURED");
                    error!("   Please set 'tcp_address' in mcpconfig.toml [embedding] section");
                    anyhow::bail!("TCP address required for TCP embedding provider");
                }
            }
            None => {
                error!("   Provider: ? NOT CONFIGURED");
                error!("   Please set 'provider' in mcpconfig.toml to: openai, gemini, or local");
                anyhow::bail!("Embedding provider not configured");
            }
        }
    } else {
        info!("   HelixDB will handle embedding generation via Embed() function");
        info!("   Configure embedding_model in helix.toml and set API key in environment");
    }

    // Use config values for HelixDB connection
    let endpoint = std::env::var("HELIX_ENDPOINT").unwrap_or_else(|_| config.helix.endpoint.clone());
    let port = std::env::var("HELIX_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(config.helix.port);

    info!(" Connecting to HelixDB at {}:{}", endpoint, port);
    let helix_client = Arc::new(HelixClient::new(&endpoint, port));
    
    match helix_client.test_connection().await {
        Ok(_) => info!(" Connected to HelixDB"),
        Err(e) => {
            error!(" Failed to connect: {}", e);
            anyhow::bail!("Connection failed: {}", e);
        }
    }

    let server = HelixMcpServer::new(helix_client, Arc::new(config));
    
    info!(" MCP Server ready on stdio");
    serve_server(server, stdio()).await?;
    
    Ok(())
}

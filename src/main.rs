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
use uuid::Uuid;

mod helix_client;
mod session;
mod config;
mod embedding_client;
mod server;

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
    memory_type: String,  // "behavior", "preference", "desire", "rule", "feedback", "communication"
    updates: serde_json::Value,  // JSON object with fields to update
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct UpdateInteractionParam {
    interaction_id: String,
    interaction_type: String,  // "product" or "service"
    composite_text: String,  // Updated text description for re-embedding
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct UpdateNavigationParam {
    memory_id: String,  // navigation_id, waypoint_id, or path_id
    navigation_type: String,  // "hub", "waypoint", or "path"
    composite_text: String,  // Updated text description for re-embedding
}

// Delete parameters
#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct DeleteMemoryParam {
    memory_id: String,
    memory_type: String,  // "product", "service", "behavior", "preference", "business", "customer", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    delete_embedding: Option<bool>,  // Whether to also delete embedding (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    delete_strategy: Option<String>,  // "node_only", "with_embedding", "cascade", "complete" (default: "with_embedding")
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
struct SearchBM25Param {
    query: String,  // Keyword search query
    memory_types: Vec<String>,  // e.g., ["products", "services", "preferences"]
    #[serde(skip_serializing_if = "Option::is_none")]
    business_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    customer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<i32>,  // Default: 10
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

// Customer Interaction parameters
#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct CreateCustomerProductInteractionParam {
    customer_id: String,
    product_id: String,
    interaction_type: String,  // "liked", "disliked", "purchased", "viewed", "favorited", "reviewed"
    #[serde(skip_serializing_if = "Option::is_none")]
    rating: Option<i32>,  // Rating if applicable (1-5 scale)
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<String>,  // "whatsapp", "website", "store", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    session_duration: Option<i32>,  // How long customer engaged (seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    purchase_amount: Option<f64>,  // Amount spent if purchased
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,  // Currency of purchase
    #[serde(skip_serializing_if = "Option::is_none")]
    issue_category: Option<String>,  // For dislikes: "quality", "price", "functionality", "service"
    #[serde(skip_serializing_if = "Option::is_none")]
    resolution_status: Option<String>,  // For issues: "resolved", "pending", "escalated"
    text_reason: String,  // Natural language reason for interaction (used for embedding)
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct CreateCustomerServiceInteractionParam {
    customer_id: String,
    service_id: String,
    interaction_type: String,  // "booked", "completed", "reviewed", "canceled"
    #[serde(skip_serializing_if = "Option::is_none")]
    satisfaction_rating: Option<i32>,  // Rating (1-5 scale)
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_actual: Option<i32>,  // Actual duration in minutes
    #[serde(skip_serializing_if = "Option::is_none")]
    cost_actual: Option<f64>,  // Actual cost paid
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,  // Currency code
    #[serde(skip_serializing_if = "Option::is_none")]
    outcome: Option<String>,  // Service outcome
    text_feedback: String,  // Natural language feedback (used for embedding)
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct QueryCustomerInteractionsParam {
    customer_id: String,
    interaction_type: String,  // "product", "service", "all"
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct SearchCustomerInteractionsParam {
    query: String,
    interaction_types: Vec<String>,  // e.g., ["product", "service"]
    #[serde(skip_serializing_if = "Option::is_none")]
    customer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<i32>,
}

// Navigation System parameters
#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct CreateNavigationHubParam {
    business_id: String,
    navigation_id: String,
    primary_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    secondary_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    building_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    building_type: Option<String>,
    latitude: f64,
    longitude: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    what3words_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plus_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compass_bearing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compass_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    magnetic_declination: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    building_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    building_floors: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    business_floor: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    building_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    building_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    main_entrance_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alternative_entrances: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entrance_restrictions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wheelchair_accessible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    elevator_available: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stairs_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    accessibility_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parking_available: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parking_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    public_transport_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    direction_varies_by_hours: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    after_hours_instructions: Option<String>,
    navigation_summary: String,  // Used for embedding
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct CreateNavigationWaypointParam {
    navigation_id: String,
    waypoint_name: String,
    waypoint_type: String,  // "landmark", "turn", "intersection", "building", "sign"
    #[serde(skip_serializing_if = "Option::is_none")]
    waypoint_category: Option<String>,
    description: String,  // Used for embedding
    #[serde(skip_serializing_if = "Option::is_none")]
    visual_cues: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audio_cues: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    relative_position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    distance_from_main: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    floor_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compass_direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compass_bearing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compass_distance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    business_specific_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    accessibility_info: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seasonal_availability: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_restrictions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    weather_dependent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority_level: Option<i32>,
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct CreateDirectionPathParam {
    navigation_id: String,
    path_name: String,
    path_type: String,  // "primary", "alternative", "accessible", "emergency"
    #[serde(skip_serializing_if = "Option::is_none")]
    transport_mode: Option<String>,  // "walking", "driving", "cycling", "public_transport"
    #[serde(skip_serializing_if = "Option::is_none")]
    estimated_duration_minutes: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    difficulty_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    distance_meters: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    starting_compass_bearing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ending_compass_bearing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path_compass_waypoints: Option<String>,  // JSON array of compass bearings
    #[serde(skip_serializing_if = "Option::is_none")]
    suitable_for_mobility_aids: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suitable_for_children: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suitable_in_rain: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suitable_at_night: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    requires_appointment: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    requires_security_clearance: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    visitor_badge_required: Option<bool>,
    step_by_step_instructions: String,  // Used for embedding
    #[serde(skip_serializing_if = "Option::is_none")]
    quick_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_recommended: Option<bool>,
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct QueryNavigationParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    business_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    navigation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_waypoints: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_paths: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter_accessible_only: Option<bool>,
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
struct SearchNavigationParam {
    query: String,
    search_types: Vec<String>,  // e.g., ["hubs", "waypoints", "paths"]
    #[serde(skip_serializing_if = "Option::is_none")]
    business_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<i32>,
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

    // Helper function to normalize memory_type to SINGULAR (for create/update/delete operations)
    fn normalize_memory_type(memory_type: &str) -> &str {
        match memory_type {
            // Plural to singular for create/update/delete operations
            "products" => "product",
            "services" => "service",
            "locations" => "location",
            "policies" => "policy",
            "events" => "event",
            // Plural to singular for customer operations
            "behaviors" => "behavior",
            "preferences" => "preference",
            "desires" => "desire",
            "rules" => "rule",
            // Already singular or special cases
            _ => memory_type
        }
    }

    // Helper function to normalize memory_type to PLURAL (for query operations)
    fn normalize_to_plural(memory_type: &str) -> &str {
        match memory_type {
            // Singular to plural for query operations
            "product" => "products",
            "service" => "services",
            "location" => "locations",
            "policy" => "policies",
            "event" => "events",
            // Singular to plural for customer operations
            "behavior" => "behaviors",
            "preference" => "preferences",
            "desire" => "desires",
            "rule" => "rules",
            // Already plural or special cases
            _ => memory_type
        }
    }

    // ========================================================================
    // HIGH-LEVEL DOMAIN-SPECIFIC TOOLS FOR AI MEMORY LAYER
    // ========================================================================

    #[tool(description = "Query business memories - unified access to products, services, locations, hours, social media, policies, and events for a specific business")]
    async fn query_business_memory(&self, params: Parameters<QueryBusinessMemoryParam>) -> Result<CallToolResult, McpError> {
        let business_id = &params.0.business_id;
        let memory_type_input = &params.0.memory_type;
        
        // Normalize to plural (accept both "product" and "products")
        let memory_type = Self::normalize_to_plural(memory_type_input);
        
        info!("query_business_memory: business_id={}, type={} (normalized from: {})", business_id, memory_type, memory_type_input);

        // Determine which query to execute based on memory_type
        let query_name = match memory_type {
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
        let memory_type_input = &params.0.memory_type;
        
        // Normalize to plural (accept both "behavior" and "behaviors")
        let memory_type = Self::normalize_to_plural(memory_type_input);
        
        info!("query_customer_memory: customer_id={}, type={} (normalized from: {})", customer_id, memory_type, memory_type_input);

        // Determine which query to execute based on memory_type
        let query_name = match memory_type {
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
        let memory_types_input = &params.0.memory_types;
        let limit = params.0.limit.unwrap_or(10);
        
        // Normalize all memory types to plural (accept both "product" and "products")
        let memory_types: Vec<&str> = memory_types_input
            .iter()
            .map(|t| Self::normalize_to_plural(t.as_str()))
            .collect();
        
        info!("search_semantic: query='{}', types={:?} (normalized from: {:?}), limit={}", query, memory_types, memory_types_input, limit);

        // Check embedding mode from config
        if self.config.is_helixdb_embedding_enabled() {
            // HelixDB mode: Just pass text query, HelixDB generates embedding via Embed()
            info!("Using HelixDB embedding mode (Embed() function in queries)");
            
            let mut all_results = Vec::new();
            
            // Search across requested memory types
            for memory_type in &memory_types {
                let query_name = match *memory_type {
                    // Business memory types
                    "products" => "search_business_products_semantic",
                    "services" => "search_business_services_semantic",
                    "locations" => "search_business_locations_semantic",
                    "hours" => "search_business_hours_semantic",
                    "social" => "search_business_social_semantic",
                    "policies" => "search_business_policies_semantic",
                    "events" => "search_business_events_semantic",
                    // Customer memory types
                    "behaviors" => "search_customer_behaviors_semantic",
                    "preferences" => "search_customer_preferences_semantic",
                    "desires" => "search_customer_desires_semantic",
                    "rules" => "search_customer_rules_semantic",
                    "feedback" => "search_customer_feedback_semantic",
                    // Customer interaction types
                    "product_interactions" => "search_customer_product_interactions_semantic",
                    "service_interactions" => "search_customer_service_interactions_semantic",
                    // Navigation types
                    "navigation_hubs" => "search_navigation_hubs_semantic",
                    "waypoints" => "search_navigation_waypoints_semantic",
                    "direction_paths" => "search_direction_paths_semantic",
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
            for memory_type in &memory_types {
                let query_name = match *memory_type {
                    // Business memory types
                    "products" => "search_business_products_hybrid",
                    "services" => "search_business_services_hybrid",
                    "locations" => "search_business_locations_hybrid",
                    "hours" => "search_business_hours_hybrid",
                    "social" => "search_business_social_hybrid",
                    "policies" => "search_business_policies_hybrid",
                    "events" => "search_business_events_hybrid",
                    // Customer memory types
                    "behaviors" => "search_customer_behaviors_hybrid",
                    "preferences" => "search_customer_preferences_hybrid",
                    "desires" => "search_customer_desires_hybrid",
                    "rules" => "search_customer_rules_hybrid",
                    "feedback" => "search_customer_feedback_hybrid",
                    // Customer interaction types
                    "product_interactions" => "search_customer_product_interactions_hybrid",
                    "service_interactions" => "search_customer_service_interactions_hybrid",
                    // Navigation types
                    "navigation_hubs" => "search_navigation_hubs_hybrid",
                    "waypoints" => "search_navigation_waypoints_hybrid",
                    "direction_paths" => "search_direction_paths_hybrid",
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
                    // Apply business_id filter to business memory types
                    match *memory_type {
                        "products" => {
                            payload["business_id"] = json!(business_id);
                            payload["min_price"] = json!(0.0);
                            payload["max_price"] = json!(1000000.0);
                        }
                        "services" | "locations" | "hours" | "social" | "policies" | "events" => {
                            payload["business_id"] = json!(business_id);
                        }
                        _ => {}
                    }
                }

                if let Some(customer_id) = &params.0.customer_id {
                    // Apply customer_id filter to customer memory types
                    match *memory_type {
                        "behaviors" | "preferences" | "desires" | "rules" | "feedback" 
                        | "product_interactions" | "service_interactions" => {
                            payload["customer_id"] = json!(customer_id);
                        }
                        _ => {}
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

    #[tool(description = "BM25 keyword search - fast text-based search across all memory types. Use for exact matches, IDs, phone numbers, or when embeddings unavailable. Always available as fallback.")]
    async fn search_bm25(&self, params: Parameters<SearchBM25Param>) -> Result<CallToolResult, McpError> {
        let query = &params.0.query;
        let memory_types_input = &params.0.memory_types;
        let limit = params.0.limit.unwrap_or(10);

        // Normalize all memory types to plural (accept both "product" and "products")
        let memory_types: Vec<&str> = memory_types_input
            .iter()
            .map(|t| Self::normalize_to_plural(t.as_str()))
            .collect();

        info!("search_bm25: query='{}', types={:?} (normalized from: {:?}), limit={}", query, memory_types, memory_types_input, limit);

        let mut all_results = Vec::new();

        // Route each memory type to its BM25 query
        for memory_type in &memory_types {
            let query_name = match *memory_type {
                "products" => "search_business_products_bm25",
                "services" => "search_business_services_bm25",
                "locations" => "search_business_locations_bm25",
                "hours" => "search_business_hours_bm25",
                "social" => "search_business_social_bm25",
                "policies" => "search_business_policies_bm25",
                "events" => "search_business_events_bm25",
                "behaviors" => "search_customer_behaviors_bm25",
                "preferences" => "search_customer_preferences_bm25",
                "desires" => "search_customer_desires_bm25",
                "rules" => "search_customer_rules_bm25",
                "feedback" => "search_customer_feedback_bm25",
                "communication" => "search_customer_communication_bm25",
                "product_interactions" => "search_customer_product_interactions_bm25",
                "service_interactions" => "search_customer_service_interactions_bm25",
                "navigation_hubs" => "search_navigation_hubs_bm25",
                "waypoints" => "search_waypoints_bm25",
                "direction_paths" => "search_direction_paths_bm25",
                _ => {
                    warn!("Unknown memory type for BM25 search: {}", memory_type);
                    continue;
                }
            };

            // Build payload with query_text and k (limit)
            let payload = json!({
                "query_text": query,
                "k": limit
            });

            // Add filters if provided
            if let Some(business_id) = &params.0.business_id {
                // Note: BM25 queries don't use filters, but we can log it for context
                info!("BM25 search context: business_id={}", business_id);
            }

            if let Some(customer_id) = &params.0.customer_id {
                info!("BM25 search context: customer_id={}", customer_id);
            }

            // Execute BM25 query
            match self.helix_client.query(query_name, payload).await {
                Ok(results) => {
                    if let Some(array) = results.as_array() {
                        all_results.extend(array.iter().cloned());
                    }
                }
                Err(e) => {
                    error!("BM25 search failed for {}: {}", memory_type, e);
                }
            }
        }

        Ok(CallToolResult::structured(json!({
            "query": query,
            "memory_types": memory_types,
            "search_type": "bm25_keyword",
            "total_results": all_results.len(),
            "limit": limit,
            "note": "BM25 uses keyword matching, not semantic embeddings. Best for exact terms, IDs, or specific phrases.",
            "results": all_results
        })))
    }

    #[tool(description = "Find customer insights - discover relationships between customers and products/services. Valid relationship_type values: 'liked' (products customer likes), 'disliked' (products customer dislikes), 'used_service' (services used), 'visited_location' (locations visited), 'all' (all relationships). Returns embedded reasons for each relationship.")]
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

    #[tool(description = "Create new business memory - add products, services, locations, hours, social media, policies, or events. REQUIRED: text_description, and depending on memory_type - product_name (products), service_name (services), location_name (locations), policy_name (policies), event_name (events), platform (social). All other fields will be auto-filled with schema defaults if not provided.")]
    async fn create_business_memory(&self, params: Parameters<CreateBusinessMemoryParam>) -> Result<CallToolResult, McpError> {
        let business_id = &params.0.business_id;
        let memory_type_input = &params.0.memory_type;
        let mut data = params.0.data.clone();
        
        // Normalize memory_type (accept both "products" and "product")
        let memory_type = Self::normalize_memory_type(memory_type_input);
        
        info!("create_business_memory: business_id={}, type={} (normalized from: {})", business_id, memory_type, memory_type_input);

        // Validate required fields based on memory_type
        match memory_type {
            "product" => {
                if data.get("product_name").is_none() {
                    return Ok(CallToolResult::structured_error(json!({
                        "error": "product_name is required for product memory type"
                    })));
                }
            },
            "service" => {
                if data.get("service_name").is_none() {
                    return Ok(CallToolResult::structured_error(json!({
                        "error": "service_name is required for service memory type"
                    })));
                }
            },
            "location" => {
                if data.get("location_name").is_none() {
                    return Ok(CallToolResult::structured_error(json!({
                        "error": "location_name is required for location memory type"
                    })));
                }
            },
            "policy" => {
                if data.get("policy_name").is_none() {
                    return Ok(CallToolResult::structured_error(json!({
                        "error": "policy_name is required for policy memory type"
                    })));
                }
            },
            "event" => {
                if data.get("event_name").is_none() {
                    return Ok(CallToolResult::structured_error(json!({
                        "error": "event_name is required for event memory type"
                    })));
                }
            },
            "social" => {
                if data.get("platform").is_none() {
                    return Ok(CallToolResult::structured_error(json!({
                        "error": "platform is required for social memory type"
                    })));
                }
            },
            "hours" => {
                // No specific name field required for hours
            },
            _ => {}
        }

        // Add business_id to data (required in schema but provided as parameter)
        data["business_id"] = json!(business_id);

        // Add timestamps (auto-filled by MCP server, not LLM - schema has DEFAULT NOW)
        let timestamp = chrono::Utc::now().timestamp();
        data["created_at"] = json!(timestamp);
        data["updated_at"] = json!(timestamp);

        // Auto-generate required IDs based on memory type
        let id_field_name = match memory_type {
            "product" => "product_id",
            "service" => "service_id",
            "location" => "location_id",
            "hours" => "hours_id",
            "social" => "social_id",
            "policy" => "policy_id",
            "event" => "event_id",
            _ => return Ok(CallToolResult::structured_error(json!({
                "error": format!("Invalid memory_type: {}. Valid types: product, service, location, hours, social, policy, event", memory_type)
            }))),
        };
        let generated_id = format!("{}_{}", memory_type.to_uppercase(), Uuid::new_v4().to_string());
        data[id_field_name] = json!(generated_id);

        // Auto-fill optional fields based on schema defaults (only if not provided)
        match memory_type {
            "product" => {
                // Optional string fields (DEFAULT "" in schema)
                if !data.get("product_category").is_some() { data["product_category"] = json!(""); }
                if !data.get("currency").is_some() { data["currency"] = json!(""); }
                if !data.get("availability").is_some() { data["availability"] = json!(""); }
                if !data.get("description").is_some() { data["description"] = json!(""); }
                if !data.get("competitor_analysis").is_some() { data["competitor_analysis"] = json!(""); }
                if !data.get("text_description").is_some() { data["text_description"] = json!(""); }
                
                // Optional numeric field (DEFAULT 0.0 in schema)
                if !data.get("price").is_some() { data["price"] = json!(0.0); }
                
                // Optional JSON string fields (DEFAULT "{}" in schema)
                if !data.get("specifications").is_some() { data["specifications"] = json!("{}"); }
                if !data.get("seasonal_trends").is_some() { data["seasonal_trends"] = json!("{}"); }
                
                // REQUIRED array fields - auto-fill with empty arrays if not provided
                // Schema says these are REQUIRED but can be empty arrays
                if !data.get("features").is_some() { data["features"] = json!([]); }
                if !data.get("tags").is_some() { data["tags"] = json!([]); }
                if !data.get("seo_keywords").is_some() { data["seo_keywords"] = json!([]); }
            },
            "service" => {
                // Optional string fields (DEFAULT "" in schema)
                if !data.get("service_category").is_some() { data["service_category"] = json!(""); }
                if !data.get("currency").is_some() { data["currency"] = json!(""); }
                if !data.get("availability").is_some() { data["availability"] = json!(""); }
                if !data.get("description").is_some() { data["description"] = json!(""); }
                if !data.get("text_description").is_some() { data["text_description"] = json!(""); }
                
                // Optional numeric fields (DEFAULT in schema)
                if !data.get("price").is_some() { data["price"] = json!(0.0); }
                if !data.get("duration_minutes").is_some() { data["duration_minutes"] = json!(60); }
                
                // REQUIRED array fields - auto-fill with empty arrays if not provided
                if !data.get("requirements").is_some() { data["requirements"] = json!([]); }
                if !data.get("deliverables").is_some() { data["deliverables"] = json!([]); }
                if !data.get("tags").is_some() { data["tags"] = json!([]); }
            },
            "location" => {
                // Optional string fields (DEFAULT "" in schema)
                if !data.get("location_name").is_some() { data["location_name"] = json!(""); }
                if !data.get("address").is_some() { data["address"] = json!(""); }
                if !data.get("city").is_some() { data["city"] = json!(""); }
                if !data.get("state").is_some() { data["state"] = json!(""); }
                if !data.get("country").is_some() { data["country"] = json!(""); }
                if !data.get("postal_code").is_some() { data["postal_code"] = json!(""); }
                if !data.get("location_type").is_some() { data["location_type"] = json!(""); }
                if !data.get("parking_info").is_some() { data["parking_info"] = json!(""); }
                if !data.get("text_description").is_some() { data["text_description"] = json!(""); }
                
                // Optional numeric fields (DEFAULT 0.0 in schema)
                if !data.get("latitude").is_some() { data["latitude"] = json!(0.0); }
                if !data.get("longitude").is_some() { data["longitude"] = json!(0.0); }
                
                // REQUIRED array field - auto-fill with empty array if not provided
                if !data.get("accessibility").is_some() { data["accessibility"] = json!([]); }
            },
            "hours" => {
                // Optional string fields (DEFAULT "" in schema) - ONLY business_id and hours_id are REQUIRED
                if !data.get("schedule_type").is_some() { data["schedule_type"] = json!(""); }
                if !data.get("monday_open").is_some() { data["monday_open"] = json!(""); }
                if !data.get("monday_close").is_some() { data["monday_close"] = json!(""); }
                if !data.get("tuesday_open").is_some() { data["tuesday_open"] = json!(""); }
                if !data.get("tuesday_close").is_some() { data["tuesday_close"] = json!(""); }
                if !data.get("wednesday_open").is_some() { data["wednesday_open"] = json!(""); }
                if !data.get("wednesday_close").is_some() { data["wednesday_close"] = json!(""); }
                if !data.get("thursday_open").is_some() { data["thursday_open"] = json!(""); }
                if !data.get("thursday_close").is_some() { data["thursday_close"] = json!(""); }
                if !data.get("friday_open").is_some() { data["friday_open"] = json!(""); }
                if !data.get("friday_close").is_some() { data["friday_close"] = json!(""); }
                if !data.get("saturday_open").is_some() { data["saturday_open"] = json!(""); }
                if !data.get("saturday_close").is_some() { data["saturday_close"] = json!(""); }
                if !data.get("sunday_open").is_some() { data["sunday_open"] = json!(""); }
                if !data.get("sunday_close").is_some() { data["sunday_close"] = json!(""); }
                if !data.get("timezone").is_some() { data["timezone"] = json!(""); }
                if !data.get("text_description").is_some() { data["text_description"] = json!(""); }
                
                // Optional JSON field (DEFAULT "{}" in schema)
                if !data.get("exceptions").is_some() { data["exceptions"] = json!("{}"); }
            },
            "social" => {
                // Optional string fields (DEFAULT "" in schema) - REQUIRED: business_id, social_id, platform
                if !data.get("handle").is_some() { data["handle"] = json!(""); }
                if !data.get("profile_url").is_some() { data["profile_url"] = json!(""); }
                if !data.get("description").is_some() { data["description"] = json!(""); }
                if !data.get("contact_info").is_some() { data["contact_info"] = json!(""); }
                if !data.get("text_description").is_some() { data["text_description"] = json!(""); }
                
                // Optional numeric fields (DEFAULT 0 in schema)
                if !data.get("follower_count").is_some() { data["follower_count"] = json!(0); }
                if !data.get("post_count").is_some() { data["post_count"] = json!(0); }
                
                // last_updated defaults to NOW but will be set by timestamp logic above
            },
            "policy" => {
                // Optional string fields (DEFAULT "" in schema) - REQUIRED: business_id, policy_id, policy_name
                if !data.get("policy_type").is_some() { data["policy_type"] = json!(""); }
                if !data.get("content").is_some() { data["content"] = json!(""); }
                if !data.get("version").is_some() { data["version"] = json!(""); }
                if !data.get("text_description").is_some() { data["text_description"] = json!(""); }
                
                // Optional boolean (DEFAULT false in schema)
                if !data.get("is_active").is_some() { data["is_active"] = json!(false); }
                
                // REQUIRED array field
                if !data.get("tags").is_some() { data["tags"] = json!([]); }
                
                // effective_date defaults to NOW but will be handled separately if needed
                if !data.get("effective_date").is_some() { 
                    data["effective_date"] = json!(chrono::Utc::now().timestamp()); 
                }
            },
            "event" => {
                // Optional string fields (DEFAULT "" in schema) - REQUIRED: business_id, event_id, event_name
                if !data.get("event_type").is_some() { data["event_type"] = json!(""); }
                if !data.get("description").is_some() { data["description"] = json!(""); }
                if !data.get("location").is_some() { data["location"] = json!(""); }
                if !data.get("text_description").is_some() { data["text_description"] = json!(""); }
                
                // Optional numeric field (DEFAULT 0 in schema)
                if !data.get("capacity").is_some() { data["capacity"] = json!(0); }
                
                // Optional boolean (DEFAULT false in schema)
                if !data.get("registration_required").is_some() { data["registration_required"] = json!(false); }
                
                // REQUIRED array field
                if !data.get("tags").is_some() { data["tags"] = json!([]); }
                
                // Date fields default to NOW
                if !data.get("start_date").is_some() { 
                    data["start_date"] = json!(chrono::Utc::now().timestamp()); 
                }
                if !data.get("end_date").is_some() { 
                    data["end_date"] = json!(chrono::Utc::now().timestamp()); 
                }
            },
            _ => {
                // Unknown type - just ensure text_description exists
                if !data.get("text_description").is_some() { data["text_description"] = json!(""); }
            }
        }

        // Check if embedding needs to be generated (MCP mode)
        if self.config.is_mcp_embedding_enabled() {
            // Get text_description from data (now guaranteed to exist, may be empty string)
            let text_description = data.get("text_description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            // Only generate embedding if text_description is not empty
            if text_description.is_empty() {
                return Ok(CallToolResult::structured_error(json!({
                    "error": "text_description is required for embedding generation in MCP mode",
                    "suggestion": "Provide text_description field with descriptive content"
                })));
            }

            info!("Generating embedding for text_description...");
            
            // Get API key (empty for local provider)
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            // Generate embedding
            match self.generate_embedding(text_description, &api_key).await {
                Ok(embedding) => {
                    info!("✓ Generated {} dimensional embedding", embedding.len());
                    
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
                    error!("✗ Failed to generate embedding: {}", e);
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
        let query_name = match memory_type {
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

    #[tool(description = "Create new customer memory - add behaviors, preferences, desires, rules, or feedback. REQUIRED: Always include text_description field in data - it's used for AI embedding generation and semantic search. All other optional fields will be auto-filled with schema defaults if not provided.")]
    async fn create_customer_memory(&self, params: Parameters<CreateCustomerMemoryParam>) -> Result<CallToolResult, McpError> {
        let customer_id = &params.0.customer_id;
        let memory_type_input = &params.0.memory_type;
        let mut data = params.0.data.clone();
        
        // Normalize memory_type (accept both "behaviors" and "behavior")
        let memory_type = Self::normalize_memory_type(memory_type_input);
        
        info!("create_customer_memory: customer_id={}, type={} (normalized from: {})", customer_id, memory_type, memory_type_input);

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

        // Add customer_id to data (required by all customer memory queries)
        data["customer_id"] = json!(customer_id);

        // Auto-fill timestamps (DEFAULT NOW in schema)
        let current_timestamp = chrono::Utc::now().timestamp();
        if !data.get("created_at").is_some() {
            data["created_at"] = json!(current_timestamp);
        }
        if !data.get("updated_at").is_some() {
            data["updated_at"] = json!(current_timestamp);
        }
        if !data.get("timestamp").is_some() {
            data["timestamp"] = json!(current_timestamp);
        }

        // Auto-generate required IDs based on memory type
        let id_field_name = match memory_type {
            "behavior" => "behavior_id",
            "preference" => "preference_id", 
            "desire" => "desire_id",
            "rule" => "rule_id",
            "feedback" => "feedback_id",
            _ => return Ok(CallToolResult::structured_error(json!({
                "error": format!("Invalid memory_type: {}. Valid types: behavior, preference, desire, rule, feedback", memory_type)
            }))),
        };
        let generated_id = format!("{}_{}", memory_type.to_uppercase(), Uuid::new_v4().to_string());
        data[id_field_name] = json!(generated_id);

        // Auto-fill optional fields based on memory type with schema defaults
        match memory_type {
            "behavior" => {
                if !data.get("behavior_type").is_some() { data["behavior_type"] = json!(""); }
                if !data.get("action").is_some() { data["action"] = json!(""); }
                if !data.get("context").is_some() { data["context"] = json!(""); }
                if !data.get("channel").is_some() { data["channel"] = json!(""); }
                if !data.get("duration_seconds").is_some() { data["duration_seconds"] = json!(0); }
                if !data.get("metadata").is_some() { data["metadata"] = json!("{}"); }
                if !data.get("text_description").is_some() { data["text_description"] = json!(""); }
            },
            "preference" => {
                if !data.get("preference_type").is_some() { data["preference_type"] = json!(""); }
                if !data.get("category").is_some() { data["category"] = json!(""); }
                if !data.get("subject").is_some() { data["subject"] = json!(""); }
                if !data.get("strength").is_some() { data["strength"] = json!(""); }
                if !data.get("is_active").is_some() { data["is_active"] = json!(false); }
                if !data.get("evidence_count").is_some() { data["evidence_count"] = json!(0); }
                if !data.get("last_evidence").is_some() { data["last_evidence"] = json!(current_timestamp); }
                if !data.get("confidence_score").is_some() { data["confidence_score"] = json!(0.0); }
                if !data.get("source_channels").is_some() { data["source_channels"] = json!([]); }
                if !data.get("text_description").is_some() { data["text_description"] = json!(""); }
            },
            "desire" => {
                if !data.get("desire_type").is_some() { data["desire_type"] = json!(""); }
                if !data.get("category").is_some() { data["category"] = json!(""); }
                if !data.get("description").is_some() { data["description"] = json!(""); }
                if !data.get("priority").is_some() { data["priority"] = json!(""); }
                if !data.get("timeframe").is_some() { data["timeframe"] = json!(""); }
                if !data.get("budget_range").is_some() { data["budget_range"] = json!(""); }
                if !data.get("is_active").is_some() { data["is_active"] = json!(false); }
                if !data.get("text_description").is_some() { data["text_description"] = json!(""); }
            },
            "rule" => {
                if !data.get("rule_type").is_some() { data["rule_type"] = json!(""); }
                if !data.get("category").is_some() { data["category"] = json!(""); }
                if !data.get("rule_category").is_some() { data["rule_category"] = json!(""); }
                if !data.get("rule_description").is_some() { data["rule_description"] = json!(""); }
                if !data.get("enforcement").is_some() { data["enforcement"] = json!(""); }
                if !data.get("exceptions").is_some() { data["exceptions"] = json!([]); }
                if !data.get("condition").is_some() { data["condition"] = json!(""); }
                if !data.get("action").is_some() { data["action"] = json!(""); }
                if !data.get("priority").is_some() { data["priority"] = json!(0); }
                if !data.get("is_active").is_some() { data["is_active"] = json!(true); }
                if !data.get("text_description").is_some() { data["text_description"] = json!(""); }
            },
            "feedback" => {
                if !data.get("feedback_type").is_some() { data["feedback_type"] = json!(""); }
                if !data.get("subject").is_some() { data["subject"] = json!(""); }
                if !data.get("rating").is_some() { data["rating"] = json!(0); }
                if !data.get("sentiment").is_some() { data["sentiment"] = json!("neutral"); }
                if !data.get("channel").is_some() { data["channel"] = json!(""); }
                if !data.get("response_required").is_some() { data["response_required"] = json!(false); }
                if !data.get("resolved").is_some() { data["resolved"] = json!(false); }
                if !data.get("feedback_category").is_some() { data["feedback_category"] = json!(""); }
                if !data.get("source").is_some() { data["source"] = json!(""); }
                if !data.get("is_resolved").is_some() { data["is_resolved"] = json!(false); }
                if !data.get("text_description").is_some() { data["text_description"] = json!(""); }
            },
            _ => {}
        }

        // Determine which query to execute based on memory_type
        let query_name = match memory_type {
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
    // CUSTOMER INTERACTION TOOLS - Track detailed customer interactions
    // ========================================================================

    #[tool(description = "Create customer product interaction - track detailed customer-product interactions with reasons (likes, dislikes, purchases, views, reviews). Use query_business_memory to get product_id.")]
    async fn create_customer_product_interaction(&self, params: Parameters<CreateCustomerProductInteractionParam>) -> Result<CallToolResult, McpError> {
        let customer_id = &params.0.customer_id;
        let product_id = &params.0.product_id;
        let interaction_type = &params.0.interaction_type;
        let text_reason = &params.0.text_reason;
        
        // Always auto-generate interaction_id
        let interaction_id = format!("INT_{}", Uuid::new_v4().to_string());
        
        info!("create_customer_product_interaction: customer_id={}, product_id={}, interaction_id={}, type={}", customer_id, product_id, interaction_id, interaction_type);

        // Build the data payload with all fields
        let timestamp = chrono::Utc::now().timestamp();
        let mut data = json!({
            "customer_id": customer_id,
            "product_id": product_id,
            "interaction_id": interaction_id,
            "interaction_type": interaction_type,
            "rating": params.0.rating.unwrap_or(0),
            "timestamp": timestamp,
            "channel": params.0.channel.as_ref().unwrap_or(&String::from("")),
            "session_duration": params.0.session_duration.unwrap_or(0),
            "purchase_amount": params.0.purchase_amount.unwrap_or(0.0),
            "currency": params.0.currency.as_ref().unwrap_or(&String::from("USD")),
            "issue_category": params.0.issue_category.as_ref().unwrap_or(&String::from("")),
            "resolution_status": params.0.resolution_status.as_ref().unwrap_or(&String::from("none")),
            "created_at": timestamp,
            "updated_at": timestamp,
            "text_reason": text_reason
        });

        // Check if embedding needs to be generated (MCP mode)
        if self.config.is_mcp_embedding_enabled() {
            info!("Generating embedding for text_reason...");
            
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            match self.generate_embedding(text_reason, &api_key).await {
                Ok(embedding) => {
                    info!("✓ Generated {} dimensional embedding", embedding.len());
                    
                    data["embedding"] = json!(embedding);
                    
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
                    error!("✗ Failed to generate embedding: {}", e);
                    return Ok(CallToolResult::structured_error(json!({
                        "error": format!("Failed to generate embedding: {}", e),
                        "suggestion": "Check embedding configuration and API connectivity"
                    })));
                }
            }
        } else {
            info!("Using HelixDB embedding mode - HelixDB will generate embedding");
        }

        // Execute the query
        match self.helix_client.query("add_customer_product_interaction", data).await {
            Ok(result) => {
                Ok(CallToolResult::structured(json!({
                    "success": true,
                    "interaction_type": "product",
                    "customer_id": customer_id,
                    "product_id": product_id,
                    "embedding_mode": if self.config.is_mcp_embedding_enabled() { "mcp" } else { "helixdb" },
                    "result": result
                })))
            }
            Err(e) => {
                error!("create_customer_product_interaction failed: {}", e);
                Ok(CallToolResult::structured_error(json!({
                    "error": format!("Failed to create product interaction: {}", e)
                })))
            }
        }
    }

    #[tool(description = "Create customer service interaction - track detailed customer-service interactions with feedback (bookings, completions, reviews, cancellations). Use query_business_memory to get service_id.")]
    async fn create_customer_service_interaction(&self, params: Parameters<CreateCustomerServiceInteractionParam>) -> Result<CallToolResult, McpError> {
        let customer_id = &params.0.customer_id;
        let service_id = &params.0.service_id;
        let interaction_type = &params.0.interaction_type;
        let text_feedback = &params.0.text_feedback;
        
        // Always auto-generate interaction_id
        let interaction_id = format!("INT_{}", Uuid::new_v4().to_string());
        
        info!("create_customer_service_interaction: customer_id={}, service_id={}, interaction_id={}, type={}", customer_id, service_id, interaction_id, interaction_type);

        // Build the data payload with all fields
        let timestamp = chrono::Utc::now().timestamp();
        let mut data = json!({
            "customer_id": customer_id,
            "service_id": service_id,
            "interaction_id": interaction_id,
            "interaction_type": interaction_type,
            "satisfaction_rating": params.0.satisfaction_rating.unwrap_or(3),
            "timestamp": timestamp,
            "duration_actual": params.0.duration_actual.unwrap_or(0),
            "cost_actual": params.0.cost_actual.unwrap_or(0.0),
            "currency": params.0.currency.as_ref().unwrap_or(&String::from("USD")),
            "outcome": params.0.outcome.as_ref().unwrap_or(&String::from("")),
            "created_at": timestamp,
            "updated_at": timestamp,
            "text_feedback": text_feedback
        });

        // Check if embedding needs to be generated (MCP mode)
        if self.config.is_mcp_embedding_enabled() {
            info!("Generating embedding for text_feedback...");
            
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            match self.generate_embedding(text_feedback, &api_key).await {
                Ok(embedding) => {
                    info!("✓ Generated {} dimensional embedding", embedding.len());
                    
                    data["embedding"] = json!(embedding);
                    
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
                    error!("✗ Failed to generate embedding: {}", e);
                    return Ok(CallToolResult::structured_error(json!({
                        "error": format!("Failed to generate embedding: {}", e),
                        "suggestion": "Check embedding configuration and API connectivity"
                    })));
                }
            }
        } else {
            info!("Using HelixDB embedding mode - HelixDB will generate embedding");
        }

        // Execute the query
        match self.helix_client.query("add_customer_service_interaction", data).await {
            Ok(result) => {
                Ok(CallToolResult::structured(json!({
                    "success": true,
                    "interaction_type": "service",
                    "customer_id": customer_id,
                    "service_id": service_id,
                    "embedding_mode": if self.config.is_mcp_embedding_enabled() { "mcp" } else { "helixdb" },
                    "result": result
                })))
            }
            Err(e) => {
                error!("create_customer_service_interaction failed: {}", e);
                Ok(CallToolResult::structured_error(json!({
                    "error": format!("Failed to create service interaction: {}", e)
                })))
            }
        }
    }

    #[tool(description = "Query customer interactions - get all product and/or service interactions for a customer")]
    async fn query_customer_interactions(&self, params: Parameters<QueryCustomerInteractionsParam>) -> Result<CallToolResult, McpError> {
        let customer_id = &params.0.customer_id;
        let interaction_type_input = &params.0.interaction_type;
        
        // Normalize to singular (accept both "products" and "product")
        let interaction_type = Self::normalize_memory_type(interaction_type_input);
        
        info!("query_customer_interactions: customer_id={}, type={} (normalized from: {})", customer_id, interaction_type, interaction_type_input);

        let mut all_interactions = json!({});

        match interaction_type {
            "product" => {
                // Get product interactions only
                match self.helix_client.query(
                    "get_customer_product_interactions",
                    json!({"customer_id": customer_id})
                ).await {
                    Ok(interactions) => {
                        all_interactions["product_interactions"] = interactions;
                    }
                    Err(e) => {
                        error!("Failed to get product interactions: {}", e);
                        return Ok(CallToolResult::structured_error(json!({
                            "error": format!("Failed to get product interactions: {}", e)
                        })));
                    }
                }
            }
            "service" => {
                // Get service interactions only
                match self.helix_client.query(
                    "get_customer_service_interactions",
                    json!({"customer_id": customer_id})
                ).await {
                    Ok(interactions) => {
                        all_interactions["service_interactions"] = interactions;
                    }
                    Err(e) => {
                        error!("Failed to get service interactions: {}", e);
                        return Ok(CallToolResult::structured_error(json!({
                            "error": format!("Failed to get service interactions: {}", e)
                        })));
                    }
                }
            }
            "all" => {
                // Get both product and service interactions
                if let Ok(product_interactions) = self.helix_client.query(
                    "get_customer_product_interactions",
                    json!({"customer_id": customer_id})
                ).await {
                    all_interactions["product_interactions"] = product_interactions;
                }
                
                if let Ok(service_interactions) = self.helix_client.query(
                    "get_customer_service_interactions",
                    json!({"customer_id": customer_id})
                ).await {
                    all_interactions["service_interactions"] = service_interactions;
                }
            }
            _ => {
                return Ok(CallToolResult::structured_error(json!({
                    "error": format!("Invalid interaction_type: {}. Valid types: product, service, all", interaction_type)
                })));
            }
        }

        // Apply filters if provided
        if let Some(filters) = &params.0.filters {
            all_interactions = self.apply_filters(all_interactions, filters);
        }

        // Count results
        let product_count = all_interactions.get("product_interactions")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        let service_count = all_interactions.get("service_interactions")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        Ok(CallToolResult::structured(json!({
            "customer_id": customer_id,
            "interaction_type": interaction_type,
            "product_count": product_count,
            "service_count": service_count,
            "total_count": product_count + service_count,
            "data": all_interactions
        })))
    }

    #[tool(description = "Search customer interactions semantically - find product and service interactions by meaning using AI embeddings")]
    async fn search_customer_interactions(&self, params: Parameters<SearchCustomerInteractionsParam>) -> Result<CallToolResult, McpError> {
        let query = &params.0.query;
        let interaction_types_input = &params.0.interaction_types;
        let limit = params.0.limit.unwrap_or(10);
        
        // Normalize all interaction types to singular (accept both "products" and "product")
        let interaction_types: Vec<&str> = interaction_types_input
            .iter()
            .map(|t| Self::normalize_memory_type(t.as_str()))
            .collect();
        
        info!("search_customer_interactions: query='{}', types={:?} (normalized from: {:?}), limit={}", query, interaction_types, interaction_types_input, limit);

        // Check embedding mode
        if self.config.is_helixdb_embedding_enabled() {
            info!("Using HelixDB embedding mode (Embed() function in queries)");
            
            let mut all_results = Vec::new();
            
            // Search across requested interaction types
            for interaction_type in &interaction_types {
                let query_name = match *interaction_type {
                    "product" => "search_customer_product_interactions",
                    "service" => "search_customer_service_interactions",
                    _ => {
                        info!("Skipping unsupported interaction type: {}", interaction_type);
                        continue;
                    }
                };

                let mut payload = json!({
                    "query_embedding": query,
                    "limit": limit,
                });

                if let Some(customer_id) = &params.0.customer_id {
                    payload["customer_id"] = json!(customer_id);
                }

                match self.helix_client.query(query_name, payload).await {
                    Ok(results) => {
                        if let Some(array) = results.as_array() {
                            all_results.extend(array.iter().cloned());
                        }
                    }
                    Err(e) => {
                        error!("Interaction search failed for {}: {}", interaction_type, e);
                    }
                }
            }

            return Ok(CallToolResult::structured(json!({
                "query": query,
                "interaction_types": interaction_types,
                "total_results": all_results.len(),
                "limit": limit,
                "embedding_mode": "helixdb",
                "results": all_results
            })));
        }

        // MCP mode: Generate embedding
        if self.config.is_mcp_embedding_enabled() {
            info!("Using MCP embedding mode");
            
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            let query_embedding = match self.generate_embedding(query, &api_key).await {
                Ok(embedding) => {
                    info!("✓ Generated embedding vector with {} dimensions", embedding.len());
                    embedding
                }
                Err(e) => {
                    error!("✗ Failed to generate embedding: {}", e);
                    return Ok(CallToolResult::structured_error(json!({
                        "error": format!("Embedding generation failed: {}", e),
                        "suggestion": "Check API key, network connection, and server status"
                    })));
                }
            };

            let mut all_results = Vec::new();
            
            for interaction_type in &interaction_types {
                let query_name = match *interaction_type {
                    "product" => "search_customer_product_interactions",
                    "service" => "search_customer_service_interactions",
                    _ => {
                        info!("Skipping unsupported interaction type: {}", interaction_type);
                        continue;
                    }
                };

                let mut payload = json!({
                    "query_embedding": query_embedding,
                    "limit": limit,
                });

                if let Some(customer_id) = &params.0.customer_id {
                    payload["customer_id"] = json!(customer_id);
                }

                match self.helix_client.query(query_name, payload).await {
                    Ok(results) => {
                        if let Some(array) = results.as_array() {
                            all_results.extend(array.iter().cloned());
                        }
                    }
                    Err(e) => {
                        error!("Interaction search failed for {}: {}", interaction_type, e);
                    }
                }
            }

            return Ok(CallToolResult::structured(json!({
                "query": query,
                "interaction_types": interaction_types,
                "total_results": all_results.len(),
                "limit": limit,
                "embedding_mode": "mcp",
                "results": all_results
            })));
        }

        Ok(CallToolResult::structured_error(json!({
            "error": "Invalid embedding configuration. Check mcpconfig.toml"
        })))
    }

    // ========================================================================
    // NAVIGATION SYSTEM TOOLS - Physical location navigation with compass data
    // ========================================================================

    #[tool(description = "Create navigation hub - set up comprehensive navigation center for a business with address, coordinates, compass bearings, and accessibility information")]
    async fn create_navigation_hub(&self, params: Parameters<CreateNavigationHubParam>) -> Result<CallToolResult, McpError> {
        let business_id = &params.0.business_id;
        let navigation_id = &params.0.navigation_id;
        let navigation_summary = &params.0.navigation_summary;
        
        info!("create_navigation_hub: business_id={}, navigation_id={}", business_id, navigation_id);

        // Build the data payload with all fields
        let timestamp = chrono::Utc::now().timestamp();
        let mut data = json!({
            "business_id": business_id,
            "navigation_id": navigation_id,
            "primary_address": params.0.primary_address,
            "secondary_address": params.0.secondary_address.as_ref().unwrap_or(&String::from("")),
            "building_name": params.0.building_name.as_ref().unwrap_or(&String::from("")),
            "building_type": params.0.building_type.as_ref().unwrap_or(&String::from("")),
            "latitude": params.0.latitude,
            "longitude": params.0.longitude,
            "what3words_code": params.0.what3words_code.as_ref().unwrap_or(&String::from("")),
            "plus_code": params.0.plus_code.as_ref().unwrap_or(&String::from("")),
            "compass_bearing": params.0.compass_bearing.unwrap_or(0.0),
            "compass_reference": params.0.compass_reference.as_ref().unwrap_or(&String::from("")),
            "magnetic_declination": params.0.magnetic_declination.unwrap_or(0.0),
            "building_description": params.0.building_description.as_ref().unwrap_or(&String::from("")),
            "building_floors": params.0.building_floors.unwrap_or(1),
            "business_floor": params.0.business_floor.unwrap_or(1),
            "building_color": params.0.building_color.as_ref().unwrap_or(&String::from("")),
            "building_size": params.0.building_size.as_ref().unwrap_or(&String::from("")),
            "main_entrance_description": params.0.main_entrance_description.as_ref().unwrap_or(&String::from("")),
            "alternative_entrances": params.0.alternative_entrances.as_ref().unwrap_or(&String::from("")),
            "entrance_restrictions": params.0.entrance_restrictions.as_ref().unwrap_or(&String::from("")),
            "wheelchair_accessible": params.0.wheelchair_accessible.unwrap_or(false),
            "elevator_available": params.0.elevator_available.unwrap_or(false),
            "stairs_required": params.0.stairs_required.unwrap_or(false),
            "accessibility_notes": params.0.accessibility_notes.as_ref().unwrap_or(&String::from("")),
            "parking_available": params.0.parking_available.unwrap_or(false),
            "parking_description": params.0.parking_description.as_ref().unwrap_or(&String::from("")),
            "public_transport_notes": params.0.public_transport_notes.as_ref().unwrap_or(&String::from("")),
            "direction_varies_by_hours": params.0.direction_varies_by_hours.unwrap_or(false),
            "after_hours_instructions": params.0.after_hours_instructions.as_ref().unwrap_or(&String::from("")),
            "created_at": timestamp,
            "updated_at": timestamp,
            "last_verified_at": timestamp,
            "verification_source": "user",
            "navigation_summary": navigation_summary
        });

        // Check if embedding needs to be generated (MCP mode)
        if self.config.is_mcp_embedding_enabled() {
            info!("Generating embedding for navigation_summary...");
            
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            match self.generate_embedding(navigation_summary, &api_key).await {
                Ok(embedding) => {
                    info!("✓ Generated {} dimensional embedding", embedding.len());
                    
                    data["embedding"] = json!(embedding);
                    
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
                    error!("✗ Failed to generate embedding: {}", e);
                    return Ok(CallToolResult::structured_error(json!({
                        "error": format!("Failed to generate embedding: {}", e),
                        "suggestion": "Check embedding configuration and API connectivity"
                    })));
                }
            }
        } else {
            info!("Using HelixDB embedding mode - HelixDB will generate embedding");
        }

        // Execute the query
        match self.helix_client.query("add_business_navigation_hub", data).await {
            Ok(result) => {
                Ok(CallToolResult::structured(json!({
                    "success": true,
                    "business_id": business_id,
                    "navigation_id": navigation_id,
                    "embedding_mode": if self.config.is_mcp_embedding_enabled() { "mcp" } else { "helixdb" },
                    "result": result
                })))
            }
            Err(e) => {
                error!("create_navigation_hub failed: {}", e);
                Ok(CallToolResult::structured_error(json!({
                    "error": format!("Failed to create navigation hub: {}", e)
                })))
            }
        }
    }

    #[tool(description = "Create navigation waypoint - add landmarks, reference points, and visual cues with compass bearings for navigation. Use query_navigation to get navigation_id.")]
    async fn create_navigation_waypoint(&self, params: Parameters<CreateNavigationWaypointParam>) -> Result<CallToolResult, McpError> {
        let navigation_id = &params.0.navigation_id;
        let description = &params.0.description;
        
        // Always auto-generate waypoint_id
        let waypoint_id = format!("WPT_{}", Uuid::new_v4().to_string());
        
        info!("create_navigation_waypoint: waypoint_id={}, navigation_id={}", waypoint_id, navigation_id);

        // Build the data payload
        let timestamp = chrono::Utc::now().timestamp();
        let mut data = json!({
            "waypoint_id": waypoint_id,
            "navigation_id": navigation_id,
            "waypoint_name": params.0.waypoint_name,
            "waypoint_type": params.0.waypoint_type,
            "waypoint_category": params.0.waypoint_category.as_ref().unwrap_or(&String::from("")),
            "description": description,
            "visual_cues": params.0.visual_cues.as_ref().unwrap_or(&String::from("")),
            "audio_cues": params.0.audio_cues.as_ref().unwrap_or(&String::from("")),
            "relative_position": params.0.relative_position.as_ref().unwrap_or(&String::from("")),
            "distance_from_main": params.0.distance_from_main.unwrap_or(0),
            "floor_level": params.0.floor_level.unwrap_or(0),
            "compass_direction": params.0.compass_direction.as_ref().unwrap_or(&String::from("")),
            "compass_bearing": params.0.compass_bearing.unwrap_or(0.0),
            "compass_distance": params.0.compass_distance.unwrap_or(0.0),
            "business_specific_notes": params.0.business_specific_notes.as_ref().unwrap_or(&String::from("")),
            "accessibility_info": params.0.accessibility_info.as_ref().unwrap_or(&String::from("")),
            "seasonal_availability": params.0.seasonal_availability.as_ref().unwrap_or(&String::from("")),
            "time_restrictions": params.0.time_restrictions.as_ref().unwrap_or(&String::from("")),
            "weather_dependent": params.0.weather_dependent.unwrap_or(false),
            "created_at": timestamp,
            "is_active": true,
            "priority_level": params.0.priority_level.unwrap_or(1)
        });

        // Check if embedding needs to be generated (MCP mode)
        if self.config.is_mcp_embedding_enabled() {
            info!("Generating embedding for waypoint description...");
            
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            match self.generate_embedding(description, &api_key).await {
                Ok(embedding) => {
                    info!("✓ Generated {} dimensional embedding", embedding.len());
                    
                    data["embedding"] = json!(embedding);
                    
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
                    error!("✗ Failed to generate embedding: {}", e);
                    return Ok(CallToolResult::structured_error(json!({
                        "error": format!("Failed to generate embedding: {}", e),
                        "suggestion": "Check embedding configuration and API connectivity"
                    })));
                }
            }
        } else {
            info!("Using HelixDB embedding mode - HelixDB will generate embedding");
        }

        // Execute the query
        match self.helix_client.query("add_navigation_waypoint", data).await {
            Ok(result) => {
                Ok(CallToolResult::structured(json!({
                    "success": true,
                    "waypoint_id": waypoint_id,
                    "navigation_id": navigation_id,
                    "embedding_mode": if self.config.is_mcp_embedding_enabled() { "mcp" } else { "helixdb" },
                    "result": result
                })))
            }
            Err(e) => {
                error!("create_navigation_waypoint failed: {}", e);
                Ok(CallToolResult::structured_error(json!({
                    "error": format!("Failed to create navigation waypoint: {}", e)
                })))
            }
        }
    }

    #[tool(description = "Create direction path - add step-by-step directions with compass waypoints, suitability flags, and accessibility information. Use query_navigation to get navigation_id.")]
    async fn create_direction_path(&self, params: Parameters<CreateDirectionPathParam>) -> Result<CallToolResult, McpError> {
        let navigation_id = &params.0.navigation_id;
        let step_by_step_instructions = &params.0.step_by_step_instructions;
        
        // Always auto-generate path_id
        let path_id = format!("PTH_{}", Uuid::new_v4().to_string());
        
        info!("create_direction_path: path_id={}, navigation_id={}", path_id, navigation_id);

        // Build the data payload
        let timestamp = chrono::Utc::now().timestamp();
        let mut data = json!({
            "path_id": path_id,
            "navigation_id": navigation_id,
            "path_name": params.0.path_name,
            "path_type": params.0.path_type,
            "transport_mode": params.0.transport_mode.as_ref().unwrap_or(&String::from("walking")),
            "estimated_duration_minutes": params.0.estimated_duration_minutes.unwrap_or(10),
            "difficulty_level": params.0.difficulty_level.as_ref().unwrap_or(&String::from("easy")),
            "distance_meters": params.0.distance_meters.unwrap_or(0),
            "starting_compass_bearing": params.0.starting_compass_bearing.unwrap_or(0.0),
            "ending_compass_bearing": params.0.ending_compass_bearing.unwrap_or(0.0),
            "path_compass_waypoints": params.0.path_compass_waypoints.as_ref().unwrap_or(&String::from("[]")),
            "suitable_for_mobility_aids": params.0.suitable_for_mobility_aids.unwrap_or(false),
            "suitable_for_children": params.0.suitable_for_children.unwrap_or(true),
            "suitable_in_rain": params.0.suitable_in_rain.unwrap_or(true),
            "suitable_at_night": params.0.suitable_at_night.unwrap_or(true),
            "requires_appointment": params.0.requires_appointment.unwrap_or(false),
            "requires_security_clearance": params.0.requires_security_clearance.unwrap_or(false),
            "visitor_badge_required": params.0.visitor_badge_required.unwrap_or(false),
            "step_by_step_instructions": step_by_step_instructions,
            "quick_summary": params.0.quick_summary.as_ref().unwrap_or(&String::from("")),
            "created_at": timestamp,
            "is_recommended": params.0.is_recommended.unwrap_or(false),
            "is_active": true,
            "last_used_feedback": ""
        });

        // Check if embedding needs to be generated (MCP mode)
        if self.config.is_mcp_embedding_enabled() {
            info!("Generating embedding for step-by-step instructions...");
            
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            match self.generate_embedding(step_by_step_instructions, &api_key).await {
                Ok(embedding) => {
                    info!("✓ Generated {} dimensional embedding", embedding.len());
                    
                    data["embedding"] = json!(embedding);
                    
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
                    error!("✗ Failed to generate embedding: {}", e);
                    return Ok(CallToolResult::structured_error(json!({
                        "error": format!("Failed to generate embedding: {}", e),
                        "suggestion": "Check embedding configuration and API connectivity"
                    })));
                }
            }
        } else {
            info!("Using HelixDB embedding mode - HelixDB will generate embedding");
        }

        // Execute the query
        match self.helix_client.query("add_direction_path", data).await {
            Ok(result) => {
                Ok(CallToolResult::structured(json!({
                    "success": true,
                    "path_id": path_id,
                    "navigation_id": navigation_id,
                    "embedding_mode": if self.config.is_mcp_embedding_enabled() { "mcp" } else { "helixdb" },
                    "result": result
                })))
            }
            Err(e) => {
                error!("create_direction_path failed: {}", e);
                Ok(CallToolResult::structured_error(json!({
                    "error": format!("Failed to create direction path: {}", e)
                })))
            }
        }
    }

    #[tool(description = "Query navigation - get complete navigation data for a business including hub, waypoints, and paths with optional filtering. PREFERRED: Use business_id parameter to query navigation by business (recommended). ALTERNATIVE: Use navigation_id if you already know it. Returns hub details, waypoints, and direction paths.")]
    async fn query_navigation(&self, params: Parameters<QueryNavigationParam>) -> Result<CallToolResult, McpError> {
        info!("query_navigation");

        let mut navigation_data = json!({});

        // Determine navigation_id from business_id or use provided navigation_id
        let nav_id = if let Some(business_id) = &params.0.business_id {
            // Get navigation hub for business
            match self.helix_client.query(
                "get_business_navigation_hub",
                json!({"business_id": business_id})
            ).await {
                Ok(hub) => {
                    navigation_data["hub"] = hub.clone();
                    
                    // Extract navigation_id from hub - try multiple paths as response structure may vary
                    let nav_id = hub.get("navigation_id")
                        .and_then(|v| v.as_str())
                        .or_else(|| {
                            // Try nav_hub.navigation_id (nested structure)
                            hub.get("nav_hub")
                                .and_then(|h| h.get("navigation_id"))
                                .and_then(|v| v.as_str())
                        })
                        .or_else(|| {
                            // Try if hub is an array, get first item
                            hub.as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|item| item.get("navigation_id"))
                                .and_then(|v| v.as_str())
                        })
                        .or_else(|| {
                            // Try nav_hub as array
                            hub.get("nav_hub")
                                .and_then(|h| h.as_array())
                                .and_then(|arr| arr.first())
                                .and_then(|item| item.get("navigation_id"))
                                .and_then(|v| v.as_str())
                        })
                        .map(|s| s.to_string());
                    
                    if nav_id.is_none() {
                        error!("Could not extract navigation_id from hub response. Hub structure: {:?}", hub);
                    }
                    
                    nav_id
                }
                Err(e) => {
                    error!("Failed to get navigation hub: {}", e);
                    return Ok(CallToolResult::structured_error(json!({
                        "error": format!("Failed to get navigation hub for business_id {}: {}", business_id, e),
                        "suggestion": "Make sure a navigation hub exists for this business_id. Use create_navigation_hub first."
                    })));
                }
            }
        } else if let Some(navigation_id) = &params.0.navigation_id {
            navigation_data["hub"] = json!({"navigation_id": navigation_id});
            Some(navigation_id.clone())
        } else {
            return Ok(CallToolResult::structured_error(json!({
                "error": "Must provide either business_id or navigation_id",
                "suggestion": "Provide business_id to look up navigation by business, or navigation_id if you know it directly"
            })));
        };

        let nav_id = match nav_id {
            Some(id) => id,
            None => {
                return Ok(CallToolResult::structured_error(json!({
                    "error": "Could not determine navigation_id from the query response",
                    "suggestion": "The navigation hub may not exist for this business. Use create_navigation_hub first."
                })));
            }
        };

        // Get waypoints if requested
        if params.0.include_waypoints.unwrap_or(true) {
            match self.helix_client.query(
                "get_navigation_waypoints",
                json!({"navigation_id": nav_id})
            ).await {
                Ok(waypoints) => {
                    navigation_data["waypoints"] = waypoints;
                }
                Err(e) => {
                    error!("Failed to get waypoints: {}", e);
                }
            }
        }

        // Get paths if requested
        if params.0.include_paths.unwrap_or(true) {
            let query_name = if params.0.filter_accessible_only.unwrap_or(false) {
                "get_accessible_navigation"
            } else {
                "get_direction_paths"
            };

            match self.helix_client.query(
                query_name,
                json!({"navigation_id": nav_id})
            ).await {
                Ok(paths) => {
                    navigation_data["paths"] = paths;
                }
                Err(e) => {
                    error!("Failed to get paths: {}", e);
                }
            }
        }

        // Count results
        let waypoint_count = navigation_data.get("waypoints")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        let path_count = navigation_data.get("paths")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        Ok(CallToolResult::structured(json!({
            "navigation_id": nav_id,
            "waypoint_count": waypoint_count,
            "path_count": path_count,
            "data": navigation_data
        })))
    }

    #[tool(description = "Search navigation semantically - find navigation hubs, waypoints, and paths by meaning using AI embeddings")]
    async fn search_navigation(&self, params: Parameters<SearchNavigationParam>) -> Result<CallToolResult, McpError> {
        let query = &params.0.query;
        let search_types = &params.0.search_types;
        let limit = params.0.limit.unwrap_or(10);
        
        info!("search_navigation: query='{}', types={:?}, limit={}", query, search_types, limit);

        // Check embedding mode
        if self.config.is_helixdb_embedding_enabled() {
            info!("Using HelixDB embedding mode");
            
            let mut all_results = Vec::new();
            
            for search_type in search_types {
                let query_name = match search_type.as_str() {
                    "hubs" => "search_navigation_hubs",
                    "waypoints" => "search_navigation_waypoints",
                    "paths" => "search_direction_paths",
                    _ => {
                        info!("Skipping unsupported search type: {}", search_type);
                        continue;
                    }
                };

                let mut payload = json!({
                    "query_embedding": query,
                    "limit": limit,
                });

                if let Some(business_id) = &params.0.business_id {
                    payload["business_id"] = json!(business_id);
                }

                match self.helix_client.query(query_name, payload).await {
                    Ok(results) => {
                        if let Some(array) = results.as_array() {
                            all_results.extend(array.iter().cloned());
                        }
                    }
                    Err(e) => {
                        error!("Navigation search failed for {}: {}", search_type, e);
                    }
                }
            }

            return Ok(CallToolResult::structured(json!({
                "query": query,
                "search_types": search_types,
                "total_results": all_results.len(),
                "limit": limit,
                "embedding_mode": "helixdb",
                "results": all_results
            })));
        }

        // MCP mode: Generate embedding
        if self.config.is_mcp_embedding_enabled() {
            info!("Using MCP embedding mode");
            
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            let query_embedding = match self.generate_embedding(query, &api_key).await {
                Ok(embedding) => {
                    info!("✓ Generated embedding vector with {} dimensions", embedding.len());
                    embedding
                }
                Err(e) => {
                    error!("✗ Failed to generate embedding: {}", e);
                    return Ok(CallToolResult::structured_error(json!({
                        "error": format!("Embedding generation failed: {}", e),
                        "suggestion": "Check API key, network connection, and server status"
                    })));
                }
            };

            let mut all_results = Vec::new();
            
            for search_type in search_types {
                let query_name = match search_type.as_str() {
                    "hubs" => "search_navigation_hubs",
                    "waypoints" => "search_navigation_waypoints",
                    "paths" => "search_direction_paths",
                    _ => {
                        info!("Skipping unsupported search type: {}", search_type);
                        continue;
                    }
                };

                let mut payload = json!({
                    "query_embedding": query_embedding,
                    "limit": limit,
                });

                if let Some(business_id) = &params.0.business_id {
                    payload["business_id"] = json!(business_id);
                }

                match self.helix_client.query(query_name, payload).await {
                    Ok(results) => {
                        if let Some(array) = results.as_array() {
                            all_results.extend(array.iter().cloned());
                        }
                    }
                    Err(e) => {
                        error!("Navigation search failed for {}: {}", search_type, e);
                    }
                }
            }

            return Ok(CallToolResult::structured(json!({
                "query": query,
                "search_types": search_types,
                "total_results": all_results.len(),
                "limit": limit,
                "embedding_mode": "mcp",
                "results": all_results
            })));
        }

        Ok(CallToolResult::structured_error(json!({
            "error": "Invalid embedding configuration. Check mcpconfig.toml"
        })))
    }

    // ========================================================================
    // UPDATE TOOLS - Modify existing memories
    // ========================================================================

    #[tool(description = "Update existing business memory (products, services, locations, hours, social media, policies, events). REQUIRED: memory_id (internal UUID from database node), memory_type, updates dict with: business_id, entity-specific ID (e.g., product_id from query), text_description for embedding regeneration. Get internal ID using query_business_memory.")]
    async fn update_business_memory(&self, params: Parameters<UpdateBusinessMemoryParam>) -> Result<CallToolResult, McpError> {
        let memory_id = &params.0.memory_id;
        let memory_type_input = &params.0.memory_type;
        let updates = &params.0.updates;
        
        // Normalize memory_type (accept both "products" and "product")
        let memory_type = Self::normalize_memory_type(memory_type_input);
        
        info!("update_business_memory: memory_id={}, type={} (normalized from: {})", memory_id, memory_type, memory_type_input);

        // Note: If product doesn't exist, query_business_memory returns empty array, so no special error handling needed here
        // The update query will fail naturally if memory_id doesn't exist

        // Extract composite_text from updates (required for vector-aware update queries)
        let composite_text = updates.get("composite_text")
            .or(updates.get("text_description"))
            .or(updates.get("description"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request(
                "Missing required field: composite_text (or text_description/description)", 
                None
            ))?;

        // Extract business_id (required for all business memory types)
        let business_id = updates.get("business_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request(
                "Missing required field: business_id in updates", 
                None
            ))?;

        // Extract entity-specific ID based on memory type
        let (entity_id_field, query_name) = match memory_type {
            "product" => ("product_id", "update_business_product_memory"),
            "service" => ("service_id", "update_business_service_memory"),
            "location" => ("location_id", "update_business_location_memory"),
            "hours" => ("hours_id", "update_business_hours_memory"),
            "social" => ("social_id", "update_business_social_memory"),
            "policy" => ("policy_id", "update_business_policy_memory"),
            "event" => ("event_id", "update_business_event_memory"),
            _ => {
                return Ok(CallToolResult::structured_error(json!({
                    "error": format!("Invalid memory_type: {}. Valid: product, service, location, hours, social, policy, event", memory_type)
                })));
            }
        };

        let entity_id = updates.get(entity_id_field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                let msg = format!("Missing required field: {} in updates", entity_id_field);
                McpError::invalid_request(msg, None)
            })?;

        info!("Updating {} with business_id={}, {}={}", memory_type, business_id, entity_id_field, entity_id);

        // Generate embedding based on mode
        let new_embedding = if self.config.is_mcp_embedding_enabled() {
            // MCP Mode: Generate embedding via OpenAI/Gemini/Local/TCP
            info!("MCP mode: Generating new embedding for {} {}", memory_type, entity_id);
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            match self.generate_embedding(composite_text, &api_key).await {
                Ok(emb) => emb,
                Err(e) => {
                    return Ok(CallToolResult::structured_error(json!({
                        "error": format!("Failed to generate embedding: {}", e)
                    })));
                }
            }
        } else {
            // HelixDB Mode: Use empty vector (HelixDB will generate via Embed() function)
            info!("HelixDB mode: Using empty embedding placeholder for {} {}", memory_type, entity_id);
            vec![]
        };

        // Build payload for vector-aware update query
        let timestamp = chrono::Utc::now().timestamp();
        let payload = json!({
            "business_id": business_id,
            entity_id_field: entity_id,
            "composite_text": composite_text,
            "new_embedding": new_embedding,
            "timestamp": timestamp
        });

        // Execute vector-aware update query (DROP old vector + CREATE new one)
        match self.helix_client.query(query_name, payload).await {
            Ok(result) => {
                Ok(CallToolResult::structured(json!({
                    "success": true,
                    "memory_type": memory_type,
                    "business_id": business_id,
                    entity_id_field: entity_id,
                    "query_used": query_name,
                    "embedding_mode": if self.config.is_mcp_embedding_enabled() { "mcp" } else { "helixdb" },
                    "updated_at": timestamp,
                    "result": result
                })))
            }
            Err(e) => {
                error!("update_business_memory failed: {}", e);
                Ok(CallToolResult::structured_error(json!({
                    "error": format!("Failed to update {} memory: {}", memory_type, e),
                    "query_used": query_name
                })))
            }
        }
    }

    #[tool(description = "Update existing customer memory (behaviors, preferences, desires, rules, feedback, communication). REQUIRED: memory_id (internal UUID from database node), memory_type, updates dict with text_description for embedding regeneration. Get internal ID using query_customer_memory.")]
    async fn update_customer_memory(&self, params: Parameters<UpdateCustomerMemoryParam>) -> Result<CallToolResult, McpError> {
        let memory_id = &params.0.memory_id;
        let memory_type_input = &params.0.memory_type;
        let updates = &params.0.updates;
        
        // Normalize memory_type (accept both "behaviors" and "behavior")
        let memory_type = Self::normalize_memory_type(memory_type_input);
        
        info!("update_customer_memory: memory_id={}, type={} (normalized from: {})", memory_id, memory_type, memory_type_input);

        // Extract composite_text from updates (required for vector-aware update queries)
        let composite_text = updates.get("composite_text")
            .or(updates.get("text_description"))
            .or(updates.get("context"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request(
                "Missing required field: composite_text (or text_description/context)", 
                None
            ))?;

        // Route to appropriate vector-aware update query
        let query_name = match memory_type {
            "preference" => "update_customer_preference_memory",
            "behavior" => "update_customer_behavior_memory",
            "desire" => "update_customer_desire_memory",
            "rule" => "update_customer_rule_memory",
            "feedback" => "update_customer_feedback_memory",
            "communication" => "update_customer_communication_memory",
            _ => {
                return Ok(CallToolResult::structured_error(json!({
                    "error": format!("Invalid memory_type: {}. Valid: preference, behavior, desire, rule, feedback, communication", memory_type)
                })));
            }
        };

        // Generate embedding based on mode
        let new_embedding = if self.config.is_mcp_embedding_enabled() {
            // MCP Mode: Generate embedding via OpenAI/Gemini/Local/TCP
            info!("MCP mode: Generating new embedding for {} {}", memory_type, memory_id);
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            match self.generate_embedding(composite_text, &api_key).await {
                Ok(emb) => emb,
                Err(e) => {
                    return Ok(CallToolResult::structured_error(json!({
                        "error": format!("Failed to generate embedding: {}", e)
                    })));
                }
            }
        } else {
            // HelixDB Mode: Use empty vector (HelixDB will generate via Embed() function)
            info!("HelixDB mode: Using empty embedding placeholder for {} {}", memory_type, memory_id);
            vec![]
        };

        // Build payload for vector-aware update query
        let timestamp = chrono::Utc::now().timestamp();
        let payload = json!({
            "memory_id": memory_id,
            "composite_text": composite_text,
            "new_embedding": new_embedding,
            "timestamp": timestamp
        });

        // Execute vector-aware update query (DROP old vector + CREATE new one)
        match self.helix_client.query(query_name, payload).await {
            Ok(result) => {
                Ok(CallToolResult::structured(json!({
                    "success": true,
                    "memory_type": memory_type,
                    "memory_id": memory_id,
                    "query_used": query_name,
                    "embedding_mode": if self.config.is_mcp_embedding_enabled() { "mcp" } else { "helixdb" },
                    "updated_at": timestamp,
                    "result": result
                })))
            }
            Err(e) => {
                error!("update_customer_memory failed: {}", e);
                Ok(CallToolResult::structured_error(json!({
                    "error": format!("Failed to update {} memory: {}", memory_type, e),
                    "query_used": query_name
                })))
            }
        }
    }

    #[tool(description = "Update customer interaction (product or service). REQUIRED: interaction_id (internal UUID from database node), interaction_type, composite_text for embedding regeneration. Get internal ID using query_customer_interactions.")]
    async fn update_interaction(&self, params: Parameters<UpdateInteractionParam>) -> Result<CallToolResult, McpError> {
        let interaction_id = &params.0.interaction_id;
        let interaction_type_input = &params.0.interaction_type;
        let composite_text = &params.0.composite_text;
        
        // Normalize to singular (accept both "products" and "product")
        let interaction_type = Self::normalize_memory_type(interaction_type_input);
        
        info!("update_interaction: interaction_id={}, type={} (normalized from: {})", interaction_id, interaction_type, interaction_type_input);

        // Route to appropriate vector-aware update query
        let query_name = match interaction_type {
            "product" => "update_customer_product_interaction_memory",
            "service" => "update_customer_service_interaction_memory",
            _ => {
                return Ok(CallToolResult::structured_error(json!({
                    "error": format!("Invalid interaction_type: {}. Valid: product, service", interaction_type)
                })));
            }
        };

        // Generate embedding based on mode
        let new_embedding = if self.config.is_mcp_embedding_enabled() {
            info!("MCP mode: Generating new embedding for {} interaction {}", interaction_type, interaction_id);
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            match self.generate_embedding(composite_text, &api_key).await {
                Ok(emb) => emb,
                Err(e) => {
                    return Ok(CallToolResult::structured_error(json!({
                        "error": format!("Failed to generate embedding: {}", e)
                    })));
                }
            }
        } else {
            info!("HelixDB mode: Using empty embedding placeholder for {} interaction {}", interaction_type, interaction_id);
            vec![]
        };

        // Build payload for vector-aware update query
        let timestamp = chrono::Utc::now().timestamp();
        let payload = json!({
            "memory_id": interaction_id,
            "composite_text": composite_text,
            "new_embedding": new_embedding,
            "timestamp": timestamp
        });

        // Execute vector-aware update query
        match self.helix_client.query(query_name, payload).await {
            Ok(result) => {
                Ok(CallToolResult::structured(json!({
                    "success": true,
                    "interaction_type": interaction_type,
                    "interaction_id": interaction_id,
                    "query_used": query_name,
                    "embedding_mode": if self.config.is_mcp_embedding_enabled() { "mcp" } else { "helixdb" },
                    "updated_at": timestamp,
                    "result": result
                })))
            }
            Err(e) => {
                error!("update_interaction failed: {}", e);
                Ok(CallToolResult::structured_error(json!({
                    "error": format!("Failed to update {} interaction: {}", interaction_type, e),
                    "query_used": query_name
                })))
            }
        }
    }

    #[tool(description = "Update navigation memory (hub, waypoint, path). REQUIRED: memory_id (internal UUID from database node), navigation_type, composite_text for embedding regeneration. Get internal ID using query_navigation.")]
    async fn update_navigation(&self, params: Parameters<UpdateNavigationParam>) -> Result<CallToolResult, McpError> {
        let memory_id = &params.0.memory_id;
        let navigation_type = &params.0.navigation_type;
        let composite_text = &params.0.composite_text;
        
        info!("update_navigation: memory_id={}, type={}", memory_id, navigation_type);

        // Route to appropriate vector-aware update query
        let query_name = match navigation_type.as_str() {
            "hub" => "update_business_navigation_hub_memory",
            "waypoint" => "update_navigation_waypoint_memory",
            "path" => "update_direction_path_memory",
            _ => {
                return Ok(CallToolResult::structured_error(json!({
                    "error": format!("Invalid navigation_type: {}. Valid: hub, waypoint, path", navigation_type)
                })));
            }
        };

        // Generate embedding based on mode
        let new_embedding = if self.config.is_mcp_embedding_enabled() {
            info!("MCP mode: Generating new embedding for navigation {} {}", navigation_type, memory_id);
            let api_key = self.config.get_api_key().unwrap_or_default();
            
            match self.generate_embedding(composite_text, &api_key).await {
                Ok(emb) => emb,
                Err(e) => {
                    return Ok(CallToolResult::structured_error(json!({
                        "error": format!("Failed to generate embedding: {}", e)
                    })));
                }
            }
        } else {
            info!("HelixDB mode: Using empty embedding placeholder for navigation {} {}", navigation_type, memory_id);
            vec![]
        };

        // Build payload for vector-aware update query
        let timestamp = chrono::Utc::now().timestamp();
        let payload = json!({
            "memory_id": memory_id,
            "composite_text": composite_text,
            "new_embedding": new_embedding,
            "timestamp": timestamp
        });

        // Execute vector-aware update query
        match self.helix_client.query(query_name, payload).await {
            Ok(result) => {
                Ok(CallToolResult::structured(json!({
                    "success": true,
                    "navigation_type": navigation_type,
                    "memory_id": memory_id,
                    "query_used": query_name,
                    "embedding_mode": if self.config.is_mcp_embedding_enabled() { "mcp" } else { "helixdb" },
                    "updated_at": timestamp,
                    "result": result
                })))
            }
            Err(e) => {
                error!("update_navigation failed: {}", e);
                Ok(CallToolResult::structured_error(json!({
                    "error": format!("Failed to update navigation {}: {}", navigation_type, e),
                    "query_used": query_name
                })))
            }
        }
    }

    // ========================================================================
    // DELETE TOOLS - Remove memories
    // ========================================================================

    #[tool(description = "Delete memory (products, services, locations, hours, social, policy, event, behaviors, preferences, desires, rules, feedback, business, customer). REQUIRED: memory_id (internal UUID from database node), memory_type. Get internal ID using appropriate query tool (query_business_memory, query_customer_memory, etc.).")]
    async fn delete_memory(&self, params: Parameters<DeleteMemoryParam>) -> Result<CallToolResult, McpError> {
        let memory_id = &params.0.memory_id;
        let memory_type_input = &params.0.memory_type;
        let delete_strategy = params.0.delete_strategy.as_ref()
            .map(|s| s.as_str())
            .unwrap_or_else(|| {
                // Backward compatibility: if delete_embedding is specified, use it
                if params.0.delete_embedding.unwrap_or(true) {
                    "with_embedding"
                } else {
                    "node_only"
                }
            });
        
        // Normalize memory_type (accept both "products" and "product")
        let memory_type = Self::normalize_memory_type(memory_type_input);
        
        info!("delete_memory: memory_id={}, type={} (normalized from: {}), strategy={}", memory_id, memory_type, memory_type_input, delete_strategy);

        // Handle cascade and complete deletion strategies
        match delete_strategy {
            "cascade" => {
                // Delete all memories for a business or customer
                let query_name = match memory_type {
                    "business" => "delete_all_business_memories",
                    "customer" => "delete_all_customer_memories",
                    _ => {
                        return Ok(CallToolResult::structured_error(json!({
                            "error": "Cascade delete strategy only valid for memory_type: business, customer"
                        })));
                    }
                };

                let payload = json!({
                    format!("{}_id", memory_type): memory_id
                });

                match self.helix_client.query(query_name, payload).await {
                    Ok(result) => {
                        return Ok(CallToolResult::structured(json!({
                            "success": true,
                            "memory_type": memory_type,
                            "memory_id": memory_id,
                            "strategy": "cascade",
                            "message": format!("Deleted all memories for {}", memory_type),
                            "result": result
                        })));
                    }
                    Err(e) => {
                        error!("Cascade delete failed: {}", e);
                        return Ok(CallToolResult::structured_error(json!({
                            "error": format!("Failed to cascade delete: {}", e)
                        })));
                    }
                }
            }
            "complete" => {
                // Delete entity and all associated memories
                let query_name = match memory_type {
                    "business" => "delete_business_complete",
                    "customer" => "delete_customer_complete",
                    _ => {
                        return Ok(CallToolResult::structured_error(json!({
                            "error": "Complete delete strategy only valid for memory_type: business, customer"
                        })));
                    }
                };

                let payload = json!({
                    format!("{}_id", memory_type): memory_id
                });

                match self.helix_client.query(query_name, payload).await {
                    Ok(result) => {
                        return Ok(CallToolResult::structured(json!({
                            "success": true,
                            "memory_type": memory_type,
                            "memory_id": memory_id,
                            "strategy": "complete",
                            "message": format!("Completely deleted {} and all associated memories", memory_type),
                            "result": result
                        })));
                    }
                    Err(e) => {
                        error!("Complete delete failed: {}", e);
                        return Ok(CallToolResult::structured_error(json!({
                            "error": format!("Failed to complete delete: {}", e)
                        })));
                    }
                }
            }
            _ => {}  // Fall through to normal delete
        }

        // Normal delete (node_only or with_embedding)
        let with_embedding = delete_strategy == "with_embedding";
        
        // Determine which delete query to use
        let query_name = match memory_type {
            "product" => if with_embedding { "delete_product_with_embedding" } else { "delete_product" },
            "service" => if with_embedding { "delete_service_with_embedding" } else { "delete_service" },
            "location" => if with_embedding { "delete_location_with_embedding" } else { "delete_location" },
            "hours" => if with_embedding { "delete_hours_with_embedding" } else { "delete_hours" },
            "social" => if with_embedding { "delete_social_with_embedding" } else { "delete_social" },
            "policy" => if with_embedding { "delete_policy_with_embedding" } else { "delete_policy" },
            "event" => if with_embedding { "delete_event_with_embedding" } else { "delete_event" },
            "behavior" => if with_embedding { "delete_behavior_with_embedding" } else { "delete_behavior" },
            "preference" => if with_embedding { "delete_preference_with_embedding" } else { "delete_preference" },
            "desire" => if with_embedding { "delete_desire_with_embedding" } else { "delete_desire" },
            "rule" => if with_embedding { "delete_rule_with_embedding" } else { "delete_rule" },
            "feedback" => if with_embedding { "delete_feedback_with_embedding" } else { "delete_feedback" },
            _ => {
                return Ok(CallToolResult::structured_error(json!({
                    "error": format!("Invalid memory_type: {}. Valid types: product, service, location, hours, social, policy, event, behavior, preference, desire, rule, feedback, business, customer", memory_type)
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
                    "strategy": delete_strategy,
                    "deleted_embedding": with_embedding,
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
                "AI Memory Layer - Business & customer intelligence system.\n\n\
                SEARCH STRATEGY: Use search_bm25 for exact terms/IDs/numbers, search_semantic for concepts.\n\n\
                CORE TOOLS:\n\
                • query_business_memory / query_customer_memory - Filter by criteria\n\
                • search_semantic - Find keywords (exact matches, IDs, phone numbers)\n\
                • create_business_memory / create_customer_memory - Add new memories\n\
                • update_business_memory / update_customer_memory - Modify existing\n\
                • delete_memory - Remove memories\n\n\
                INTERACTIONS:\n\
                • create_customer_product_interaction - Track product engagement\n\
                • create_customer_service_interaction - Track service usage\n\
                • query_customer_interactions / search_customer_interactions - Find interactions\n\
                • update_interaction - Modify interactions\n\n\
                NAVIGATION:\n\
                • create_navigation_hub / create_navigation_waypoint / create_direction_path\n\
                • query_navigation / search_navigation - Get directions\n\
                • update_navigation - Modify navigation\n\n\
                INSIGHTS:\n\
                • find_customer_insights - Discover customer relationships\n\n\
                ADVANCED:\n\
                • do_query - Direct database queries (last resort)".to_string()
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
                Business & customer intelligence system with semantic search.\n\n\
                ## Tools (22 total)\n\
                - 5 Search: query_business_memory, query_customer_memory, search_semantic, search_bm25, find_customer_insights\n\
                - 8 Create: business, customer, product_interaction, service_interaction, navigation_hub, waypoint, direction_path\n\
                - 4 Update: business, customer, interaction, navigation\n\
                - 3 Query Specialized: customer_interactions, navigation\n\
                - 1 Delete: all types\n\
                - 1 Advanced: do_query\n\n\
                ## Memory Types\n\
                Business: Products, Services, Locations, Hours, Social, Policies, Events\n\
                Customer: Behaviors, Preferences, Desires, Rules, Feedback, Communication\n\
                Interactions: Product, Service (node-based with reasons)\n\
                Navigation: Hubs, Waypoints, Paths (with compass bearings)\n\n\
                ## Search\n\
                - search_bm25: Keywords (exact, fast)\n\
                - search_semantic: Meaning (concepts, similar)"
            },
            "meta://instructions" => {
                "# AI Memory Layer - Usage Instructions\n\n\
                ## Search Strategy\n\
                1. Exact match (ID/phone/keyword)? → search_bm25\n\
                2. Conceptual search? → search_semantic\n\
                3. Not sure? Try search_bm25 first\n\n\
                ## Core Operations\n\n\
                **Query**: query_business_memory, query_customer_memory\n\
                **Search**: search_bm25 (keywords), search_semantic (meaning)\n\
                **Create**: create_business_memory, create_customer_memory\n\
                **Update**: update_business_memory, update_customer_memory\n\
                **Delete**: delete_memory\n\n\
                ## Interactions\n\
                **Create**: create_customer_product_interaction, create_customer_service_interaction\n\
                **Query**: query_customer_interactions, search_customer_interactions\n\
                **Update**: update_interaction\n\n\
                ## Navigation\n\
                **Create**: create_navigation_hub, create_navigation_waypoint, create_direction_path\n\
                **Query**: query_navigation, search_navigation\n\
                **Update**: update_navigation\n\n\
                ## Examples\n\n\
                Search by keyword:\n\
                search_bm25(query: \"headphones\", memory_types: [\"products\"])\n\n\
                Search by concept:\n\
                search_semantic(query: \"gifts for music lovers\", memory_types: [\"products\"])\n\n\
                Track interaction:\n\
                create_customer_product_interaction(\n\
                  customer_id: \"C123\",\n\
                  product_id: \"P456\",\n\
                  interaction_type: \"purchased\",\n\
                  text_reason: \"Needed for daily commute\"\n\
                )\n\n\
                Update with auto-refresh:\n\
                update_business_memory(\n\
                  memory_id: \"P456\",\n\
                  memory_type: \"product\",\n\
                  updates: {composite_text: \"New description\"}\n\
                )"
            },
            "meta://schema" => {
                "# AI Memory Layer Schema\n\n\
                ## Business Memory (7 types)\n\
                1. Product: product_id, name, category, price, currency, availability, description, features\n\
                2. Service: service_id, name, category, price, duration, description, benefits\n\
                3. Location: location_id, name, type, address, lat/lng, phone, email, website\n\
                4. Hours: hours_id, location_id, daily hours (mon-sun), special_hours, holidays\n\
                5. Social: social_id, platform, handle, profile_url, follower/post counts, engagement\n\
                6. Policy: policy_id, type, name, content, version, effective_date, is_active\n\
                7. Event: event_id, name, type, description, start/end dates, location, capacity\n\n\
                ## Customer Memory (6 types)\n\
                1. Behavior: behavior_id, customer_id, type, action, context, timestamp, channel\n\
                2. Preference: preference_id, customer_id, type, subject, strength, evidence_count\n\
                3. Desire: desire_id, customer_id, type, goal, priority, timeline, budget_range\n\
                4. Rule: rule_id, customer_id, type, condition, action, priority, is_active\n\
                5. Feedback: feedback_id, customer_id, subject, rating, sentiment, text\n\
                6. Communication: communication_id, customer_id, channel, message, timestamp\n\n\
                ## Interactions (node-based, 2 types)\n\
                1. CustomerProductInteraction: interaction_id, customer_id, product_id, type (liked/purchased/viewed/reviewed), rating, channel, purchase_amount, text_reason\n\
                2. CustomerServiceInteraction: interaction_id, customer_id, service_id, type (booked/completed/reviewed/canceled), satisfaction_rating, duration_actual, cost_actual, text_feedback\n\n\
                ## Navigation (3 types)\n\
                1. NavigationHub: navigation_id, business_id, address, lat/lng, what3words, plus_code, compass_bearing, building details, accessibility\n\
                2. NavigationWaypoint: waypoint_id, navigation_id, name, type, description, visual/audio cues, compass_bearing, floor_level\n\
                3. DirectionPath: path_id, navigation_id, name, type, transport_mode, duration, distance, compass waypoints, accessibility flags\n\n\
                All types have vector embeddings for semantic search."
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
        .without_time()
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

    let server = HelixMcpServer::new(helix_client, Arc::new(config.clone()));
    
    // Check which transports are enabled
    let tcp_enabled = config.server.enable_tcp;
    let http_enabled = config.server.enable_http;
    let stdio_mode = config.server.transport.as_str() == "stdio";
    
    // Handle multiple transports concurrently
    if tcp_enabled && http_enabled {
        info!("🔧 MCP Server ready - starting both TCP and HTTP servers");
        
        let tcp_addr = format!("{}:{}", config.server.tcp_host, config.server.tcp_port);
        let http_addr = format!("{}:{}", config.server.http_host, config.server.http_port);
        
        info!("   TCP:  {}", tcp_addr);
        info!("   HTTP: {}", http_addr);
        
        let server_tcp = server.clone();
        let server_http = server.clone();
        let config_tcp = Arc::new(config.server.clone());
        let config_http = Arc::new(config.server.clone());
        
        // Spawn both servers concurrently
        let tcp_handle = tokio::spawn(async move {
            if let Err(e) = server::start_tcp_server(server_tcp, &tcp_addr, config_tcp).await {
                error!("TCP server error: {}", e);
            }
        });
        
        let http_handle = tokio::spawn(async move {
            if let Err(e) = server::start_http_server(server_http, &http_addr, config_http).await {
                error!("HTTP server error: {}", e);
            }
        });
        
        // Wait for both servers (they run forever unless error)
        tokio::try_join!(tcp_handle, http_handle)?;
        
    } else if tcp_enabled {
        let tcp_addr = format!("{}:{}", config.server.tcp_host, config.server.tcp_port);
        info!("🔧 MCP Server ready - starting TCP server on {}", tcp_addr);
        server::start_tcp_server(server, &tcp_addr, Arc::new(config.server)).await?;
        
    } else if http_enabled {
        let http_addr = format!("{}:{}", config.server.http_host, config.server.http_port);
        info!("🔧 MCP Server ready - starting HTTP server on {}", http_addr);
        server::start_http_server(server, &http_addr, Arc::new(config.server)).await?;
        
    } else if stdio_mode {
        info!("🔧 MCP Server ready - using stdio transport");
        serve_server(server, stdio()).await?;
        
    } else {
        error!("❌ No transport enabled!");
        error!("   Enable at least one transport in mcpconfig.toml:");
        error!("   - Set enable_tcp = true for TCP");
        error!("   - Set enable_http = true for HTTP");
        error!("   - Set transport = \"stdio\" for STDIO");
        anyhow::bail!("No transport enabled");
    }
    
    Ok(())
}

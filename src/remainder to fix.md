ON CALL THIS TOOL: 
create_business_memory
 {
  "data": {
    "platform": "twitter",
    "handle": "@testbusiness",
    "text_description": "Social media profile with auto-generated ID"
  },
  "business_id": "test_business_1",
  "memory_type": "social"
}

I got this error :

{"error":"Failed to create social memory: HelixDB query failed with status 404 Not Found: Couldn't find `add_business_social_media_memory` of type Query"}

/////////////////////////////////////////////////////////

create_customer_memory

{
  "customer_id": "CUST001",
  "data": {
    "preference_category": "communication",
    "preference_value": "email",
    "text_description": "Customer prefers email communication"
  },
  "memory_type": "preference"
}

got this error :

{"error":"Failed to create preference memory: HelixDB query failed with status 500 Internal Server Error: Decode error: missing field `preference_type` at line 1 column 8449\n\n\t62385938}\n\t........^\n"}

/////////////////////////////////////////////////////////

**BM25 Search Issue:**
- MCP server implements `search_bm25` tool but HelixDB queries.hx file lacks corresponding BM25 search queries
- Semantic search works (uses HelixDB's built-in `Embed()` function), but BM25 fails due to missing queries like `search_business_products_bm25`, etc.
- Need to add BM25 keyword search queries to queries.hx for each memory type

/////////////////////////////////////////////////////////

**ID Auto-Generation Changes:**
- **COMPLETELY REMOVED** auto-generated parameters from schemas to prevent LLMs from trying to provide duplicate IDs
- Added UUID auto-generation for:
  - `interaction_id` in create_customer_product_interaction (prefix: INT_)
  - `interaction_id` in create_customer_service_interaction (prefix: INT_)
  - `navigation_id` in create_navigation_hub (prefix: NAV_)
  - `waypoint_id` in create_navigation_waypoint (prefix: WPT_)
  - `path_id` in create_direction_path (prefix: PTH_)
- Navigation child tools (waypoint, path) keep `navigation_id` as required since they reference existing parent hubs

**Schema Updates (Parameters Removed):**
- `interaction_id: Option<String>` REMOVED from CreateCustomerProductInteractionParam
- `interaction_id: Option<String>` REMOVED from CreateCustomerServiceInteractionParam
- `navigation_id: Option<String>` REMOVED from CreateNavigationHubParam
- `waypoint_id: Option<String>` REMOVED from CreateNavigationWaypointParam
- `path_id: Option<String>` REMOVED from CreateDirectionPathParam
- Parent dependency IDs kept as required String parameters: `business_id`, `customer_id`, `product_id`, `service_id`, `navigation_id` (for child tools)

**Create Tools Dependency Analysis:**

**❌ All Create Tools Actually Depend on Existing IDs:**

1. **`create_business_memory`** - Depends on existing `business_id` to create products/services/etc.
2. **`create_customer_memory`** - Depends on existing `customer_id` to create preferences/behaviors/etc.
3. **`create_customer_product_interaction`** - Depends on existing `product_id` (from business products)
4. **`create_customer_service_interaction`** - Depends on existing `service_id` (from business services)
5. **`create_navigation_hub`** - Depends on existing `business_id` to create navigation hub
6. **`create_navigation_waypoint`** - Depends on existing `navigation_id` (from navigation hub)
7. **`create_direction_path`** - Depends on existing `navigation_id` (from navigation hub)

**The correct workflow is:**
1. Create business (external process)
2. Create customer (external process) 
3. Create business products/services via `create_business_memory`
4. Create navigation hub via `create_navigation_hub`
5. Create customer interactions via `create_customer_product_interaction`/`create_customer_service_interaction`
6. Create navigation waypoints/paths via `create_navigation_waypoint`/`create_direction_path`

**Conclusion:** No create tools are truly independent - they all depend on existing parent entities (business_id, customer_id, product_id, service_id, navigation_id).

**✅ IMPLEMENTATION COMPLETE:**

**Auto-Generated Entity IDs (Working):**
- `create_business_memory`: Already auto-generates product_id, service_id, location_id, hours_id, social_id, policy_id, event_id
- `create_customer_memory`: Already auto-generates behavior_id, preference_id, desire_id, rule_id, feedback_id  
- `create_customer_product_interaction`: Now auto-generates interaction_id (INT_ prefix)
- `create_customer_service_interaction`: Now auto-generates interaction_id (INT_ prefix)
- `create_navigation_hub`: Now auto-generates navigation_id (NAV_ prefix)
- `create_navigation_waypoint`: Now auto-generates waypoint_id (WPT_ prefix) 
- `create_direction_path`: Now auto-generates path_id (PTH_ prefix)

**Required Parent IDs (Correctly Kept Required):**
- `business_id` (needed by: create_business_memory, create_navigation_hub)
- `customer_id` (needed by: create_customer_memory, create_customer_product_interaction, create_customer_service_interaction)
- `product_id` (needed by: create_customer_product_interaction)
- `service_id` (needed by: create_customer_service_interaction)
- `navigation_id` (needed by: create_navigation_waypoint, create_direction_path)

**Schema Updates (Parameters Removed):**
- ✅ `interaction_id: Option<String>` REMOVED from CreateCustomerProductInteractionParam
- ✅ `interaction_id: Option<String>` REMOVED from CreateCustomerServiceInteractionParam
- ✅ `navigation_id: Option<String>` REMOVED from CreateNavigationHubParam
- ✅ `waypoint_id: Option<String>` REMOVED from CreateNavigationWaypointParam
- ✅ `path_id: Option<String>` REMOVED from CreateDirectionPathParam
- Parent dependency IDs kept as required String parameters: `business_id`, `customer_id`, `product_id`, `service_id`, `navigation_id` (for child tools)

/////////////////////////////////////////////////////////

 we have to find tools that needs this parameter and advise LLM to use first the tools that expose this ids so that it can use them where needed:
✅ product_id (for product interactions)
✅ service_id (for service interactions)
✅ navigation_id (for waypoints & paths)

// HelixDB Queries for AI Memory Layer
// Contains working queries only

// ============================================================================
// BUSINESS MEMORY QUERIES  
// ============================================================================

// Add a new product memory node
QUERY add_business_product_memory(
    business_id: String,
    product_id: String,
    product_name: String,
    product_category: String,
    price: F64,
    currency: String,
    availability: String,
    description: String,
    features: [String],
    specifications: String,
    tags: [String],
    seo_keywords: [String],
    competitor_analysis: String,
    seasonal_trends: String,
    created_at: I64,
    updated_at: I64,
    text_description: String,
    embedding: [F32],
    embedding_model: String
) =>
    product <- AddN<BusinessProductMemory>({
        business_id: business_id,
        product_id: product_id,
        product_name: product_name,
        product_category: product_category,
        price: price,
        currency: currency,
        availability: availability,
        description: description,
        features: features,
        specifications: specifications,
        tags: tags,
        seo_keywords: seo_keywords,
        competitor_analysis: competitor_analysis,
        seasonal_trends: seasonal_trends,
        created_at: created_at,
        updated_at: updated_at,
        text_description: text_description
    })
    embedding_node <- AddV<BusinessProductEmbedding>(embedding, {
        text_description: text_description,
        embedding_model: embedding_model
    })
    edge <- AddE<HasProductEmbedding>({
        created_at: created_at
    })::From(product)::To(embedding_node)
    RETURN product

// Search products by vector similarity  
QUERY search_business_products(query_embedding: [F32], limit: I64) =>
    embeddings <- SearchV<BusinessProductEmbedding>(query_embedding, limit)
    products <- embeddings::In<HasProductEmbedding>
    RETURN products

// Hybrid search: vector + structured filters
QUERY search_business_products_hybrid(
    query_embedding: [F32],
    limit: I64,
    business_id: String,
    min_price: F64,
    max_price: F64
) =>
    embeddings <- SearchV<BusinessProductEmbedding>(query_embedding, limit)
    products <- embeddings::In<HasProductEmbedding>
    filtered <- products::WHERE(
        AND(
            _::{business_id}::EQ(business_id),
            _::{price}::GTE(min_price),
            _::{price}::LTE(max_price)
        )
    )
    RETURN filtered

// Get products by business ID (structured query)
QUERY get_business_products(business_id: String) =>
    products <- N<BusinessProductMemory>::WHERE(_::{business_id}::EQ(business_id))
    RETURN products

// ============================================================================
// CUSTOMER MEMORY QUERIES
// ============================================================================

// Add customer preference memory
QUERY add_customer_preference_memory(
    customer_id: String,
    preference_id: String,
    preference_type: String,
    category: String,
    subject: String,
    strength: String,
    is_active: Boolean,
    evidence_count: I32,
    last_evidence: I64,
    confidence_score: F64,
    source_channels: [String],
    created_at: I64,
    updated_at: I64,
    text_description: String,
    embedding: [F32],
    embedding_model: String
) =>
    preference <- AddN<CustomerPreferenceMemory>({
        customer_id: customer_id,
        preference_id: preference_id,
        preference_type: preference_type,
        category: category,
        subject: subject,
        strength: strength,
        is_active: is_active,
        evidence_count: evidence_count,
        last_evidence: last_evidence,
        confidence_score: confidence_score,
        source_channels: source_channels,
        created_at: created_at,
        updated_at: updated_at,
        text_description: text_description
    })
    embedding_node <- AddV<CustomerPreferenceEmbedding>(embedding, {
        text_description: text_description,
        embedding_model: embedding_model
    })
    edge <- AddE<HasPreferenceEmbedding>({
        created_at: created_at
    })::From(preference)::To(embedding_node)
    RETURN preference

// Search customer preferences by vector
QUERY search_customer_preferences(query_embedding: [F32], limit: I64) =>
    embeddings <- SearchV<CustomerPreferenceEmbedding>(query_embedding, limit)
    preferences <- embeddings::In<HasPreferenceEmbedding>
    RETURN preferences

// Hybrid search for preferences
QUERY search_customer_preferences_hybrid(
    query_embedding: [F32],
    limit: I64,
    customer_id: String
) =>
    embeddings <- SearchV<CustomerPreferenceEmbedding>(query_embedding, limit)
    preferences <- embeddings::In<HasPreferenceEmbedding>
    filtered <- preferences::WHERE(_::{customer_id}::EQ(customer_id))
    RETURN filtered

// Get customer preferences by ID
QUERY get_customer_preferences(customer_id: String) =>
    preferences <- N<CustomerPreferenceMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    RETURN preferences

// ============================================================================
// GENERAL GET QUERIES
// ============================================================================

// Get customer behaviors
QUERY get_customer_behaviors(customer_id: String) =>
    behaviors <- N<CustomerBehaviorMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    RETURN behaviors

// Get customer desires
QUERY get_customer_desires(customer_id: String) =>
    desires <- N<CustomerDesireMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    RETURN desires

// Get customer rules
QUERY get_customer_rules(customer_id: String) =>
    rules <- N<CustomerRuleMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    RETURN rules

// Get customer feedback
QUERY get_customer_feedback(customer_id: String) =>
    feedback <- N<CustomerFeedbackMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    RETURN feedback

// Get business services
QUERY get_business_services(business_id: String) =>
    services <- N<BusinessServiceMemory>::WHERE(_::{business_id}::EQ(business_id))
    RETURN services

// Get business locations
QUERY get_business_locations(business_id: String) =>
    locations <- N<BusinessLocationMemory>::WHERE(_::{business_id}::EQ(business_id))
    RETURN locations

// Get business hours
QUERY get_business_hours(business_id: String) =>
    hours <- N<BusinessHoursMemory>::WHERE(_::{business_id}::EQ(business_id))
    RETURN hours

// Get business social media
QUERY get_business_social_media(business_id: String) =>
    social <- N<BusinessSocialMemory>::WHERE(_::{business_id}::EQ(business_id))
    RETURN social

// Get business policies
QUERY get_business_policies(business_id: String) =>
    policies <- N<BusinessPolicyMemory>::WHERE(_::{business_id}::EQ(business_id))
    RETURN policies

// Get business events
QUERY get_business_events(business_id: String) =>
    events <- N<BusinessEventMemory>::WHERE(_::{business_id}::EQ(business_id))
    RETURN events

// ============================================================================
// TODO: ADD CREATE QUERIES FOR REMAINING MEMORY TYPES
// The following CREATE queries need to be added once helix check validates them:
// - add_customer_behavior_memory
// - add_customer_desire_memory
// - add_customer_rule_memory
// - add_customer_feedback_memory
// - add_business_service_memory
// - add_business_location_memory
// - add_business_hours_memory
// - add_business_social_memory
// - add_business_policy_memory
// - add_business_event_memory
//
// Note: These queries are triggering an internal HelixDB compiler panic.
// This appears to be a bug in HelixDB's query validator.
// ============================================================================
// ============================================================================
// COMPLETE UPDATE QUERIES - All Memory Types
// ============================================================================
// Correct syntax: variable <- traversal, then variable::UPDATE({})
// ============================================================================

// ============================================================================
// BUSINESS PRODUCT UPDATES
// ============================================================================

QUERY update_product_price(product_id: String, price: F64, updated_at: I64) =>
    product <- N<BusinessProductMemory>::WHERE(_::{product_id}::EQ(product_id))
    updated <- product::UPDATE({price: price, updated_at: updated_at})
    RETURN updated

QUERY update_product_availability(product_id: String, availability: String, updated_at: I64) =>
    product <- N<BusinessProductMemory>::WHERE(_::{product_id}::EQ(product_id))
    updated <- product::UPDATE({availability: availability, updated_at: updated_at})
    RETURN updated

QUERY update_product_full(
    product_id: String,
    product_name: String,
    price: F64,
    availability: String,
    description: String,
    updated_at: I64
) =>
    product <- N<BusinessProductMemory>::WHERE(_::{product_id}::EQ(product_id))
    updated <- product::UPDATE({
        product_name: product_name,
        price: price,
        availability: availability,
        description: description,
        updated_at: updated_at
    })
    RETURN updated

// ============================================================================
// BUSINESS SERVICE UPDATES
// ============================================================================

QUERY update_service_price(service_id: String, price: F64, updated_at: I64) =>
    service <- N<BusinessServiceMemory>::WHERE(_::{service_id}::EQ(service_id))
    updated <- service::UPDATE({price: price, updated_at: updated_at})
    RETURN updated

QUERY update_service_availability(service_id: String, availability: String, updated_at: I64) =>
    service <- N<BusinessServiceMemory>::WHERE(_::{service_id}::EQ(service_id))
    updated <- service::UPDATE({availability: availability, updated_at: updated_at})
    RETURN updated

// ============================================================================
// BUSINESS LOCATION UPDATES
// ============================================================================

QUERY update_location_address(
    location_id: String,
    address: String,
    city: String,
    state: String,
    postal_code: String,
    updated_at: I64
) =>
    location <- N<BusinessLocationMemory>::WHERE(_::{location_id}::EQ(location_id))
    updated <- location::UPDATE({
        address: address,
        city: city,
        state: state,
        postal_code: postal_code,
        updated_at: updated_at
    })
    RETURN updated

QUERY update_location_coordinates(location_id: String, latitude: F64, longitude: F64, updated_at: I64) =>
    location <- N<BusinessLocationMemory>::WHERE(_::{location_id}::EQ(location_id))
    updated <- location::UPDATE({latitude: latitude, longitude: longitude, updated_at: updated_at})
    RETURN updated

// ============================================================================
// BUSINESS HOURS UPDATES
// ============================================================================

QUERY update_business_hours_monday(hours_id: String, monday_open: String, monday_close: String, updated_at: I64) =>
    hours <- N<BusinessHoursMemory>::WHERE(_::{hours_id}::EQ(hours_id))
    updated <- hours::UPDATE({monday_open: monday_open, monday_close: monday_close, updated_at: updated_at})
    RETURN updated

QUERY update_business_hours_weekend(
    hours_id: String,
    saturday_open: String,
    saturday_close: String,
    sunday_open: String,
    sunday_close: String,
    updated_at: I64
) =>
    hours <- N<BusinessHoursMemory>::WHERE(_::{hours_id}::EQ(hours_id))
    updated <- hours::UPDATE({
        saturday_open: saturday_open,
        saturday_close: saturday_close,
        sunday_open: sunday_open,
        sunday_close: sunday_close,
        updated_at: updated_at
    })
    RETURN updated

// ============================================================================
// BUSINESS SOCIAL MEDIA UPDATES
// ============================================================================

QUERY update_social_stats(social_id: String, follower_count: I64, post_count: I64, updated_at: I64) =>
    social <- N<BusinessSocialMemory>::WHERE(_::{social_id}::EQ(social_id))
    updated <- social::UPDATE({
        follower_count: follower_count,
        post_count: post_count,
        last_updated: updated_at,
        updated_at: updated_at
    })
    RETURN updated

QUERY update_social_profile(social_id: String, handle: String, profile_url: String, description: String, updated_at: I64) =>
    social <- N<BusinessSocialMemory>::WHERE(_::{social_id}::EQ(social_id))
    updated <- social::UPDATE({
        handle: handle,
        profile_url: profile_url,
        description: description,
        updated_at: updated_at
    })
    RETURN updated

// ============================================================================
// BUSINESS POLICY UPDATES
// ============================================================================

QUERY update_policy_content(policy_id: String, content: String, version: String, updated_at: I64) =>
    policy <- N<BusinessPolicyMemory>::WHERE(_::{policy_id}::EQ(policy_id))
    updated <- policy::UPDATE({content: content, version: version, updated_at: updated_at})
    RETURN updated

QUERY update_policy_status(policy_id: String, is_active: Boolean, updated_at: I64) =>
    policy <- N<BusinessPolicyMemory>::WHERE(_::{policy_id}::EQ(policy_id))
    updated <- policy::UPDATE({is_active: is_active, updated_at: updated_at})
    RETURN updated

// ============================================================================
// BUSINESS EVENT UPDATES
// ============================================================================

QUERY update_event_dates(event_id: String, start_date: I64, end_date: I64, updated_at: I64) =>
    event <- N<BusinessEventMemory>::WHERE(_::{event_id}::EQ(event_id))
    updated <- event::UPDATE({start_date: start_date, end_date: end_date, updated_at: updated_at})
    RETURN updated

QUERY update_event_capacity(event_id: String, capacity: I32, updated_at: I64) =>
    event <- N<BusinessEventMemory>::WHERE(_::{event_id}::EQ(event_id))
    updated <- event::UPDATE({capacity: capacity, updated_at: updated_at})
    RETURN updated

// ============================================================================
// CUSTOMER BEHAVIOR UPDATES
// ============================================================================

QUERY update_behavior_context(behavior_id: String, context: String, updated_at: I64) =>
    behavior <- N<CustomerBehaviorMemory>::WHERE(_::{behavior_id}::EQ(behavior_id))
    updated <- behavior::UPDATE({context: context, updated_at: updated_at})
    RETURN updated

// ============================================================================
// CUSTOMER PREFERENCE UPDATES
// ============================================================================

QUERY update_preference_strength(
    preference_id: String,
    strength: String,
    evidence_count: I32,
    confidence_score: F64,
    last_evidence: I64,
    updated_at: I64
) =>
    preference <- N<CustomerPreferenceMemory>::WHERE(_::{preference_id}::EQ(preference_id))
    updated <- preference::UPDATE({
        strength: strength,
        evidence_count: evidence_count,
        confidence_score: confidence_score,
        last_evidence: last_evidence,
        updated_at: updated_at
    })
    RETURN updated

QUERY deactivate_preference(preference_id: String, updated_at: I64) =>
    preference <- N<CustomerPreferenceMemory>::WHERE(_::{preference_id}::EQ(preference_id))
    updated <- preference::UPDATE({is_active: false, updated_at: updated_at})
    RETURN updated

QUERY activate_preference(preference_id: String, updated_at: I64) =>
    preference <- N<CustomerPreferenceMemory>::WHERE(_::{preference_id}::EQ(preference_id))
    updated <- preference::UPDATE({is_active: true, updated_at: updated_at})
    RETURN updated

// ============================================================================
// CUSTOMER DESIRE UPDATES
// ============================================================================

QUERY update_desire_priority(desire_id: String, priority: String, updated_at: I64) =>
    desire <- N<CustomerDesireMemory>::WHERE(_::{desire_id}::EQ(desire_id))
    updated <- desire::UPDATE({priority: priority, updated_at: updated_at})
    RETURN updated

QUERY deactivate_desire(desire_id: String, updated_at: I64) =>
    desire <- N<CustomerDesireMemory>::WHERE(_::{desire_id}::EQ(desire_id))
    updated <- desire::UPDATE({is_active: false, updated_at: updated_at})
    RETURN updated

// ============================================================================
// CUSTOMER RULE UPDATES
// ============================================================================

QUERY update_rule_enforcement(rule_id: String, enforcement: String, updated_at: I64) =>
    rule <- N<CustomerRuleMemory>::WHERE(_::{rule_id}::EQ(rule_id))
    updated <- rule::UPDATE({enforcement: enforcement, updated_at: updated_at})
    RETURN updated

QUERY deactivate_rule(rule_id: String, updated_at: I64) =>
    rule <- N<CustomerRuleMemory>::WHERE(_::{rule_id}::EQ(rule_id))
    updated <- rule::UPDATE({is_active: false, updated_at: updated_at})
    RETURN updated

// ============================================================================
// CUSTOMER FEEDBACK UPDATES
// ============================================================================

QUERY update_feedback_resolution(feedback_id: String, resolved: Boolean, updated_at: I64) =>
    feedback <- N<CustomerFeedbackMemory>::WHERE(_::{feedback_id}::EQ(feedback_id))
    updated <- feedback::UPDATE({resolved: resolved, response_required: false, updated_at: updated_at})
    RETURN updated

QUERY update_feedback_rating(feedback_id: String, rating: I32, sentiment: String, updated_at: I64) =>
    feedback <- N<CustomerFeedbackMemory>::WHERE(_::{feedback_id}::EQ(feedback_id))
    updated <- feedback::UPDATE({rating: rating, sentiment: sentiment, updated_at: updated_at})
    RETURN updated

// ============================================================================
// ENTITY UPDATES
// ============================================================================

QUERY update_business_status(business_id: String, status: String) =>
    business <- N<Business>::WHERE(_::{business_id}::EQ(business_id))
    updated <- business::UPDATE({status: status})
    RETURN updated

QUERY update_customer_contact(customer_id: String, phone: String, email: String) =>
    customer <- N<Customer>::WHERE(_::{customer_id}::EQ(customer_id))
    updated <- customer::UPDATE({phone: phone, email: email})
    RETURN updated

QUERY update_customer_language(customer_id: String, language: String) =>
    customer <- N<Customer>::WHERE(_::{customer_id}::EQ(customer_id))
    updated <- customer::UPDATE({language: language})
    RETURN updated
// ============================================================================
// COMPLETE DELETE QUERIES - All Memory Types
// ============================================================================
// Correct syntax: DROP with direct traversal (no variable assignment)
// ============================================================================

// ============================================================================
// BUSINESS PRODUCT DELETES
// ============================================================================

QUERY delete_product(product_id: String) =>
    DROP N<BusinessProductMemory>::WHERE(_::{product_id}::EQ(product_id))
    RETURN "Deleted product"

QUERY delete_product_with_embedding(product_id: String) =>
    DROP N<BusinessProductMemory>::WHERE(_::{product_id}::EQ(product_id))::Out<HasProductEmbedding>
    DROP N<BusinessProductMemory>::WHERE(_::{product_id}::EQ(product_id))
    RETURN "Deleted product and embedding"

QUERY delete_product_embedding_edge_only(product_id: String) =>
    DROP N<BusinessProductMemory>::WHERE(_::{product_id}::EQ(product_id))::OutE<HasProductEmbedding>
    RETURN "Deleted product embedding edge"

QUERY delete_all_business_products(business_id: String) =>
    DROP N<BusinessProductMemory>::WHERE(_::{business_id}::EQ(business_id))
    RETURN "Deleted all products for business"

// ============================================================================
// BUSINESS SERVICE DELETES
// ============================================================================

QUERY delete_service(service_id: String) =>
    DROP N<BusinessServiceMemory>::WHERE(_::{service_id}::EQ(service_id))
    RETURN "Deleted service"

QUERY delete_service_with_embedding(service_id: String) =>
    DROP N<BusinessServiceMemory>::WHERE(_::{service_id}::EQ(service_id))::Out<HasServiceEmbedding>
    DROP N<BusinessServiceMemory>::WHERE(_::{service_id}::EQ(service_id))
    RETURN "Deleted service and embedding"

QUERY delete_all_business_services(business_id: String) =>
    DROP N<BusinessServiceMemory>::WHERE(_::{business_id}::EQ(business_id))
    RETURN "Deleted all services for business"

// ============================================================================
// BUSINESS LOCATION DELETES
// ============================================================================

QUERY delete_location(location_id: String) =>
    DROP N<BusinessLocationMemory>::WHERE(_::{location_id}::EQ(location_id))
    RETURN "Deleted location"

QUERY delete_location_with_embedding(location_id: String) =>
    DROP N<BusinessLocationMemory>::WHERE(_::{location_id}::EQ(location_id))::Out<HasLocationEmbedding>
    DROP N<BusinessLocationMemory>::WHERE(_::{location_id}::EQ(location_id))
    RETURN "Deleted location and embedding"

QUERY delete_all_business_locations(business_id: String) =>
    DROP N<BusinessLocationMemory>::WHERE(_::{business_id}::EQ(business_id))
    RETURN "Deleted all locations for business"

// ============================================================================
// BUSINESS HOURS DELETES
// ============================================================================

QUERY delete_hours(hours_id: String) =>
    DROP N<BusinessHoursMemory>::WHERE(_::{hours_id}::EQ(hours_id))
    RETURN "Deleted hours"

QUERY delete_hours_with_embedding(hours_id: String) =>
    DROP N<BusinessHoursMemory>::WHERE(_::{hours_id}::EQ(hours_id))::Out<HasHoursEmbedding>
    DROP N<BusinessHoursMemory>::WHERE(_::{hours_id}::EQ(hours_id))
    RETURN "Deleted hours and embedding"

QUERY delete_all_business_hours(business_id: String) =>
    DROP N<BusinessHoursMemory>::WHERE(_::{business_id}::EQ(business_id))
    RETURN "Deleted all hours for business"

// ============================================================================
// BUSINESS SOCIAL MEDIA DELETES
// ============================================================================

QUERY delete_social(social_id: String) =>
    DROP N<BusinessSocialMemory>::WHERE(_::{social_id}::EQ(social_id))
    RETURN "Deleted social media"

QUERY delete_social_with_embedding(social_id: String) =>
    DROP N<BusinessSocialMemory>::WHERE(_::{social_id}::EQ(social_id))::Out<HasSocialEmbedding>
    DROP N<BusinessSocialMemory>::WHERE(_::{social_id}::EQ(social_id))
    RETURN "Deleted social and embedding"

QUERY delete_all_business_social(business_id: String) =>
    DROP N<BusinessSocialMemory>::WHERE(_::{business_id}::EQ(business_id))
    RETURN "Deleted all social media for business"

// ============================================================================
// BUSINESS POLICY DELETES
// ============================================================================

QUERY delete_policy(policy_id: String) =>
    DROP N<BusinessPolicyMemory>::WHERE(_::{policy_id}::EQ(policy_id))
    RETURN "Deleted policy"

QUERY delete_policy_with_embedding(policy_id: String) =>
    DROP N<BusinessPolicyMemory>::WHERE(_::{policy_id}::EQ(policy_id))::Out<HasPolicyEmbedding>
    DROP N<BusinessPolicyMemory>::WHERE(_::{policy_id}::EQ(policy_id))
    RETURN "Deleted policy and embedding"

QUERY delete_all_business_policies(business_id: String) =>
    DROP N<BusinessPolicyMemory>::WHERE(_::{business_id}::EQ(business_id))
    RETURN "Deleted all policies for business"

// ============================================================================
// BUSINESS EVENT DELETES
// ============================================================================

QUERY delete_event(event_id: String) =>
    DROP N<BusinessEventMemory>::WHERE(_::{event_id}::EQ(event_id))
    RETURN "Deleted event"

QUERY delete_event_with_embedding(event_id: String) =>
    DROP N<BusinessEventMemory>::WHERE(_::{event_id}::EQ(event_id))::Out<HasEventEmbedding>
    DROP N<BusinessEventMemory>::WHERE(_::{event_id}::EQ(event_id))
    RETURN "Deleted event and embedding"

QUERY delete_all_business_events(business_id: String) =>
    DROP N<BusinessEventMemory>::WHERE(_::{business_id}::EQ(business_id))
    RETURN "Deleted all events for business"

// ============================================================================
// CUSTOMER BEHAVIOR DELETES
// ============================================================================

QUERY delete_behavior(behavior_id: String) =>
    DROP N<CustomerBehaviorMemory>::WHERE(_::{behavior_id}::EQ(behavior_id))
    RETURN "Deleted behavior"

QUERY delete_behavior_with_embedding(behavior_id: String) =>
    DROP N<CustomerBehaviorMemory>::WHERE(_::{behavior_id}::EQ(behavior_id))::Out<HasBehaviorEmbedding>
    DROP N<CustomerBehaviorMemory>::WHERE(_::{behavior_id}::EQ(behavior_id))
    RETURN "Deleted behavior and embedding"

QUERY delete_all_customer_behaviors(customer_id: String) =>
    DROP N<CustomerBehaviorMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    RETURN "Deleted all behaviors for customer"

// ============================================================================
// CUSTOMER PREFERENCE DELETES
// ============================================================================

QUERY delete_preference(preference_id: String) =>
    DROP N<CustomerPreferenceMemory>::WHERE(_::{preference_id}::EQ(preference_id))
    RETURN "Deleted preference"

QUERY delete_preference_with_embedding(preference_id: String) =>
    DROP N<CustomerPreferenceMemory>::WHERE(_::{preference_id}::EQ(preference_id))::Out<HasPreferenceEmbedding>
    DROP N<CustomerPreferenceMemory>::WHERE(_::{preference_id}::EQ(preference_id))
    RETURN "Deleted preference and embedding"

QUERY delete_all_customer_preferences(customer_id: String) =>
    DROP N<CustomerPreferenceMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    RETURN "Deleted all preferences for customer"

// ============================================================================
// CUSTOMER DESIRE DELETES
// ============================================================================

QUERY delete_desire(desire_id: String) =>
    DROP N<CustomerDesireMemory>::WHERE(_::{desire_id}::EQ(desire_id))
    RETURN "Deleted desire"

QUERY delete_desire_with_embedding(desire_id: String) =>
    DROP N<CustomerDesireMemory>::WHERE(_::{desire_id}::EQ(desire_id))::Out<HasDesireEmbedding>
    DROP N<CustomerDesireMemory>::WHERE(_::{desire_id}::EQ(desire_id))
    RETURN "Deleted desire and embedding"

QUERY delete_all_customer_desires(customer_id: String) =>
    DROP N<CustomerDesireMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    RETURN "Deleted all desires for customer"

// ============================================================================
// CUSTOMER RULE DELETES
// ============================================================================

QUERY delete_rule(rule_id: String) =>
    DROP N<CustomerRuleMemory>::WHERE(_::{rule_id}::EQ(rule_id))
    RETURN "Deleted rule"

QUERY delete_rule_with_embedding(rule_id: String) =>
    DROP N<CustomerRuleMemory>::WHERE(_::{rule_id}::EQ(rule_id))::Out<HasRuleEmbedding>
    DROP N<CustomerRuleMemory>::WHERE(_::{rule_id}::EQ(rule_id))
    RETURN "Deleted rule and embedding"

QUERY delete_all_customer_rules(customer_id: String) =>
    DROP N<CustomerRuleMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    RETURN "Deleted all rules for customer"

// ============================================================================
// CUSTOMER FEEDBACK DELETES
// ============================================================================

QUERY delete_feedback(feedback_id: String) =>
    DROP N<CustomerFeedbackMemory>::WHERE(_::{feedback_id}::EQ(feedback_id))
    RETURN "Deleted feedback"

QUERY delete_feedback_with_embedding(feedback_id: String) =>
    DROP N<CustomerFeedbackMemory>::WHERE(_::{feedback_id}::EQ(feedback_id))::Out<HasFeedbackEmbedding>
    DROP N<CustomerFeedbackMemory>::WHERE(_::{feedback_id}::EQ(feedback_id))
    RETURN "Deleted feedback and embedding"

QUERY delete_all_customer_feedback(customer_id: String) =>
    DROP N<CustomerFeedbackMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    RETURN "Deleted all feedback for customer"

// ============================================================================
// CASCADE DELETES - Delete entire memory hierarchies
// ============================================================================

QUERY delete_all_business_memories(business_id: String) =>
    DROP N<BusinessProductMemory>::WHERE(_::{business_id}::EQ(business_id))
    DROP N<BusinessServiceMemory>::WHERE(_::{business_id}::EQ(business_id))
    DROP N<BusinessLocationMemory>::WHERE(_::{business_id}::EQ(business_id))
    DROP N<BusinessHoursMemory>::WHERE(_::{business_id}::EQ(business_id))
    DROP N<BusinessSocialMemory>::WHERE(_::{business_id}::EQ(business_id))
    DROP N<BusinessPolicyMemory>::WHERE(_::{business_id}::EQ(business_id))
    DROP N<BusinessEventMemory>::WHERE(_::{business_id}::EQ(business_id))
    RETURN "Deleted all business memories"

QUERY delete_all_customer_memories(customer_id: String) =>
    DROP N<CustomerBehaviorMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    DROP N<CustomerPreferenceMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    DROP N<CustomerDesireMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    DROP N<CustomerRuleMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    DROP N<CustomerFeedbackMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    RETURN "Deleted all customer memories"

QUERY delete_customer_complete(customer_id: String) =>
    DROP N<CustomerBehaviorMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    DROP N<CustomerPreferenceMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    DROP N<CustomerDesireMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    DROP N<CustomerRuleMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    DROP N<CustomerFeedbackMemory>::WHERE(_::{customer_id}::EQ(customer_id))
    DROP N<Customer>::WHERE(_::{customer_id}::EQ(customer_id))
    RETURN "Deleted customer and all memories"

QUERY delete_business_complete(business_id: String) =>
    DROP N<BusinessProductMemory>::WHERE(_::{business_id}::EQ(business_id))
    DROP N<BusinessServiceMemory>::WHERE(_::{business_id}::EQ(business_id))
    DROP N<BusinessLocationMemory>::WHERE(_::{business_id}::EQ(business_id))
    DROP N<BusinessHoursMemory>::WHERE(_::{business_id}::EQ(business_id))
    DROP N<BusinessSocialMemory>::WHERE(_::{business_id}::EQ(business_id))
    DROP N<BusinessPolicyMemory>::WHERE(_::{business_id}::EQ(business_id))
    DROP N<BusinessEventMemory>::WHERE(_::{business_id}::EQ(business_id))
    DROP N<Business>::WHERE(_::{business_id}::EQ(business_id))
    RETURN "Deleted business and all memories"


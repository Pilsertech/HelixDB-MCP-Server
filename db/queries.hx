// HelixDB Queries for AI Memory Layer
// Contains working queries only

// ============================================================================
// FOUNDATIONAL ENTITY CREATION QUERIES  
// ============================================================================

// Create a new business entity
QUERY create_business(
    business_id: String,
    business_name: String,
    business_type: String,
    status: String,
    allow_collaboration: Boolean,
    metadata: String
) =>
    business <- AddN<Business>({
        business_id: business_id,
        business_name: business_name,
        business_type: business_type,
        status: status,
        allow_collaboration: allow_collaboration,
        metadata: metadata
    })
    RETURN business

// Create a new customer entity
QUERY create_customer(
    customer_id: String,
    customer_name: String,
    phone: String,
    email: String,
    language: String,
    metadata: String
) =>
    customer <- AddN<Customer>({
        customer_id: customer_id,
        customer_name: customer_name,
        phone: phone,
        email: email,
        language: language,
        metadata: metadata
    })
    RETURN customer

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
    embedding: [F64],
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
        composite_embedding_text: text_description,
        product_name: product_name,
        category_context: product_category,
        price_context: text_description,      // Using text_description for all contexts
        availability_context: text_description,
        feature_context: text_description,
        brand_context: text_description,
        customer_context: text_description,
        seasonal_context: text_description,
        specification_summary: specifications,
        use_case_context: description,
        competitor_context: competitor_analysis,
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "1.0"
    })
    edge <- AddE<HasProductEmbedding>({
        created_at: created_at
    })::From(product)::To(embedding_node)
    RETURN product

// Search products by vector similarity  
QUERY search_business_products(query_embedding: [F64], limit: I64) =>
    embeddings <- SearchV<BusinessProductEmbedding>(query_embedding, limit)
    products <- embeddings::In<HasProductEmbedding>
    RETURN products

// Semantic search: HelixDB generates embedding from text
QUERY search_business_products_semantic(query_text: String, k: I64) =>
    results <- SearchV<BusinessProductEmbedding>(Embed(query_text), k)
    products <- results::In<HasProductEmbedding>
    RETURN products

// Hybrid search: vector + structured filters
QUERY search_business_products_hybrid(
    query_embedding: [F64],
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

// Add a new service memory node
QUERY add_business_service_memory(
    business_id: String,
    service_id: String,
    service_name: String,
    service_category: String,
    price: F64,
    currency: String,
    duration_minutes: I32,
    availability: String,
    description: String,
    requirements: [String],
    deliverables: [String],
    tags: [String],
    created_at: I64,
    updated_at: I64,
    text_description: String,
    embedding: [F64],
    embedding_model: String
) =>
    service <- AddN<BusinessServiceMemory>({
        business_id: business_id,
        service_id: service_id,
        service_name: service_name,
        service_category: service_category,
        price: price,
        currency: currency,
        duration_minutes: duration_minutes,
        availability: availability,
        description: description,
        requirements: requirements,
        deliverables: deliverables,
        tags: tags,
        created_at: created_at,
        updated_at: updated_at,
        text_description: text_description
    })
    embedding_node <- AddV<BusinessServiceEmbedding>(embedding, {
        composite_embedding_text: text_description,
        service_name: service_name,
        category_context: service_category,
        price_context: currency,
        duration_context: "minutes",
        availability_context: availability,
        skill_context: "professional",
        target_context: description,
        location_context: "flexible",
        urgency_context: "standard",
        outcome_context: description,
        requirement_context: description,
        deliverable_context: description,
        seasonal_context: "year-round",
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "1.0"
    })
    edge <- AddE<HasServiceEmbedding>({
        created_at: created_at
    })::From(service)::To(embedding_node)
    RETURN service

// Add a new location memory node
QUERY add_business_location_memory(
    business_id: String,
    location_id: String,
    location_name: String,
    address: String,
    city: String,
    state: String,
    country: String,
    postal_code: String,
    latitude: F64,
    longitude: F64,
    location_type: String,
    accessibility: [String],
    parking_info: String,
    created_at: I64,
    updated_at: I64,
    text_description: String,
    embedding: [F64],
    embedding_model: String
) =>
    location <- AddN<BusinessLocationMemory>({
        business_id: business_id,
        location_id: location_id,
        location_name: location_name,
        address: address,
        city: city,
        state: state,
        country: country,
        postal_code: postal_code,
        latitude: latitude,
        longitude: longitude,
        location_type: location_type,
        accessibility: accessibility,
        parking_info: parking_info,
        created_at: created_at,
        updated_at: updated_at,
        text_description: text_description
    })
    embedding_node <- AddV<BusinessLocationEmbedding>(embedding, {
        composite_embedding_text: text_description,
        location_name: location_name,
        address_context: address,
        area_context: city,
        accessibility_context: parking_info,
        transport_context: "accessible",
        parking_context: parking_info,
        landmark_context: address,
        neighborhood_context: city,
        convenience_context: "accessible",
        building_context: location_type,
        entrance_context: "main entrance",
        hours_context: "business hours",
        security_context: "public",
        visitor_context: "welcome",
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "1.0"
    })
    edge <- AddE<HasLocationEmbedding>({
        created_at: created_at
    })::From(location)::To(embedding_node)
    RETURN location

// Add a new hours memory node
QUERY add_business_hours_memory(
    business_id: String,
    hours_id: String,
    schedule_type: String,
    monday_open: String,
    monday_close: String,
    tuesday_open: String,
    tuesday_close: String,
    wednesday_open: String,
    wednesday_close: String,
    thursday_open: String,
    thursday_close: String,
    friday_open: String,
    friday_close: String,
    saturday_open: String,
    saturday_close: String,
    sunday_open: String,
    sunday_close: String,
    timezone: String,
    exceptions: String,
    created_at: I64,
    updated_at: I64,
    text_description: String,
    embedding: [F64],
    embedding_model: String
) =>
    hours <- AddN<BusinessHoursMemory>({
        business_id: business_id,
        hours_id: hours_id,
        schedule_type: schedule_type,
        monday_open: monday_open,
        monday_close: monday_close,
        tuesday_open: tuesday_open,
        tuesday_close: tuesday_close,
        wednesday_open: wednesday_open,
        wednesday_close: wednesday_close,
        thursday_open: thursday_open,
        thursday_close: thursday_close,
        friday_open: friday_open,
        friday_close: friday_close,
        saturday_open: saturday_open,
        saturday_close: saturday_close,
        sunday_open: sunday_open,
        sunday_close: sunday_close,
        timezone: timezone,
        exceptions: exceptions,
        created_at: created_at,
        updated_at: updated_at,
        text_description: text_description
    })
    embedding_node <- AddV<BusinessHoursEmbedding>(embedding, {
        composite_embedding_text: text_description,
        schedule_pattern: schedule_type,
        availability_context: "business hours",
        day_context: "weekdays weekend",
        time_context: "standard hours",
        convenience_context: "regular schedule",
        access_context: "open during hours",
        special_context: exceptions,
        customer_context: "all welcome",
        service_context: "appointments available",
        timezone_context: timezone,
        flexibility_context: schedule_type,
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "1.0"
    })
    edge <- AddE<HasHoursEmbedding>({
        created_at: created_at
    })::From(hours)::To(embedding_node)
    RETURN hours

// Add a new social media memory node
QUERY add_business_social_media_memory(
    business_id: String,
    social_id: String,
    platform: String,
    handle: String,
    profile_url: String,
    follower_count: I64,
    post_count: I64,
    description: String,
    contact_info: String,
    last_updated: I64,
    created_at: I64,
    updated_at: I64,
    text_description: String,
    embedding: [F64],
    embedding_model: String
) =>
    social <- AddN<BusinessSocialMemory>({
        business_id: business_id,
        social_id: social_id,
        platform: platform,
        handle: handle,
        profile_url: profile_url,
        follower_count: follower_count,
        post_count: post_count,
        description: description,
        contact_info: contact_info,
        last_updated: last_updated,
        created_at: created_at,
        updated_at: updated_at,
        text_description: text_description
    })
    embedding_node <- AddV<BusinessSocialEmbedding>(embedding, {
        composite_embedding_text: text_description,
        platform_context: platform,
        engagement_context: "active responsive",
        audience_context: "followers customers",
        content_context: "updates promotions",
        interaction_context: "responsive helpful",
        reach_context: "local regional",
        frequency_context: "regular active",
        tone_context: "professional friendly",
        purpose_context: "marketing customer service",
        verification_context: "authentic trusted",
        contact_context: contact_info,
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "1.0"
    })
    edge <- AddE<HasSocialEmbedding>({
        created_at: created_at
    })::From(social)::To(embedding_node)
    RETURN social

// Add a new policy memory node
QUERY add_business_policy_memory(
    business_id: String,
    policy_id: String,
    policy_type: String,
    policy_name: String,
    content: String,
    effective_date: I64,
    version: String,
    is_active: Boolean,
    tags: [String],
    created_at: I64,
    updated_at: I64,
    text_description: String,
    embedding: [F64],
    embedding_model: String
) =>
    policy <- AddN<BusinessPolicyMemory>({
        business_id: business_id,
        policy_id: policy_id,
        policy_type: policy_type,
        policy_name: policy_name,
        content: content,
        effective_date: effective_date,
        version: version,
        is_active: is_active,
        tags: tags,
        created_at: created_at,
        updated_at: updated_at,
        text_description: text_description
    })
    embedding_node <- AddV<BusinessPolicyEmbedding>(embedding, {
        composite_embedding_text: text_description,
        policy_name: policy_name,
        type_context: policy_type,
        scope_context: "customers all users",
        timeline_context: "effective immediately",
        conditions_context: content,
        process_context: "standard procedure",
        cost_context: "no additional cost",
        restriction_context: "reasonable limits",
        communication_context: "notification provided",
        compliance_context: "required policy",
        customer_impact_context: "customer friendly",
        urgency_context: "standard processing",
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "1.0"
    })
    edge <- AddE<HasPolicyEmbedding>({
        created_at: created_at
    })::From(policy)::To(embedding_node)
    RETURN policy

// Add a new event memory node
QUERY add_business_event_memory(
    business_id: String,
    event_id: String,
    event_name: String,
    event_type: String,
    start_date: I64,
    end_date: I64,
    description: String,
    location: String,
    capacity: I32,
    registration_required: Boolean,
    tags: [String],
    created_at: I64,
    updated_at: I64,
    text_description: String,
    embedding: [F64],
    embedding_model: String
) =>
    event <- AddN<BusinessEventMemory>({
        business_id: business_id,
        event_id: event_id,
        event_name: event_name,
        event_type: event_type,
        start_date: start_date,
        end_date: end_date,
        description: description,
        location: location,
        capacity: capacity,
        registration_required: registration_required,
        tags: tags,
        created_at: created_at,
        updated_at: updated_at,
        text_description: text_description
    })
    embedding_node <- AddV<BusinessEventEmbedding>(embedding, {
        composite_embedding_text: text_description,
        event_name: event_name,
        type_context: event_type,
        timing_context: "scheduled event",
        duration_context: "limited time",
        format_context: location,
        audience_context: "everyone welcome",
        capacity_context: "limited seats",
        access_context: "registration required",
        location_context: location,
        urgency_context: "register now",
        benefit_context: "valuable opportunity",
        requirement_context: "none required",
        seasonal_context: "special event",
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "1.0"
    })
    edge <- AddE<HasEventEmbedding>({
        created_at: created_at
    })::From(event)::To(embedding_node)
    RETURN event

// Add a new information memory node
QUERY add_business_information_memory(
    business_id: String,
    info_id: String,
    info_type: String,
    title: String,
    content: String,
    category: String,
    tags: [String],
    created_at: I64,
    updated_at: I64,
    text_description: String,
    embedding: [F64],
    embedding_model: String
) =>
    info <- AddN<BusinessInformationMemory>({
        business_id: business_id,
        info_id: info_id,
        info_type: info_type,
        title: title,
        content: content,
        category: category,
        tags: tags,
        created_at: created_at,
        updated_at: updated_at,
        text_description: text_description
    })
    embedding_node <- AddV<BusinessInformationEmbedding>(embedding, {
        composite_embedding_text: text_description,
        title: title,
        type_context: info_type,
        category_context: category,
        content_context: content,
        audience_context: "general audience",
        format_context: "text document",
        update_context: "current information",
        importance_context: "useful reference",
        access_context: "available to all",
        language_context: "english",
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "1.0"
    })
    edge <- AddE<HasInformationEmbedding>({
        created_at: created_at
    })::From(info)::To(embedding_node)
    RETURN info

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
    embedding: [F64],
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
        composite_embedding_text: text_description,
        preference_subject: subject,
        type_context: preference_type,
        category_context: category,
        strength_context: strength,
        reliability_context: "consistent",
        scope_context: category,
        trigger_context: "general",
        value_context: subject,
        lifestyle_context: category,
        decision_context: strength,
        communication_context: "flexible",
        timing_context: "general",
        brand_context: "flexible",
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "1.0"
    })
    edge <- AddE<HasPreferenceEmbedding>({
        created_at: created_at
    })::From(preference)::To(embedding_node)
    RETURN preference

// Search customer preferences by vector
QUERY search_customer_preferences(query_embedding: [F64], limit: I64) =>
    embeddings <- SearchV<CustomerPreferenceEmbedding>(query_embedding, limit)
    preferences <- embeddings::In<HasPreferenceEmbedding>
    RETURN preferences

// Hybrid search for preferences
QUERY search_customer_preferences_hybrid(
    query_embedding: [F64],
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

// Add a new customer behavior memory node
QUERY add_customer_behavior_memory(
    customer_id: String,
    behavior_id: String,
    behavior_type: String,
    action: String,
    context: String,
    timestamp: I64,
    channel: String,
    duration_seconds: I32,
    metadata: String,
    created_at: I64,
    updated_at: I64,
    text_description: String,
    embedding: [F64],
    embedding_model: String
) =>
    behavior <- AddN<CustomerBehaviorMemory>({
        customer_id: customer_id,
        behavior_id: behavior_id,
        behavior_type: behavior_type,
        action: action,
        context: context,
        timestamp: timestamp,
        channel: channel,
        duration_seconds: duration_seconds,
        metadata: metadata,
        created_at: created_at,
        updated_at: updated_at,
        text_description: text_description
    })
    embedding_node <- AddV<CustomerBehaviorEmbedding>(embedding, {
        composite_embedding_text: text_description,
        behavior_type: behavior_type,
        action_context: action,
        channel_context: channel,
        engagement_context: "focused interaction",
        intent_context: "exploring options",
        frequency_context: "regular visitor",
        timing_context: "business hours",
        device_context: "mobile desktop",
        location_context: "home office",
        mood_context: "interested engaged",
        interaction_context: "self-service",
        outcome_context: "completed successfully",
        value_context: "meaningful engagement",
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "1.0"
    })
    edge <- AddE<HasBehaviorEmbedding>({
        created_at: created_at
    })::From(behavior)::To(embedding_node)
    RETURN behavior

// Add a new customer desire memory node
QUERY add_customer_desire_memory(
    customer_id: String,
    desire_id: String,
    desire_type: String,
    category: String,
    description: String,
    priority: String,
    timeframe: String,
    budget_range: String,
    is_active: Boolean,
    created_at: I64,
    updated_at: I64,
    text_description: String,
    embedding: [F64],
    embedding_model: String
) =>
    desire <- AddN<CustomerDesireMemory>({
        customer_id: customer_id,
        desire_id: desire_id,
        desire_type: desire_type,
        category: category,
        description: description,
        priority: priority,
        timeframe: timeframe,
        budget_range: budget_range,
        is_active: is_active,
        created_at: created_at,
        updated_at: updated_at,
        text_description: text_description
    })
    embedding_node <- AddV<CustomerDesireEmbedding>(embedding, {
        composite_embedding_text: text_description,
        desire_subject: description,
        urgency_context: timeframe,
        budget_context: budget_range,
        purpose_context: category,
        quality_context: "standard premium",
        feature_context: description,
        brand_context: "flexible preference",
        timing_context: timeframe,
        research_context: "exploring options",
        motivation_context: desire_type,
        constraint_context: budget_range,
        outcome_context: "problem solving",
        influence_context: "self decided",
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "1.0"
    })
    edge <- AddE<HasDesireEmbedding>({
        created_at: created_at
    })::From(desire)::To(embedding_node)
    RETURN desire

// Add a new customer rule memory node
QUERY add_customer_rule_memory(
    customer_id: String,
    rule_id: String,
    rule_type: String,
    category: String,
    rule_description: String,
    enforcement: String,
    exceptions: [String],
    is_active: Boolean,
    created_at: I64,
    updated_at: I64,
    text_description: String,
    embedding: [F64],
    embedding_model: String
) =>
    rule <- AddN<CustomerRuleMemory>({
        customer_id: customer_id,
        rule_id: rule_id,
        rule_type: rule_type,
        category: category,
        rule_description: rule_description,
        enforcement: enforcement,
        exceptions: exceptions,
        is_active: is_active,
        created_at: created_at,
        updated_at: updated_at,
        text_description: text_description
    })
    embedding_node <- AddV<CustomerRuleEmbedding>(embedding, {
        composite_embedding_text: text_description,
        rule_subject: rule_description,
        enforcement_context: enforcement,
        scope_context: category,
        compliance_context: "always required",
        priority_context: "important preference",
        consequence_context: "strong preference",
        communication_context: category,
        timing_context: "immediate response",
        privacy_context: "confidential private",
        business_context: "professional interaction",
        accessibility_context: "standard access",
        cultural_context: "respectful approach",
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "1.0"
    })
    edge <- AddE<HasRuleEmbedding>({
        created_at: created_at
    })::From(rule)::To(embedding_node)
    RETURN rule

// Add a new customer feedback memory node
QUERY add_customer_feedback_memory(
    customer_id: String,
    feedback_id: String,
    feedback_type: String,
    subject: String,
    rating: I32,
    sentiment: String,
    channel: String,
    response_required: Boolean,
    resolved: Boolean,
    created_at: I64,
    updated_at: I64,
    text_description: String,
    embedding: [F64],
    embedding_model: String
) =>
    feedback <- AddN<CustomerFeedbackMemory>({
        customer_id: customer_id,
        feedback_id: feedback_id,
        feedback_type: feedback_type,
        subject: subject,
        rating: rating,
        sentiment: sentiment,
        channel: channel,
        response_required: response_required,
        resolved: resolved,
        created_at: created_at,
        updated_at: updated_at,
        text_description: text_description
    })
    embedding_node <- AddV<CustomerFeedbackEmbedding>(embedding, {
        composite_embedding_text: text_description,
        feedback_subject: subject,
        sentiment_context: sentiment,
        rating_context: "customer satisfaction",
        category_context: feedback_type,
        urgency_context: "standard priority",
        resolution_context: "pending review",
        channel_context: channel,
        tone_context: "constructive feedback",
        detail_context: "specific feedback",
        impact_context: "valuable input",
        credibility_context: "verified customer",
        actionable_context: "actionable feedback",
        public_context: "private feedback",
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "1.0"
    })
    edge <- AddE<HasFeedbackEmbedding>({
        created_at: created_at
    })::From(feedback)::To(embedding_node)
    RETURN feedback

// Add a new customer communication memory node
QUERY add_customer_communication_memory(
    customer_id: String,
    business_id: String,
    communication_id: String,
    contact_method: String,
    contact_reason: String,
    timestamp: I64,
    duration_seconds: I32,
    resolution_status: String,
    agent_id: String,
    channel_details: String,
    created_at: I64,
    updated_at: I64,
    text_interaction: String,
    embedding: [F64],
    embedding_model: String
) =>
    communication <- AddN<CustomerBusinessCommunication>({
        customer_id: customer_id,
        business_id: business_id,
        communication_id: communication_id,
        contact_method: contact_method,
        contact_reason: contact_reason,
        timestamp: timestamp,
        duration_seconds: duration_seconds,
        resolution_status: resolution_status,
        agent_id: agent_id,
        channel_details: channel_details,
        created_at: created_at,
        updated_at: updated_at,
        text_interaction: text_interaction
    })
    embedding_node <- AddV<CustomerCommunicationEmbedding>(embedding, {
        composite_embedding_text: text_interaction,
        contact_method: contact_method,
        contact_reason: contact_reason,
        urgency_context: "routine inquiry",
        tone_context: "professional friendly",
        resolution_context: resolution_status,
        complexity_context: "straightforward",
        agent_performance_context: "helpful professional",
        customer_mood_context: "satisfied",
        outcome_context: "successful resolution",
        efficiency_context: "timely response",
        follow_up_context: "completed",
        channel_context: contact_method,
        expertise_context: "knowledgeable",
        satisfaction_context: "satisfied customer",
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "1.0"
    })
    edge <- AddE<HasCommunicationEmbedding>({
        created_at: created_at
    })::From(communication)::To(embedding_node)
    RETURN communication

// Add customer location visit memory node
QUERY add_customer_location_visit(
    customer_id: String,
    location_id: String,
    visit_id: String,
    visit_type: String,
    timestamp: I64,
    duration_minutes: I32,
    party_size: I32,
    purchase_made: Boolean,
    purchase_amount: F64,
    currency: String,
    created_at: I64,
    updated_at: I64,
    text_experience: String,
    embedding: [F64],
    embedding_model: String
) =>
    visit <- AddN<CustomerLocationVisit>({
        customer_id: customer_id,
        location_id: location_id,
        visit_id: visit_id,
        visit_type: visit_type,
        timestamp: timestamp,
        duration_minutes: duration_minutes,
        party_size: party_size,
        purchase_made: purchase_made,
        purchase_amount: purchase_amount,
        currency: currency,
        created_at: created_at,
        updated_at: updated_at,
        text_experience: text_experience
    })
    embedding_node <- AddV<CustomerLocationVisitEmbedding>(embedding, {
        composite_embedding_text: text_experience,
        visit_type: visit_type,
        accessibility_context: "easy access",
        atmosphere_context: "welcoming comfortable",
        navigation_context: "easy to find",
        staff_context: "helpful friendly",
        crowd_context: "comfortable level",
        convenience_context: "convenient visit",
        purchase_context: "browsed considered",
        satisfaction_context: "satisfied experience",
        return_context: "will return",
        timing_context: "good timing",
        logistics_context: "smooth process",
        comparison_context: "met expectations",
        value_context: "good experience",
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "1.0"
    })
    edge <- AddE<HasLocationVisitEmbedding>({
        created_at: created_at
    })::From(visit)::To(embedding_node)
    RETURN visit

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

// Get business information
QUERY get_business_information(business_id: String) =>
    information <- N<BusinessInformationMemory>::WHERE(_::{business_id}::EQ(business_id))
    RETURN information

// ============================================================================
// INFORMATION RELATIONSHIP QUERIES - Create Network of Knowledge
// ============================================================================

// Link two information documents as related
QUERY link_related_information(
    from_info_id: String,
    to_info_id: String,
    relationship_type: String,
    strength: I32,
    notes: String
) =>
    from_info <- N<BusinessInformationMemory>({info_id: from_info_id})
    to_info <- N<BusinessInformationMemory>({info_id: to_info_id})
    edge <- AddE<RelatedInformation>({
        relationship_type: relationship_type,
        strength: strength,
        notes: notes
    })::From(from_info)::To(to_info)
    RETURN edge

// Create prerequisite relationship between information documents
QUERY link_prerequisite_information(
    prerequisite_info_id: String,
    dependent_info_id: String,
    notes: String
) =>
    prereq <- N<BusinessInformationMemory>({info_id: prerequisite_info_id})
    dependent <- N<BusinessInformationMemory>({info_id: dependent_info_id})
    edge <- AddE<PrerequisiteFor>({
        notes: notes
    })::From(prereq)::To(dependent)
    RETURN edge

// Link information documents as part of a series
QUERY link_series_information(
    from_info_id: String,
    to_info_id: String,
    series_name: String,
    order: I32
) =>
    from_info <- N<BusinessInformationMemory>({info_id: from_info_id})
    to_info <- N<BusinessInformationMemory>({info_id: to_info_id})
    edge <- AddE<PartOfSeries>({
        series_name: series_name,
        order: order
    })::From(from_info)::To(to_info)
    RETURN edge

// Create reference relationship between information documents
QUERY link_reference_information(
    referencing_info_id: String,
    referenced_info_id: String,
    reference_type: String,
    page_section: String
) =>
    referencing <- N<BusinessInformationMemory>({info_id: referencing_info_id})
    referenced <- N<BusinessInformationMemory>({info_id: referenced_info_id})
    edge <- AddE<References>({
        reference_type: reference_type,
        page_section: page_section
    })::From(referencing)::To(referenced)
    RETURN edge

// Link information to product
QUERY link_information_to_product(
    info_id: String,
    product_id: String,
    info_type: String,
    notes: String
) =>
    info <- N<BusinessInformationMemory>({info_id: info_id})
    product <- N<BusinessProductMemory>({product_id: product_id})
    edge <- AddE<InformationAboutProduct>({
        info_type: info_type,
        notes: notes
    })::From(info)::To(product)
    RETURN edge

// Link information to service
QUERY link_information_to_service(
    info_id: String,
    service_id: String,
    info_type: String,
    notes: String
) =>
    info <- N<BusinessInformationMemory>({info_id: info_id})
    service <- N<BusinessServiceMemory>({service_id: service_id})
    edge <- AddE<InformationAboutService>({
        info_type: info_type,
        notes: notes
    })::From(info)::To(service)
    RETURN edge

// Link information to location
QUERY link_information_to_location(
    info_id: String,
    location_id: String,
    info_type: String,
    notes: String
) =>
    info <- N<BusinessInformationMemory>({info_id: info_id})
    location <- N<BusinessLocationMemory>({location_id: location_id})
    edge <- AddE<InformationForLocation>({
        info_type: info_type,
        notes: notes
    })::From(info)::To(location)
    RETURN edge

// Link information to event
QUERY link_information_to_event(
    info_id: String,
    event_id: String,
    info_type: String,
    notes: String
) =>
    info <- N<BusinessInformationMemory>({info_id: info_id})
    event <- N<BusinessEventMemory>({event_id: event_id})
    edge <- AddE<InformationForEvent>({
        info_type: info_type,
        notes: notes
    })::From(info)::To(event)
    RETURN edge

// Query information relationships
QUERY get_related_information(info_id: String) =>
    info <- N<BusinessInformationMemory>({info_id: info_id})
    related <- info::Out<RelatedInformation>
    RETURN related

// UNCOMMENTED FOR TESTING - get_prerequisites_for_info
QUERY get_prerequisites_for_info(info_id: String) =>
    info <- N<BusinessInformationMemory>({info_id: info_id})
    prereqs <- info::In<PrerequisiteFor>
    RETURN prereqs

// UNCOMMENTED FOR TESTING - get_dependent_information
QUERY get_dependent_information(info_id: String) =>
    info <- N<BusinessInformationMemory>({info_id: info_id})
    dependents <- info::Out<PrerequisiteFor>
    RETURN dependents

// UNCOMMENTED FOR TESTING - get_series_information
QUERY get_series_information(series_name: String) =>
    series <- N<BusinessInformationMemory>::OutE<PartOfSeries>::WHERE(_::{series_name}::EQ(series_name))
    RETURN series

// UNCOMMENTED FOR TESTING - get_information_references
QUERY get_information_references(info_id: String) =>
    info <- N<BusinessInformationMemory>({info_id: info_id})
    references <- info::Out<References>
    RETURN references

// UNCOMMENTED FOR TESTING - get_referenced_by_info
QUERY get_referenced_by_info(info_id: String) =>
    info <- N<BusinessInformationMemory>({info_id: info_id})
    references <- info::In<References>
    RETURN references

// UNCOMMENTED FOR TESTING - get_product_information
QUERY get_product_information(product_id: String) =>
    product <- N<BusinessProductMemory>({product_id: product_id})
    info <- product::In<InformationAboutProduct>
    RETURN info

// UNCOMMENTED FOR TESTING - get_service_information
QUERY get_service_information(service_id: String) =>
    service <- N<BusinessServiceMemory>({service_id: service_id})
    info <- service::In<InformationAboutService>
    RETURN info

// UNCOMMENTED FOR TESTING - get_location_information
QUERY get_location_information(location_id: String) =>
    location <- N<BusinessLocationMemory>({location_id: location_id})
    info <- location::In<InformationForLocation>
    RETURN info

// UNCOMMENTED FOR TESTING - get_event_information
QUERY get_event_information(event_id: String) =>
    event <- N<BusinessEventMemory>({event_id: event_id})
    info <- event::In<InformationForEvent>
    RETURN info

// ============================================================================
// ✅ ALL CREATE QUERIES IMPLEMENTED
// All memory creation queries are now implemented and working:
// Business Memory Types:
// - add_business_product_memory ✅
// - add_business_service_memory ✅
// - add_business_location_memory ✅
// - add_business_hours_memory ✅
// - add_business_social_media_memory ✅
// - add_business_policy_memory ✅
// - add_business_event_memory ✅
// - add_business_information_memory ✅
//
// Customer Memory Types:
// - add_customer_behavior_memory ✅
// - add_customer_preference_memory ✅
// - add_customer_desire_memory ✅
// - add_customer_rule_memory ✅
// - add_customer_feedback_memory ✅
// - add_customer_communication_memory ✅
// ============================================================================
// ============================================================================
// COMPLETE UPDATE QUERIES - All Memory Types
// ============================================================================
// Correct syntax: variable <- traversal, then variable::UPDATE({})
// ============================================================================

// ============================================================================
// REMOVED: OLD BROKEN UPDATE QUERIES 
// These queries did NOT update vectors - use new vector-aware queries instead
// ============================================================================





// ============================================================================
// BUSINESS LOCATION UPDATES
// ============================================================================





















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
// BUSINESS INFORMATION DELETES
// ============================================================================

// UNCOMMENTED FOR TESTING - delete_information
QUERY delete_information(info_id: String) =>
    DROP N<BusinessInformationMemory>({info_id: info_id})
    RETURN "Deleted information"

// UNCOMMENTED FOR TESTING - delete_information_with_embedding
QUERY delete_information_with_embedding(info_id: String) =>
    DROP N<BusinessInformationMemory>({info_id: info_id})::Out<HasInformationEmbedding>
    DROP N<BusinessInformationMemory>({info_id: info_id})
    RETURN "Deleted information and embedding"

// UNCOMMENTED FOR TESTING - delete_all_business_information
QUERY delete_all_business_information(business_id: String) =>
    DROP N<BusinessInformationMemory>({business_id: business_id})
    RETURN "Deleted all information for business"

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

// ============================================================================
// CUSTOMER INTERACTION QUERIES (NEW ARCHITECTURE)
// ============================================================================

// Add customer product interaction with embedding
QUERY add_customer_product_interaction(
    customer_id: String,
    product_id: String,
    interaction_id: String,
    interaction_type: String,
    rating: I32,
    timestamp: I64,
    channel: String,
    session_duration: I32,
    purchase_amount: F64,
    currency: String,
    issue_category: String,
    resolution_status: String,
    created_at: I64,
    updated_at: I64,
    text_reason: String,
    embedding: [F64],
    embedding_model: String
) =>
    interaction <- AddN<CustomerProductInteraction>({
        customer_id: customer_id,
        product_id: product_id,
        interaction_id: interaction_id,
        interaction_type: interaction_type,
        rating: rating,
        timestamp: timestamp,
        channel: channel,
        session_duration: session_duration,
        purchase_amount: purchase_amount,
        currency: currency,
        issue_category: issue_category,
        resolution_status: resolution_status,
        created_at: created_at,
        updated_at: updated_at,
        text_reason: text_reason
    })
    embedding_node <- AddV<CustomerProductInteractionEmbedding>(embedding, {
        composite_embedding_text: text_reason,
        interaction_type: interaction_type,
        sentiment_context: interaction_type,
        engagement_context: channel,
        outcome_context: resolution_status,
        value_context: currency,
        feature_context: text_reason,
        comparison_context: "standard",
        timing_context: channel,
        channel_context: channel,
        decision_context: interaction_type,
        influence_context: "self",
        experience_context: "standard",
        resolution_context: resolution_status,
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "v1.0"
    })
    embed_edge <- AddE<HasProductInteractionEmbedding>({
        created_at: created_at
    })::From(interaction)::To(embedding_node)
    RETURN interaction

// Add customer service interaction with embedding
QUERY add_customer_service_interaction(
    customer_id: String,
    service_id: String,
    interaction_id: String,
    interaction_type: String,
    satisfaction_rating: I32,
    timestamp: I64,
    duration_actual: I32,
    cost_actual: F64,
    currency: String,
    outcome: String,
    created_at: I64,
    updated_at: I64,
    text_feedback: String,
    embedding: [F64],
    embedding_model: String
) =>
    interaction <- AddN<CustomerServiceInteraction>({
        customer_id: customer_id,
        service_id: service_id,
        interaction_id: interaction_id,
        interaction_type: interaction_type,
        satisfaction_rating: satisfaction_rating,
        timestamp: timestamp,
        duration_actual: duration_actual,
        cost_actual: cost_actual,
        currency: currency,
        outcome: outcome,
        created_at: created_at,
        updated_at: updated_at,
        text_feedback: text_feedback
    })
    embedding_node <- AddV<CustomerServiceInteractionEmbedding>(embedding, {
        composite_embedding_text: text_feedback,
        interaction_type: interaction_type,
        satisfaction_context: outcome,
        service_quality_context: outcome,
        timing_context: "standard",
        communication_context: text_feedback,
        outcome_context: outcome,
        value_context: currency,
        experience_context: text_feedback,
        staff_context: "professional",
        process_context: interaction_type,
        follow_up_context: "standard",
        recommendation_context: outcome,
        repeat_context: outcome,
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "v1.0"
    })
    embed_edge <- AddE<HasServiceInteractionEmbedding>({
        created_at: created_at
    })::From(interaction)::To(embedding_node)
    RETURN interaction

// Search customer product interactions by embedding
QUERY search_customer_product_interactions(query_embedding: [F64], limit: I64) =>
    embeddings <- SearchV<CustomerProductInteractionEmbedding>(query_embedding, limit)
    interactions <- embeddings::In<HasProductInteractionEmbedding>
    RETURN interactions

// Search customer service interactions by embedding
QUERY search_customer_service_interactions(query_embedding: [F64], limit: I64) =>
    embeddings <- SearchV<CustomerServiceInteractionEmbedding>(query_embedding, limit)
    interactions <- embeddings::In<HasServiceInteractionEmbedding>
    RETURN interactions

// Get customer product interactions
QUERY get_customer_product_interactions(customer_id: String) =>
    interactions <- N<CustomerProductInteraction>::WHERE(_::{customer_id}::EQ(customer_id))
    RETURN interactions

// Get customer service interactions
QUERY get_customer_service_interactions(customer_id: String) =>
    interactions <- N<CustomerServiceInteraction>::WHERE(_::{customer_id}::EQ(customer_id))
    RETURN interactions

// ============================================================================
// NAVIGATION SYSTEM QUERIES (Multi-Tenant Flexible)
// ============================================================================

// Add business navigation hub with compass data
QUERY add_business_navigation_hub(
    business_id: String,
    navigation_id: String,
    primary_address: String,
    secondary_address: String,
    building_name: String,
    building_type: String,
    latitude: F64,
    longitude: F64,
    what3words_code: String,
    plus_code: String,
    compass_bearing: F64,
    compass_reference: String,
    magnetic_declination: F64,
    building_description: String,
    building_floors: I32,
    business_floor: I32,
    building_color: String,
    building_size: String,
    main_entrance_description: String,
    alternative_entrances: String,
    entrance_restrictions: String,
    wheelchair_accessible: Boolean,
    elevator_available: Boolean,
    stairs_required: Boolean,
    accessibility_notes: String,
    parking_available: Boolean,
    parking_description: String,
    public_transport_notes: String,
    direction_varies_by_hours: Boolean,
    after_hours_instructions: String,
    created_at: I64,
    updated_at: I64,
    last_verified_at: I64,
    verification_source: String,
    navigation_summary: String,
    embedding: [F64],
    embedding_model: String
) =>
    nav_hub <- AddN<BusinessNavigationHub>({
        business_id: business_id,
        navigation_id: navigation_id,
        primary_address: primary_address,
        secondary_address: secondary_address,
        building_name: building_name,
        building_type: building_type,
        latitude: latitude,
        longitude: longitude,
        what3words_code: what3words_code,
        plus_code: plus_code,
        compass_bearing: compass_bearing,
        compass_reference: compass_reference,
        magnetic_declination: magnetic_declination,
        building_description: building_description,
        building_floors: building_floors,
        business_floor: business_floor,
        building_color: building_color,
        building_size: building_size,
        main_entrance_description: main_entrance_description,
        alternative_entrances: alternative_entrances,
        entrance_restrictions: entrance_restrictions,
        wheelchair_accessible: wheelchair_accessible,
        elevator_available: elevator_available,
        stairs_required: stairs_required,
        accessibility_notes: accessibility_notes,
        parking_available: parking_available,
        parking_description: parking_description,
        public_transport_notes: public_transport_notes,
        direction_varies_by_hours: direction_varies_by_hours,
        after_hours_instructions: after_hours_instructions,
        created_at: created_at,
        updated_at: updated_at,
        last_verified_at: last_verified_at,
        verification_source: verification_source,
        navigation_summary: navigation_summary
    })
    embedding_node <- AddV<BusinessNavigationEmbedding>(embedding, {
        composite_embedding_text: navigation_summary,
        location_context: primary_address,
        building_context: building_description,
        access_context: main_entrance_description,
        landmark_context: secondary_address,
        transport_context: public_transport_notes,
        parking_context: parking_description,
        accessibility_context: accessibility_notes,
        compass_context: compass_reference,
        visibility_context: building_name,
        timing_context: after_hours_instructions,
        navigation_summary: navigation_summary,
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "v1.0"
    })
    embed_edge <- AddE<HasNavigationEmbedding>({
        created_at: created_at
    })::From(nav_hub)::To(embedding_node)
    business <- N<Business>::WHERE(_::{business_id}::EQ(business_id))
    nav_edge <- AddE<HasNavigation>({
        is_primary: true,
        created_at: created_at
    })::From(business)::To(nav_hub)
    RETURN nav_hub

// Add navigation waypoint with compass data
QUERY add_navigation_waypoint(
    waypoint_id: String,
    navigation_id: String,
    waypoint_name: String,
    waypoint_type: String,
    waypoint_category: String,
    description: String,
    visual_cues: String,
    audio_cues: String,
    relative_position: String,
    distance_from_main: I32,
    floor_level: I32,
    compass_direction: String,
    compass_bearing: F64,
    compass_distance: F64,
    business_specific_notes: String,
    accessibility_info: String,
    seasonal_availability: String,
    time_restrictions: String,
    weather_dependent: Boolean,
    created_at: I64,
    is_active: Boolean,
    priority_level: I32,
    embedding: [F64],
    embedding_model: String
) =>
    waypoint <- AddN<NavigationWaypoint>({
        waypoint_id: waypoint_id,
        navigation_id: navigation_id,
        waypoint_name: waypoint_name,
        waypoint_type: waypoint_type,
        waypoint_category: waypoint_category,
        description: description,
        visual_cues: visual_cues,
        audio_cues: audio_cues,
        relative_position: relative_position,
        distance_from_main: distance_from_main,
        floor_level: floor_level,
        compass_direction: compass_direction,
        compass_bearing: compass_bearing,
        compass_distance: compass_distance,
        business_specific_notes: business_specific_notes,
        accessibility_info: accessibility_info,
        seasonal_availability: seasonal_availability,
        time_restrictions: time_restrictions,
        weather_dependent: weather_dependent,
        created_at: created_at,
        is_active: is_active,
        priority_level: priority_level
    })
    embedding_node <- AddV<NavigationWaypointEmbedding>(embedding, {
        composite_embedding_text: description,
        waypoint_type_context: waypoint_type,
        location_context: relative_position,
        visibility_context: visual_cues,
        accessibility_context: accessibility_info,
        compass_context: compass_direction,
        distance_context: relative_position,
        recognition_context: waypoint_name,
        seasonal_context: seasonal_availability,
        traffic_context: business_specific_notes,
        safety_context: "standard",
        waypoint_description: description,
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "v1.0"
    })
    embed_edge <- AddE<HasWaypointEmbedding>({
        created_at: created_at
    })::From(waypoint)::To(embedding_node)
    nav_hub <- N<BusinessNavigationHub>::WHERE(_::{navigation_id}::EQ(navigation_id))
    waypoint_edge <- AddE<HasWaypoint>({
        waypoint_order: 0,
        is_critical: false,
        created_at: created_at
    })::From(nav_hub)::To(waypoint)
    RETURN waypoint

// Add direction path with compass data
QUERY add_direction_path(
    path_id: String,
    navigation_id: String,
    path_name: String,
    path_type: String,
    transport_mode: String,
    estimated_duration_minutes: I32,
    difficulty_level: String,
    distance_meters: I32,
    starting_compass_bearing: F64,
    ending_compass_bearing: F64,
    path_compass_waypoints: String,
    suitable_for_mobility_aids: Boolean,
    suitable_for_children: Boolean,
    suitable_in_rain: Boolean,
    suitable_at_night: Boolean,
    requires_appointment: Boolean,
    requires_security_clearance: Boolean,
    visitor_badge_required: Boolean,
    step_by_step_instructions: String,
    quick_summary: String,
    created_at: I64,
    is_recommended: Boolean,
    is_active: Boolean,
    last_used_feedback: String,
    embedding: [F64],
    embedding_model: String
) =>
    path <- AddN<DirectionPath>({
        path_id: path_id,
        navigation_id: navigation_id,
        path_name: path_name,
        path_type: path_type,
        transport_mode: transport_mode,
        estimated_duration_minutes: estimated_duration_minutes,
        difficulty_level: difficulty_level,
        distance_meters: distance_meters,
        starting_compass_bearing: starting_compass_bearing,
        ending_compass_bearing: ending_compass_bearing,
        path_compass_waypoints: path_compass_waypoints,
        suitable_for_mobility_aids: suitable_for_mobility_aids,
        suitable_for_children: suitable_for_children,
        suitable_in_rain: suitable_in_rain,
        suitable_at_night: suitable_at_night,
        requires_appointment: requires_appointment,
        requires_security_clearance: requires_security_clearance,
        visitor_badge_required: visitor_badge_required,
        step_by_step_instructions: step_by_step_instructions,
        quick_summary: quick_summary,
        created_at: created_at,
        is_recommended: is_recommended,
        is_active: is_active,
        last_used_feedback: last_used_feedback
    })
    embedding_node <- AddV<DirectionPathEmbedding>(embedding, {
        composite_embedding_text: step_by_step_instructions,
        transport_context: transport_mode,
        difficulty_context: difficulty_level,
        duration_context: quick_summary,
        accessibility_context: "accessible",
        weather_context: "weather_flexible",
        safety_context: "safe",
        time_context: "standard",
        traffic_context: "standard",
        landmark_context: step_by_step_instructions,
        compass_context: path_compass_waypoints,
        convenience_context: path_name,
        requirements_context: "standard",
        path_instructions: step_by_step_instructions,
        embedding_model: embedding_model,
        embedding_date: created_at,
        embedding_version: "v1.0"
    })
    embed_edge <- AddE<HasPathEmbedding>({
        created_at: created_at
    })::From(path)::To(embedding_node)
    nav_hub <- N<BusinessNavigationHub>::WHERE(_::{navigation_id}::EQ(navigation_id))
    path_edge <- AddE<HasPath>({
        is_default: is_recommended,
        context_tags: "[]",
        created_at: created_at
    })::From(nav_hub)::To(path)
    RETURN path

// Search navigation hubs by location description
QUERY search_navigation_hubs(query_embedding: [F64], limit: I64) =>
    embeddings <- SearchV<BusinessNavigationEmbedding>(query_embedding, limit)
    nav_hubs <- embeddings::In<HasNavigationEmbedding>
    RETURN nav_hubs

// Search waypoints by description
QUERY search_navigation_waypoints(query_embedding: [F64], limit: I64) =>
    embeddings <- SearchV<NavigationWaypointEmbedding>(query_embedding, limit)
    waypoints <- embeddings::In<HasWaypointEmbedding>
    RETURN waypoints

// Search direction paths by instructions
QUERY search_direction_paths(query_embedding: [F64], limit: I64) =>
    embeddings <- SearchV<DirectionPathEmbedding>(query_embedding, limit)
    paths <- embeddings::In<HasPathEmbedding>
    RETURN paths

// Get business navigation hub
QUERY get_business_navigation_hub(business_id: String) =>
    nav_hub <- N<BusinessNavigationHub>::WHERE(_::{business_id}::EQ(business_id))
    RETURN nav_hub

// Get navigation waypoints for a business
QUERY get_navigation_waypoints(navigation_id: String) =>
    waypoints <- N<NavigationWaypoint>::WHERE(_::{navigation_id}::EQ(navigation_id))
    RETURN waypoints

// Get direction paths for a business
QUERY get_direction_paths(navigation_id: String) =>
    paths <- N<DirectionPath>::WHERE(_::{navigation_id}::EQ(navigation_id))
    RETURN paths

// Get recommended paths for a business
QUERY get_recommended_paths(navigation_id: String) =>
    paths <- N<DirectionPath>::WHERE(
        AND(
            _::{navigation_id}::EQ(navigation_id),
            _::{is_recommended}::EQ(true),
            _::{is_active}::EQ(true)
        )
    )
    RETURN paths

// Get accessible navigation options
QUERY get_accessible_navigation(navigation_id: String) =>
    nav_hub <- N<BusinessNavigationHub>::WHERE(
        AND(
            _::{navigation_id}::EQ(navigation_id),
            _::{wheelchair_accessible}::EQ(true)
        )
    )
    waypoints <- N<NavigationWaypoint>::WHERE(
        AND(
            _::{navigation_id}::EQ(navigation_id),
            _::{is_active}::EQ(true)
        )
    )
    paths <- N<DirectionPath>::WHERE(
        AND(
            _::{navigation_id}::EQ(navigation_id),
            _::{suitable_for_mobility_aids}::EQ(true),
            _::{is_active}::EQ(true)
        )
    )
    RETURN nav_hub, waypoints, paths


// ============================================================================
// SEMANTIC SEARCH QUERIES (HelixDB Embedding Mode)
// ============================================================================
// These queries use Embed() function for HelixDB to generate embeddings
// Requires embedding_model configured in config.hx.json

// Business Services Semantic Search
QUERY search_business_services_semantic(query_text: String, k: I64) =>
    results <- SearchV<BusinessServiceEmbedding>(Embed(query_text), k)
    services <- results::In<HasServiceEmbedding>
    RETURN services

// Business Locations Semantic Search  
QUERY search_business_locations_semantic(query_text: String, k: I64) =>
    results <- SearchV<BusinessLocationEmbedding>(Embed(query_text), k)
    locations <- results::In<HasLocationEmbedding>
    RETURN locations

// Business Hours Semantic Search
QUERY search_business_hours_semantic(query_text: String, k: I64) =>
    results <- SearchV<BusinessHoursEmbedding>(Embed(query_text), k)
    hours <- results::In<HasHoursEmbedding>
    RETURN hours

// Business Social Semantic Search
QUERY search_business_social_semantic(query_text: String, k: I64) =>
    results <- SearchV<BusinessSocialEmbedding>(Embed(query_text), k)
    social <- results::In<HasSocialEmbedding>
    RETURN social

// Business Policies Semantic Search
QUERY search_business_policies_semantic(query_text: String, k: I64) =>
    results <- SearchV<BusinessPolicyEmbedding>(Embed(query_text), k)
    policies <- results::In<HasPolicyEmbedding>
    RETURN policies

// Business Events Semantic Search
QUERY search_business_events_semantic(query_text: String, k: I64) =>
    results <- SearchV<BusinessEventEmbedding>(Embed(query_text), k)
    events <- results::In<HasEventEmbedding>
    RETURN events

// Customer Behaviors Semantic Search
QUERY search_customer_behaviors_semantic(query_text: String, k: I64) =>
    results <- SearchV<CustomerBehaviorEmbedding>(Embed(query_text), k)
    behaviors <- results::In<HasBehaviorEmbedding>
    RETURN behaviors

// Customer Preferences Semantic Search
QUERY search_customer_preferences_semantic(query_text: String, k: I64) =>
    results <- SearchV<CustomerPreferenceEmbedding>(Embed(query_text), k)
    preferences <- results::In<HasPreferenceEmbedding>
    RETURN preferences

// Customer Desires Semantic Search
QUERY search_customer_desires_semantic(query_text: String, k: I64) =>
    results <- SearchV<CustomerDesireEmbedding>(Embed(query_text), k)
    desires <- results::In<HasDesireEmbedding>
    RETURN desires

// Customer Rules Semantic Search
QUERY search_customer_rules_semantic(query_text: String, k: I64) =>
    results <- SearchV<CustomerRuleEmbedding>(Embed(query_text), k)
    rules <- results::In<HasRuleEmbedding>
    RETURN rules

// Customer Feedback Semantic Search
QUERY search_customer_feedback_semantic(query_text: String, k: I64) =>
    results <- SearchV<CustomerFeedbackEmbedding>(Embed(query_text), k)
    feedback <- results::In<HasFeedbackEmbedding>
    RETURN feedback

// Customer Product Interactions Semantic Search
QUERY search_customer_product_interactions_semantic(query_text: String, k: I64) =>
    results <- SearchV<CustomerProductInteractionEmbedding>(Embed(query_text), k)
    interactions <- results::In<HasProductInteractionEmbedding>
    RETURN interactions

// Customer Service Interactions Semantic Search
QUERY search_customer_service_interactions_semantic(query_text: String, k: I64) =>
    results <- SearchV<CustomerServiceInteractionEmbedding>(Embed(query_text), k)
    interactions <- results::In<HasServiceInteractionEmbedding>
    RETURN interactions

// Navigation Hubs Semantic Search
QUERY search_navigation_hubs_semantic(query_text: String, k: I64) =>
    results <- SearchV<BusinessNavigationEmbedding>(Embed(query_text), k)
    hubs <- results::In<HasNavigationEmbedding>
    RETURN hubs

// Navigation Waypoints Semantic Search
QUERY search_waypoints_semantic(query_text: String, k: I64) =>
    results <- SearchV<NavigationWaypointEmbedding>(Embed(query_text), k)
    waypoints <- results::In<HasWaypointEmbedding>
    RETURN waypoints

// Direction Paths Semantic Search
QUERY search_direction_paths_semantic(query_text: String, k: I64) =>
    results <- SearchV<DirectionPathEmbedding>(Embed(query_text), k)
    paths <- results::In<HasPathEmbedding>
    RETURN paths


// ============================================================================
// BM25 TEXT SEARCH QUERIES (No Embeddings Required!)
// ============================================================================
// These queries use BM25 algorithm for keyword/phrase search
// NO embedding model needed - works with any number of nodes
// Automatically indexes all text fields in nodes
// Perfect for exact keyword matching and fast search

// Business Products BM25 Search
QUERY search_business_products_bm25(query_text: String, k: I64) =>
    products <- SearchBM25<BusinessProductMemory>(query_text, k)
    RETURN products

// Business Services BM25 Search
QUERY search_business_services_bm25(query_text: String, k: I64) =>
    services <- SearchBM25<BusinessServiceMemory>(query_text, k)
    RETURN services

// Business Locations BM25 Search
QUERY search_business_locations_bm25(query_text: String, k: I64) =>
    locations <- SearchBM25<BusinessLocationMemory>(query_text, k)
    RETURN locations

// Business Hours BM25 Search
QUERY search_business_hours_bm25(query_text: String, k: I64) =>
    hours <- SearchBM25<BusinessHoursMemory>(query_text, k)
    RETURN hours

// Business Social BM25 Search
QUERY search_business_social_bm25(query_text: String, k: I64) =>
    social <- SearchBM25<BusinessSocialMemory>(query_text, k)
    RETURN social

// Business Policies BM25 Search
QUERY search_business_policies_bm25(query_text: String, k: I64) =>
    policies <- SearchBM25<BusinessPolicyMemory>(query_text, k)
    RETURN policies

// Business Events BM25 Search
QUERY search_business_events_bm25(query_text: String, k: I64) =>
    events <- SearchBM25<BusinessEventMemory>(query_text, k)
    RETURN events

// Customer Behaviors BM25 Search
QUERY search_customer_behaviors_bm25(query_text: String, k: I64) =>
    behaviors <- SearchBM25<CustomerBehaviorMemory>(query_text, k)
    RETURN behaviors

// Customer Preferences BM25 Search
QUERY search_customer_preferences_bm25(query_text: String, k: I64) =>
    preferences <- SearchBM25<CustomerPreferenceMemory>(query_text, k)
    RETURN preferences

// Customer Desires BM25 Search
QUERY search_customer_desires_bm25(query_text: String, k: I64) =>
    desires <- SearchBM25<CustomerDesireMemory>(query_text, k)
    RETURN desires

// Customer Rules BM25 Search
QUERY search_customer_rules_bm25(query_text: String, k: I64) =>
    rules <- SearchBM25<CustomerRuleMemory>(query_text, k)
    RETURN rules

// Customer Feedback BM25 Search
QUERY search_customer_feedback_bm25(query_text: String, k: I64) =>
    feedback <- SearchBM25<CustomerFeedbackMemory>(query_text, k)
    RETURN feedback

// Customer Communication BM25 Search
QUERY search_customer_communication_bm25(query_text: String, k: I64) =>
    communication <- SearchBM25<CustomerBusinessCommunication>(query_text, k)
    RETURN communication

// Customer Product Interactions BM25 Search
QUERY search_customer_product_interactions_bm25(query_text: String, k: I64) =>
    interactions <- SearchBM25<CustomerProductInteraction>(query_text, k)
    RETURN interactions

// Customer Service Interactions BM25 Search
QUERY search_customer_service_interactions_bm25(query_text: String, k: I64) =>
    interactions <- SearchBM25<CustomerServiceInteraction>(query_text, k)
    RETURN interactions

// Navigation Hubs BM25 Search
QUERY search_navigation_hubs_bm25(query_text: String, k: I64) =>
    hubs <- SearchBM25<BusinessNavigationHub>(query_text, k)
    RETURN hubs

// Navigation Waypoints BM25 Search
QUERY search_waypoints_bm25(query_text: String, k: I64) =>
    waypoints <- SearchBM25<NavigationWaypoint>(query_text, k)
    RETURN waypoints

// Direction Paths BM25 Search
QUERY search_direction_paths_bm25(query_text: String, k: I64) =>
    paths <- SearchBM25<DirectionPath>(query_text, k)
    RETURN paths


// ============================================================================
// VECTOR REGENERATION UPDATE QUERIES
// ============================================================================

// Test File: Update Business Product Memory
// Status: ✅ VALIDATED - Passes helix check

QUERY update_business_product_memory(
    business_id: String,
    product_id: String,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<BusinessProductMemory>::WHERE(_::{business_id}::EQ(business_id))::WHERE(_::{product_id}::EQ(product_id))
    updated <- memory::UPDATE({text_description: composite_text, updated_at: timestamp})
    DROP memory::Out<HasProductEmbedding>
    DROP memory::OutE<HasProductEmbedding>
    vec <- AddV<BusinessProductEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasProductEmbedding>({created_at: timestamp})::From(updated)::To(vec)
    RETURN updated

// Test File: Update Business Service Memory
// Status: Ready for validation

QUERY update_business_service_memory(
    business_id: String,
    service_id: String,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<BusinessServiceMemory>::WHERE(_::{business_id}::EQ(business_id))::WHERE(_::{service_id}::EQ(service_id))
    updated <- memory::UPDATE({text_description: composite_text, updated_at: timestamp})
    DROP memory::Out<HasServiceEmbedding>
    DROP memory::OutE<HasServiceEmbedding>
    vec <- AddV<BusinessServiceEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasServiceEmbedding>({created_at: timestamp})::From(updated)::To(vec)
    RETURN updated

// Test File: Update Business Location Memory
// Status: Ready for validation

QUERY update_business_location_memory(
    business_id: String,
    location_id: String,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<BusinessLocationMemory>::WHERE(_::{business_id}::EQ(business_id))::WHERE(_::{location_id}::EQ(location_id))
    updated <- memory::UPDATE({text_description: composite_text, updated_at: timestamp})
    DROP memory::Out<HasLocationEmbedding>
    DROP memory::OutE<HasLocationEmbedding>
    vec <- AddV<BusinessLocationEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasLocationEmbedding>({created_at: timestamp})::From(updated)::To(vec)
    RETURN updated

// Test File: Update Business Hours Memory
// Status: Ready for validation

QUERY update_business_hours_memory(
    business_id: String,
    hours_id: String,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<BusinessHoursMemory>::WHERE(_::{business_id}::EQ(business_id))::WHERE(_::{hours_id}::EQ(hours_id))
    updated <- memory::UPDATE({text_description: composite_text, updated_at: timestamp})
    DROP memory::Out<HasHoursEmbedding>
    DROP memory::OutE<HasHoursEmbedding>
    vec <- AddV<BusinessHoursEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasHoursEmbedding>({created_at: timestamp})::From(updated)::To(vec)
    RETURN updated

// Test File: Update Business Social Memory
// Status: Ready for validation

QUERY update_business_social_memory(
    business_id: String,
    social_id: String,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<BusinessSocialMemory>::WHERE(_::{business_id}::EQ(business_id))::WHERE(_::{social_id}::EQ(social_id))
    updated <- memory::UPDATE({text_description: composite_text, updated_at: timestamp})
    DROP memory::Out<HasSocialEmbedding>
    DROP memory::OutE<HasSocialEmbedding>
    vec <- AddV<BusinessSocialEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasSocialEmbedding>({created_at: timestamp})::From(updated)::To(vec)
    RETURN updated

// Test File: Update Business Policy Memory
// Status: Ready for validation

QUERY update_business_policy_memory(
    business_id: String,
    policy_id: String,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<BusinessPolicyMemory>::WHERE(_::{business_id}::EQ(business_id))::WHERE(_::{policy_id}::EQ(policy_id))
    updated <- memory::UPDATE({text_description: composite_text, updated_at: timestamp})
    DROP memory::Out<HasPolicyEmbedding>
    DROP memory::OutE<HasPolicyEmbedding>
    vec <- AddV<BusinessPolicyEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasPolicyEmbedding>({created_at: timestamp})::From(updated)::To(vec)
    RETURN updated

// Test File: Update Business Event Memory
// Status: Ready for validation

QUERY update_business_event_memory(
    business_id: String,
    event_id: String,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<BusinessEventMemory>::WHERE(_::{business_id}::EQ(business_id))::WHERE(_::{event_id}::EQ(event_id))
    updated <- memory::UPDATE({text_description: composite_text, updated_at: timestamp})
    DROP memory::Out<HasEventEmbedding>
    DROP memory::OutE<HasEventEmbedding>
    vec <- AddV<BusinessEventEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasEventEmbedding>({created_at: timestamp})::From(updated)::To(vec)
    RETURN updated

// UNCOMMENTED FOR TESTING - update_business_information_memory
QUERY update_business_information_memory(
    business_id: String,
    info_id: String,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<BusinessInformationMemory>({info_id: info_id})
    updated <- memory::UPDATE({text_description: composite_text, updated_at: timestamp})
    DROP memory::Out<HasInformationEmbedding>
    DROP memory::OutE<HasInformationEmbedding>
    vec <- AddV<BusinessInformationEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasInformationEmbedding>({created_at: timestamp})::From(updated)::To(vec)
    RETURN updated

// Test File: Update Customer Preference Memory
// Status: Ready for validation

QUERY update_customer_preference_memory(
    memory_id: ID,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<CustomerPreferenceMemory>(memory_id)::UPDATE({text_description: composite_text})
    DROP N<CustomerPreferenceMemory>(memory_id)::Out<HasPreferenceEmbedding>
    DROP N<CustomerPreferenceMemory>(memory_id)::OutE<HasPreferenceEmbedding>
    vec <- AddV<CustomerPreferenceEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasPreferenceEmbedding>({created_at: timestamp})::From(memory)::To(vec)
    RETURN memory

// Test File: Update Customer Behavior Memory
// Status: Ready for validation

QUERY update_customer_behavior_memory(
    memory_id: ID,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<CustomerBehaviorMemory>(memory_id)::UPDATE({text_description: composite_text})
    DROP N<CustomerBehaviorMemory>(memory_id)::Out<HasBehaviorEmbedding>
    DROP N<CustomerBehaviorMemory>(memory_id)::OutE<HasBehaviorEmbedding>
    vec <- AddV<CustomerBehaviorEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasBehaviorEmbedding>({created_at: timestamp})::From(memory)::To(vec)
    RETURN memory

// Test File: Update Customer Desire Memory
// Status: Ready for validation

QUERY update_customer_desire_memory(
    memory_id: ID,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<CustomerDesireMemory>(memory_id)::UPDATE({text_description: composite_text})
    DROP N<CustomerDesireMemory>(memory_id)::Out<HasDesireEmbedding>
    DROP N<CustomerDesireMemory>(memory_id)::OutE<HasDesireEmbedding>
    vec <- AddV<CustomerDesireEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasDesireEmbedding>({created_at: timestamp})::From(memory)::To(vec)
    RETURN memory

// Test File: Update Customer Rule Memory
// Status: Ready for validation

QUERY update_customer_rule_memory(
    memory_id: ID,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<CustomerRuleMemory>(memory_id)::UPDATE({text_description: composite_text})
    DROP N<CustomerRuleMemory>(memory_id)::Out<HasRuleEmbedding>
    DROP N<CustomerRuleMemory>(memory_id)::OutE<HasRuleEmbedding>
    vec <- AddV<CustomerRuleEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasRuleEmbedding>({created_at: timestamp})::From(memory)::To(vec)
    RETURN memory

// Test File: Update Customer Feedback Memory
// Status: Ready for validation

QUERY update_customer_feedback_memory(
    memory_id: ID,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<CustomerFeedbackMemory>(memory_id)::UPDATE({text_description: composite_text})
    DROP N<CustomerFeedbackMemory>(memory_id)::Out<HasFeedbackEmbedding>
    DROP N<CustomerFeedbackMemory>(memory_id)::OutE<HasFeedbackEmbedding>
    vec <- AddV<CustomerFeedbackEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasFeedbackEmbedding>({created_at: timestamp})::From(memory)::To(vec)
    RETURN memory

// Test File: Update Customer Communication Memory
// Status: Ready for validation

QUERY update_customer_communication_memory(
    memory_id: ID,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<CustomerBusinessCommunication>(memory_id)::UPDATE({text_interaction: composite_text})
    DROP N<CustomerBusinessCommunication>(memory_id)::Out<HasCommunicationEmbedding>
    DROP N<CustomerBusinessCommunication>(memory_id)::OutE<HasCommunicationEmbedding>
    vec <- AddV<CustomerCommunicationEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasCommunicationEmbedding>({created_at: timestamp})::From(memory)::To(vec)
    RETURN memory

// ============================================================================
// MISSING VECTOR-AWARE UPDATE QUERIES
// ============================================================================

// Update Customer Product Interaction Memory with Vector Regeneration
QUERY update_customer_product_interaction_memory(
    memory_id: ID,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<CustomerProductInteraction>(memory_id)::UPDATE({text_reason: composite_text})
    DROP N<CustomerProductInteraction>(memory_id)::Out<HasProductInteractionEmbedding>
    DROP N<CustomerProductInteraction>(memory_id)::OutE<HasProductInteractionEmbedding>
    vec <- AddV<CustomerProductInteractionEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasProductInteractionEmbedding>({created_at: timestamp})::From(memory)::To(vec)
    RETURN memory

// Update Customer Service Interaction Memory with Vector Regeneration
QUERY update_customer_service_interaction_memory(
    memory_id: ID,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<CustomerServiceInteraction>(memory_id)::UPDATE({text_feedback: composite_text})
    DROP N<CustomerServiceInteraction>(memory_id)::Out<HasServiceInteractionEmbedding>
    DROP N<CustomerServiceInteraction>(memory_id)::OutE<HasServiceInteractionEmbedding>
    vec <- AddV<CustomerServiceInteractionEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasServiceInteractionEmbedding>({created_at: timestamp})::From(memory)::To(vec)
    RETURN memory

// Update Business Navigation Hub Memory with Vector Regeneration
QUERY update_business_navigation_hub_memory(
    memory_id: ID,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<BusinessNavigationHub>(memory_id)::UPDATE({navigation_summary: composite_text})
    DROP N<BusinessNavigationHub>(memory_id)::Out<HasNavigationEmbedding>
    DROP N<BusinessNavigationHub>(memory_id)::OutE<HasNavigationEmbedding>
    vec <- AddV<BusinessNavigationEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasNavigationEmbedding>({created_at: timestamp})::From(memory)::To(vec)
    RETURN memory

// Update Navigation Waypoint Memory with Vector Regeneration
QUERY update_navigation_waypoint_memory(
    memory_id: ID,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<NavigationWaypoint>(memory_id)::UPDATE({business_specific_notes: composite_text})
    DROP N<NavigationWaypoint>(memory_id)::Out<HasWaypointEmbedding>
    DROP N<NavigationWaypoint>(memory_id)::OutE<HasWaypointEmbedding>
    vec <- AddV<NavigationWaypointEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasWaypointEmbedding>({created_at: timestamp})::From(memory)::To(vec)
    RETURN memory

// Update Direction Path Memory with Vector Regeneration
QUERY update_direction_path_memory(
    memory_id: ID,
    composite_text: String,
    new_embedding: [F64],
    timestamp: I64
) =>
    memory <- N<DirectionPath>(memory_id)::UPDATE({step_by_step_instructions: composite_text})
    DROP N<DirectionPath>(memory_id)::Out<HasPathEmbedding>
    DROP N<DirectionPath>(memory_id)::OutE<HasPathEmbedding>
    vec <- AddV<DirectionPathEmbedding>(new_embedding, {composite_embedding_text: composite_text})
    edge <- AddE<HasPathEmbedding>({created_at: timestamp})::From(memory)::To(vec)
    RETURN memory


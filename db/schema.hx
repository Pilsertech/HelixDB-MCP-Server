// HelixDB Schema for AI Memory Layer
// Business and Customer Memory Nodes with Vector Embeddings

// ============================================================================
// BUSINESS/ORGANIZATION ENTITIES (For Isolation & Collaboration)
// ============================================================================

// Business/Organization Node - Represents a tenant (business, service provider, institution)
N::Business {
    business_id: String,                        // Unique business identifier (REQUIRED)
    business_name: String,                      // Business name (REQUIRED)
    business_type: String DEFAULT "",           // Type - optional, empty if not specified
    status: String DEFAULT "",                  // Status - optional, empty if not specified
    allow_collaboration: Boolean DEFAULT false, // Collaboration setting - optional, defaults to false
    created_at: I64 DEFAULT NOW,               // Auto-generated timestamp
    metadata: String DEFAULT "{}"               // JSON for additional fields - optional
}

// Customer/User Node - Represents a person interacting with businesses
N::Customer {
    customer_id: String,                // Unique customer identifier (REQUIRED)
    customer_name: String DEFAULT "",   // Customer name - optional for privacy
    phone: String DEFAULT "",           // Phone number - optional
    email: String DEFAULT "",           // Email - optional
    language: String DEFAULT "",        // Preferred language - optional, empty if not specified
    created_at: I64 DEFAULT NOW,       // Auto-generated timestamp
    metadata: String DEFAULT "{}"       // JSON for additional fields - optional
}

// ============================================================================
// BUSINESS MEMORY NODES
// Each business aspect (products, services, location, etc.) gets its own node
// ============================================================================

// Product Memory Node - Each product is a separate memory node  
N::BusinessProductMemory {
    business_id: String,                    // Business identifier (REQUIRED)
    product_id: String,                     // Unique product identifier (REQUIRED)
    product_name: String,                   // Product name (REQUIRED)
    product_category: String DEFAULT "",    // Category (electronics, clothing, etc.) - optional
    price: F64 DEFAULT 0.0,                // Product price - optional, defaults to 0.0
    currency: String DEFAULT "",            // Currency code - optional, empty if not specified
    availability: String DEFAULT "",        // Availability status - optional, empty if not specified
    description: String DEFAULT "",         // Detailed product description - optional
    features: [String],                     // List of product features (REQUIRED - use empty array if none)
    specifications: String DEFAULT "{}",    // Technical specs as JSON string - optional
    tags: [String],                         // Search tags (REQUIRED - use empty array if none)
    seo_keywords: [String],                 // SEO keywords for search optimization (REQUIRED - use empty array if none)
    competitor_analysis: String DEFAULT "", // Analysis of competitor products - optional
    seasonal_trends: String DEFAULT "{}",   // Seasonal demand trends as JSON string - optional
    created_at: I64 DEFAULT NOW,           // Timestamp - auto-generated
    updated_at: I64 DEFAULT NOW,           // Last update timestamp - auto-generated
    text_description: String DEFAULT ""     // RICH Natural language description for embeddings - optional
                                           // Format: "Product: {name}. Category: {category}. Price: {price} {currency}. Description: {description}. Features: {features}. Tags: {tags}. Availability: {availability}"
}

// Enhanced Vector embedding for Business Product Memory
V::BusinessProductEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text combining all searchable fields
    product_name: String DEFAULT "",               // Product name for direct matching
    category_context: String DEFAULT "",           // "electronics > mobile > smartphones android iOS"
    price_context: String DEFAULT "",              // "budget under 500 dollars affordable mid-range premium luxury"
    availability_context: String DEFAULT "",       // "in stock available immediately out of stock discontinued"
    feature_context: String DEFAULT "",            // "camera wireless charging battery dual sim face unlock"
    brand_context: String DEFAULT "",              // "apple samsung google premium quality reliable"
    customer_context: String DEFAULT "",           // "students professionals families young adults seniors"
    seasonal_context: String DEFAULT "",           // "holiday gift back to school summer winter sale"
    specification_summary: String DEFAULT "",      // "128GB storage 6GB RAM 5000mAh battery 48MP camera"
    use_case_context: String DEFAULT "",           // "gaming photography business work entertainment"
    competitor_context: String DEFAULT "",         // "similar to iPhone Galaxy alternative cheaper better"
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Link product node to its embedding vector
E::HasProductEmbedding {
    From: BusinessProductMemory,
    To: BusinessProductEmbedding,
    Properties: {
        created_at: I64
    }
}

// Service Memory Node - Each service is a separate memory node
N::BusinessServiceMemory {
    business_id: String,                    // Business identifier (REQUIRED)
    service_id: String,                     // Unique service identifier (REQUIRED)
    service_name: String,                   // Service name (REQUIRED)
    service_category: String DEFAULT "",    // Category (consulting, repair, etc.) - optional
    price: F64 DEFAULT 0.0,                // Service price - optional, defaults to 0.0
    currency: String DEFAULT "",            // Currency code - optional, empty if not specified
    duration_minutes: I32 DEFAULT 60,       // Service duration in minutes - optional, defaults to 1 hour
    availability: String DEFAULT "",        // Availability status - optional, empty if not specified
    description: String DEFAULT "",         // Detailed service description - optional
    requirements: [String],                 // Service requirements (REQUIRED - use empty array if none)
    deliverables: [String],                 // What customer gets (REQUIRED - use empty array if none)  
    tags: [String],                         // Search tags (REQUIRED - use empty array if none)
    created_at: I64 DEFAULT NOW,           // Timestamp - auto-generated
    updated_at: I64 DEFAULT NOW,           // Last update timestamp - auto-generated
    text_description: String DEFAULT ""     // Natural language description for embeddings - optional
}

// Enhanced Vector embedding for Business Service Memory
V::BusinessServiceEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text combining all searchable fields
    service_name: String DEFAULT "",               // Service name for direct matching
    category_context: String DEFAULT "",           // "consulting repair maintenance installation support"
    price_context: String DEFAULT "",              // "affordable budget premium hourly flat rate subscription"
    duration_context: String DEFAULT "",           // "quick 30 minutes hour half day full day consultation"
    availability_context: String DEFAULT "",       // "available immediately book ahead appointment walk-in"
    skill_context: String DEFAULT "",              // "expert professional certified experienced beginner"
    target_context: String DEFAULT "",             // "individuals businesses students professionals seniors"
    location_context: String DEFAULT "",           // "on-site remote office home visit workshop"
    urgency_context: String DEFAULT "",            // "emergency same day next day flexible scheduling"
    outcome_context: String DEFAULT "",            // "repair fix solve improve optimize troubleshoot"
    requirement_context: String DEFAULT "",        // "no experience needed technical knowledge required"
    deliverable_context: String DEFAULT "",        // "report certificate warranty guarantee follow-up"
    seasonal_context: String DEFAULT "",           // "year-round seasonal holiday maintenance preparation"
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Link service node to its embedding vector
E::HasServiceEmbedding {
    From: BusinessServiceMemory,
    To: BusinessServiceEmbedding,
    Properties: {
        created_at: I64
    }
}

// Location Memory Node - Business location information
N::BusinessLocationMemory {
    business_id: String,                    // Business identifier (REQUIRED)
    location_id: String,                    // Unique location identifier (REQUIRED)
    location_name: String DEFAULT "",       // Location name - optional
    address: String DEFAULT "",             // Full address - optional
    city: String DEFAULT "",               // City - optional
    state: String DEFAULT "",              // State/Province - optional
    country: String DEFAULT "",            // Country - optional
    postal_code: String DEFAULT "",        // Postal/ZIP code - optional
    latitude: F64 DEFAULT 0.0,             // GPS latitude - optional, defaults to 0.0
    longitude: F64 DEFAULT 0.0,            // GPS longitude - optional, defaults to 0.0
    location_type: String DEFAULT "",      // Location type - optional, empty if not specified
    accessibility: [String],               // Accessibility features (REQUIRED - use empty array if none)
    parking_info: String DEFAULT "",       // Parking information - optional
    created_at: I64 DEFAULT NOW,           // Auto-generated timestamp
    updated_at: I64 DEFAULT NOW,           // Auto-generated timestamp
    text_description: String DEFAULT ""     // Natural language description - optional
}

// Enhanced Vector embedding for Business Location Memory
V::BusinessLocationEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text combining all location data
    location_name: String DEFAULT "",              // Location name for direct matching
    address_context: String DEFAULT "",            // "downtown main street shopping center mall plaza"
    area_context: String DEFAULT "",               // "business district university area residential commercial"
    accessibility_context: String DEFAULT "",      // "wheelchair accessible elevator ramp parking disabled"
    transport_context: String DEFAULT "",          // "public transport bus metro train taxi uber walking"
    parking_context: String DEFAULT "",            // "free parking paid street garage valet available"
    landmark_context: String DEFAULT "",           // "near hospital school mall library post office bank"
    neighborhood_context: String DEFAULT "",       // "safe quiet busy trendy upscale residential commercial"
    convenience_context: String DEFAULT "",        // "easy access main road highway off-street central"
    building_context: String DEFAULT "",           // "modern historic glass brick high-rise ground floor"
    entrance_context: String DEFAULT "",           // "main entrance side door street level elevator required"
    hours_context: String DEFAULT "",              // "24/7 business hours extended weekend limited access"
    security_context: String DEFAULT "",           // "secure gated public private requires badge visitor"
    visitor_context: String DEFAULT "",            // "walk-in appointment only reception desk lobby"
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Link location node to its embedding vector
E::HasLocationEmbedding {
    From: BusinessLocationMemory,
    To: BusinessLocationEmbedding,
    Properties: {
        created_at: I64
    }
}

// Hours Memory Node - Business operating hours
N::BusinessHoursMemory {
    business_id: String,                      // Business identifier (REQUIRED)
    hours_id: String,                         // Unique hours identifier (REQUIRED)
    schedule_type: String DEFAULT "",         // Schedule type - optional, empty if not specified
    monday_open: String DEFAULT "",           // Opening time - optional, empty if not specified
    monday_close: String DEFAULT "",          // Closing time - optional, empty if not specified
    tuesday_open: String DEFAULT "",          // Opening time - optional, empty if not specified
    tuesday_close: String DEFAULT "",         // Closing time - optional, empty if not specified
    wednesday_open: String DEFAULT "",        // Opening time - optional, empty if not specified
    wednesday_close: String DEFAULT "",       // Closing time - optional, empty if not specified
    thursday_open: String DEFAULT "",         // Opening time - optional, empty if not specified
    thursday_close: String DEFAULT "",        // Closing time - optional, empty if not specified
    friday_open: String DEFAULT "",           // Opening time - optional, empty if not specified
    friday_close: String DEFAULT "",          // Closing time - optional, empty if not specified
    saturday_open: String DEFAULT "",         // Opening time - optional, empty if not specified
    saturday_close: String DEFAULT "",        // Closing time - optional, empty if not specified
    sunday_open: String DEFAULT "",           // Opening time - optional, empty if not specified
    sunday_close: String DEFAULT "",          // Closing time - optional, empty if not specified
    timezone: String DEFAULT "",              // Timezone - optional, empty if not specified
    exceptions: String DEFAULT "{}",          // Special dates and hours as JSON - optional
    created_at: I64 DEFAULT NOW,             // Auto-generated timestamp
    updated_at: I64 DEFAULT NOW,             // Auto-generated timestamp
    text_description: String DEFAULT ""       // Natural language description - optional
}

// Enhanced Vector embedding for Business Hours Memory
V::BusinessHoursEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text with schedule patterns
    schedule_pattern: String DEFAULT "",           // "monday to friday weekdays weekend saturday sunday"
    availability_context: String DEFAULT "",       // "early morning late night 24/7 business hours extended"
    day_context: String DEFAULT "",                // "weekdays weekends holidays monday tuesday friday"
    time_context: String DEFAULT "",               // "morning afternoon evening night dawn dusk rush hour"
    convenience_context: String DEFAULT "",        // "convenient flexible limited restricted appointment only"
    access_context: String DEFAULT "",             // "open closed available emergency after hours on-call"
    special_context: String DEFAULT "",            // "holiday hours summer winter seasonal exceptions"
    customer_context: String DEFAULT "",           // "working professionals students families emergency"
    service_context: String DEFAULT "",            // "consultations appointments walk-ins emergency calls"
    timezone_context: String DEFAULT "",           // "eastern pacific mountain central local time"
    flexibility_context: String DEFAULT "",        // "strict flexible by appointment emergency available"
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Link hours node to its embedding vector
E::HasHoursEmbedding {
    From: BusinessHoursMemory,
    To: BusinessHoursEmbedding,
    Properties: {
        created_at: I64
    }
}

// Social Media Memory Node - Business social media presence
N::BusinessSocialMemory {
    business_id: String,                   // Business identifier (REQUIRED)
    social_id: String,                     // Unique social media identifier (REQUIRED)
    platform: String,                      // Platform name (REQUIRED)
    handle: String DEFAULT "",              // @handle or profile URL - optional
    profile_url: String DEFAULT "",         // Full profile URL - optional
    follower_count: I64 DEFAULT 0,         // Number of followers - optional, defaults to 0
    post_count: I64 DEFAULT 0,             // Number of posts - optional, defaults to 0
    description: String DEFAULT "",         // Profile description/bio - optional
    contact_info: String DEFAULT "",        // Contact information shared on social - optional
    last_updated: I64 DEFAULT NOW,         // When social data was last fetched - auto-generated
    created_at: I64 DEFAULT NOW,           // Auto-generated timestamp
    updated_at: I64 DEFAULT NOW,           // Auto-generated timestamp
    text_description: String DEFAULT ""     // Natural language description - optional
}

// Enhanced Vector embedding for Business Social Memory
V::BusinessSocialEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text with social presence
    platform_context: String DEFAULT "",          // "facebook instagram twitter linkedin youtube tiktok"
    engagement_context: String DEFAULT "",         // "active responsive high engagement popular trending"
    audience_context: String DEFAULT "",           // "followers customers community professionals students"
    content_context: String DEFAULT "",            // "updates promotions news tips tutorials behind scenes"
    interaction_context: String DEFAULT "",        // "responds quickly customer service support helpful"
    reach_context: String DEFAULT "",              // "local regional national international global"
    frequency_context: String DEFAULT "",          // "daily weekly regular occasional active inactive"
    tone_context: String DEFAULT "",               // "professional casual friendly formal informative fun"
    purpose_context: String DEFAULT "",            // "marketing customer service news updates community"
    verification_context: String DEFAULT "",       // "verified official authentic legitimate trusted"
    contact_context: String DEFAULT "",            // "direct message email phone website contact form"
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Link social node to its embedding vector
E::HasSocialEmbedding {
    From: BusinessSocialMemory,
    To: BusinessSocialEmbedding,
    Properties: {
        created_at: I64
    }
}

// Policy Memory Node - Business policies and rules
N::BusinessPolicyMemory {
    business_id: String,         // Business identifier
    policy_id: String,          // Unique policy identifier
    policy_type: String DEFAULT "",         // Policy type - optional, empty if not specified
    policy_name: String,                    // Human-readable policy name (REQUIRED)
    content: String DEFAULT "",             // Full policy content - optional
    effective_date: I64 DEFAULT NOW,       // When policy takes effect - optional, defaults to now
    version: String DEFAULT "",            // Policy version - optional, empty if not specified
    is_active: Boolean DEFAULT false,       // Whether policy is currently active - optional, safer to default false
    tags: [String],                         // Search tags (REQUIRED - use empty array if none)
    created_at: I64 DEFAULT NOW,           // Auto-generated timestamp
    updated_at: I64 DEFAULT NOW,           // Auto-generated timestamp
    text_description: String DEFAULT ""     // Natural language description - optional
}

// Enhanced Vector embedding for Business Policy Memory
V::BusinessPolicyEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text with policy details
    policy_name: String DEFAULT "",                // Policy name for direct matching
    type_context: String DEFAULT "",               // "refund return shipping privacy terms warranty"
    scope_context: String DEFAULT "",              // "customers employees partners all users visitors"
    timeline_context: String DEFAULT "",           // "immediate 30 days 90 days lifetime warranty"
    conditions_context: String DEFAULT "",         // "unconditional receipt required original packaging"
    process_context: String DEFAULT "",            // "automatic approval required contact support"
    cost_context: String DEFAULT "",               // "free customer pays shipping restocking fee"
    restriction_context: String DEFAULT "",        // "no restrictions some limits strict requirements"
    communication_context: String DEFAULT "",      // "email notification automatic update required"
    compliance_context: String DEFAULT "",         // "legal required optional recommended industry standard"
    customer_impact_context: String DEFAULT "",    // "customer friendly strict moderate flexible"
    urgency_context: String DEFAULT "",            // "immediate processing standard urgent priority"
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Link policy node to its embedding vector
E::HasPolicyEmbedding {
    From: BusinessPolicyMemory,
    To: BusinessPolicyEmbedding,
    Properties: {
        created_at: I64
    }
}

// Business Event Memory Node - Business events and promotions
N::BusinessEventMemory {
    business_id: String,          // Business identifier
    event_id: String,            // Unique event identifier
    event_name: String,          // Event name
    event_type: String DEFAULT "",             // Event type - optional, empty if not specified
    start_date: I64 DEFAULT NOW,               // Event start timestamp - optional, defaults to now
    end_date: I64 DEFAULT NOW,                 // Event end timestamp - optional, defaults to now
    description: String DEFAULT "",            // Detailed event description - optional
    location: String DEFAULT "",               // Event location or virtual link - optional
    capacity: I32 DEFAULT 0,                   // Maximum capacity - optional, 0 if not specified
    registration_required: Boolean DEFAULT false, // Whether registration is needed - optional, defaults to false
    tags: [String],                            // Search tags (REQUIRED - use empty array if none)
    created_at: I64 DEFAULT NOW,              // Auto-generated timestamp
    updated_at: I64 DEFAULT NOW,              // Auto-generated timestamp
    text_description: String DEFAULT ""        // Natural language description - optional
}

// Enhanced Vector embedding for Business Event Memory
V::BusinessEventEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text with event details
    event_name: String DEFAULT "",                 // Event name for direct matching
    type_context: String DEFAULT "",               // "promotion sale webinar workshop conference training"
    timing_context: String DEFAULT "",             // "morning afternoon evening weekend weekday holiday"
    duration_context: String DEFAULT "",           // "quick hour half day full day multi day series"
    format_context: String DEFAULT "",             // "in-person virtual hybrid online live recorded"
    audience_context: String DEFAULT "",           // "everyone customers members students professionals"
    capacity_context: String DEFAULT "",           // "small intimate large limited unlimited exclusive"
    access_context: String DEFAULT "",             // "free paid registration required walk-in invite only"
    location_context: String DEFAULT "",           // "on-site remote online conference room auditorium"
    urgency_context: String DEFAULT "",            // "register now limited seats last chance ongoing"
    benefit_context: String DEFAULT "",            // "learn network save money exclusive access certification"
    requirement_context: String DEFAULT "",        // "no prerequisites experience required bring laptop"
    seasonal_context: String DEFAULT "",           // "holiday back to school summer winter quarterly"
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Link event node to its embedding vector
E::HasEventEmbedding {
    From: BusinessEventMemory,
    To: BusinessEventEmbedding,
    Properties: {
        created_at: I64
    }
}

// ============================================================================
// CUSTOMER MEMORY NODES
// Customer behaviors, preferences, and memories with natural language text
// ============================================================================

// Customer Behavior Memory Node - Customer actions and behaviors
N::CustomerBehaviorMemory {
    customer_id: String,         // Customer identifier
    behavior_id: String,        // Unique behavior identifier
    behavior_type: String,      // "purchase", "browsing", "interaction", "feedback"
    action: String,             // What the customer did
    context: String DEFAULT "",             // Context of the behavior - optional
    timestamp: I64 DEFAULT NOW,            // When behavior occurred - optional, defaults to now
    channel: String DEFAULT "",            // Channel - optional, empty if not specified
    duration_seconds: I32 DEFAULT 0,       // How long the behavior lasted - optional, defaults to 0
    metadata: String DEFAULT "{}",         // Additional structured data as JSON - optional
    created_at: I64 DEFAULT NOW,          // Auto-generated timestamp
    updated_at: I64 DEFAULT NOW,          // Auto-generated timestamp
    text_description: String DEFAULT ""    // Natural language description - optional
}

// Enhanced Vector embedding for Customer Behavior Memory
V::CustomerBehaviorEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text with behavior patterns
    behavior_type: String DEFAULT "",              // Behavior type for direct matching
    action_context: String DEFAULT "",             // "browsing purchasing comparing searching asking"
    channel_context: String DEFAULT "",            // "website mobile app phone in-store online offline"
    engagement_context: String DEFAULT "",         // "brief extended focused casual intensive detailed"
    intent_context: String DEFAULT "",             // "buying researching comparing price checking learning"
    frequency_context: String DEFAULT "",          // "first time repeat regular occasional frequent rare"
    timing_context: String DEFAULT "",             // "morning afternoon evening weekend weekday rush hour"
    device_context: String DEFAULT "",             // "mobile desktop tablet phone computer smart device"
    location_context: String DEFAULT "",           // "home office store car public transport traveling"
    mood_context: String DEFAULT "",               // "interested excited frustrated confused satisfied"
    interaction_context: String DEFAULT "",        // "self-service asked help needed assistance independent"
    outcome_context: String DEFAULT "",            // "completed abandoned converted satisfied dissatisfied"
    value_context: String DEFAULT "",              // "high value low value budget premium bulk single"
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Link behavior node to its embedding vector
E::HasBehaviorEmbedding {
    From: CustomerBehaviorMemory,
    To: CustomerBehaviorEmbedding,
    Properties: {
        created_at: I64
    }
}

// Customer Preference Memory Node - Customer likes and dislikes
N::CustomerPreferenceMemory {
    customer_id: String,                     // Customer identifier (REQUIRED)
    preference_id: String,                   // Unique preference identifier (REQUIRED)
    preference_type: String DEFAULT "",      // Type - optional, empty if not specified
    category: String DEFAULT "",             // Category - optional, empty if not specified
    subject: String DEFAULT "",              // What the preference is about - optional
    strength: String DEFAULT "",             // Strength - empty to avoid misleading LLM
    is_active: Boolean DEFAULT false,        // Whether preference is still relevant - optional, safer to default false
    evidence_count: I32 DEFAULT 0,          // How many times observed - optional, 0 if not specified
    last_evidence: I64 DEFAULT NOW,         // When preference was last observed - optional, defaults to now
    confidence_score: F64 DEFAULT 0.0,      // Confidence score - optional, 0.0 if not specified
    source_channels: [String],               // Where this preference was observed (REQUIRED - use empty array if none)
    created_at: I64 DEFAULT NOW,            // Auto-generated timestamp
    updated_at: I64 DEFAULT NOW,            // Auto-generated timestamp
    text_description: String DEFAULT ""      // Natural language description - optional
}

// Enhanced Vector embedding for Customer Preference Memory
V::CustomerPreferenceEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text with preference details
    preference_subject: String DEFAULT "",         // Subject for direct matching
    type_context: String DEFAULT "",               // "likes dislikes preferences restrictions requirements"
    category_context: String DEFAULT "",           // "products services communication pricing features"
    strength_context: String DEFAULT "",           // "strong moderate weak absolute flexible conditional"
    reliability_context: String DEFAULT "",        // "consistent inconsistent reliable changing evolving"
    scope_context: String DEFAULT "",              // "specific general broad narrow focused universal"
    trigger_context: String DEFAULT "",            // "always sometimes never seasonal situational mood"
    value_context: String DEFAULT "",              // "price quality convenience speed service support"
    lifestyle_context: String DEFAULT "",          // "busy professional student family health conscious"
    decision_context: String DEFAULT "",           // "research carefully impulse buyer price sensitive quality"
    communication_context: String DEFAULT "",      // "email phone text social media in person chat"
    timing_context: String DEFAULT "",             // "immediate flexible patient urgent deadline driven"
    brand_context: String DEFAULT "",              // "loyal flexible brand conscious generic premium budget"
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Link preference node to its embedding vector
E::HasPreferenceEmbedding {
    From: CustomerPreferenceMemory,
    To: CustomerPreferenceEmbedding,
    Properties: {
        created_at: I64
    }
}

// Customer Desire Memory Node - Customer wants and goals
N::CustomerDesireMemory {
    customer_id: String,                        // Customer identifier
    desire_id: String,                         // Unique desire identifier
    desire_type: String DEFAULT "",            // "immediate", "future", "aspirational" - empty to avoid misleading LLM
    category: String DEFAULT "",               // "products", "services", "experiences" - empty to avoid misleading LLM
    description: String DEFAULT "",            // What the customer wants
    priority: String DEFAULT "",               // "high", "medium", "low" - empty to avoid misleading LLM
    timeframe: String DEFAULT "",              // "immediate", "short_term", "long_term" - empty if not specified
    budget_range: String DEFAULT "",           // Budget consideration if mentioned
    is_active: Boolean DEFAULT false,          // Whether desire is still relevant - safer to default false
    created_at: I64 DEFAULT NOW,              // Timestamp
    updated_at: I64 DEFAULT NOW,              // Last update timestamp
    text_description: String DEFAULT ""       // Natural language description for embeddings
                                              // Example: "Customer is looking for eco-friendly products under $100"
}

// Enhanced Vector embedding for Customer Desire Memory
V::CustomerDesireEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text with desire details
    desire_subject: String DEFAULT "",             // What customer wants for direct matching
    urgency_context: String DEFAULT "",            // "urgent immediate flexible patient can wait"
    budget_context: String DEFAULT "",             // "budget conscious value seeker premium quality cost"
    purpose_context: String DEFAULT "",            // "personal business gift family work entertainment"
    quality_context: String DEFAULT "",           // "basic standard premium luxury professional grade"
    feature_context: String DEFAULT "",           // "specific features requirements must have nice to have"
    brand_context: String DEFAULT "",             // "brand preference flexible loyal specific no preference"
    timing_context: String DEFAULT "",            // "now soon eventually seasonal holiday specific date"
    research_context: String DEFAULT "",          // "researching comparing decided ready to buy exploring"
    motivation_context: String DEFAULT "",        // "need want upgrade replacement first time repeat"
    constraint_context: String DEFAULT "",        // "budget limited space restricted compatibility required"
    outcome_context: String DEFAULT "",           // "problem solving improvement convenience status comfort"
    influence_context: String DEFAULT "",         // "self decided family input expert recommendation peer"
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Link desire node to its embedding vector
E::HasDesireEmbedding {
    From: CustomerDesireMemory,
    To: CustomerDesireEmbedding,
    Properties: {
        created_at: I64
    }
}

// Customer Rule Memory Node - Customer rules and constraints
N::CustomerRuleMemory {
    customer_id: String,                         // Customer identifier
    rule_id: String,                            // Unique rule identifier
    rule_type: String DEFAULT "",                // "constraint", "requirement", "preference_rule" - empty if not specified
    category: String DEFAULT "",                 // "communication", "delivery", "payment", "service" - empty if not specified
    rule_description: String DEFAULT "",         // What rule applies
    enforcement: String DEFAULT "",              // "strict", "flexible", "guideline" - empty if not specified
    exceptions: [String],                       // When rule doesn't apply (must remain required - arrays can't have defaults)
    is_active: Boolean DEFAULT false,           // Whether rule is still in effect - safer to default false
    created_at: I64 DEFAULT NOW,               // Timestamp
    updated_at: I64 DEFAULT NOW,               // Last update timestamp
    text_description: String DEFAULT ""         // Natural language description for embeddings
                                               // Example: "Customer prefers email communication only, no phone calls"
}

// Enhanced Vector embedding for Customer Rule Memory
V::CustomerRuleEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text with rule details
    rule_subject: String DEFAULT "",               // What rule applies to for direct matching
    enforcement_context: String DEFAULT "",        // "strict flexible conditional situational absolute"
    scope_context: String DEFAULT "",              // "communication service product pricing schedule contact"
    compliance_context: String DEFAULT "",         // "always never sometimes exceptions allowed required"
    priority_context: String DEFAULT "",          // "critical important moderate low flexible negotiable"
    consequence_context: String DEFAULT "",        // "deal breaker strong preference minor issue acceptable"
    communication_context: String DEFAULT "",     // "email only phone text social media in person chat"
    timing_context: String DEFAULT "",            // "immediate response hours days weeks availability"
    privacy_context: String DEFAULT "",           // "confidential private public shareable restricted sensitive"
    business_context: String DEFAULT "",          // "professional personal family medical financial legal"
    accessibility_context: String DEFAULT "",     // "visual hearing mobility cognitive language technical"
    cultural_context: String DEFAULT "",          // "religious dietary cultural language customs traditions"
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Link rule node to its embedding vector
E::HasRuleEmbedding {
    From: CustomerRuleMemory,
    To: CustomerRuleEmbedding,
    Properties: {
        created_at: I64
    }
}

// Customer Feedback Memory Node - Customer feedback and reviews
N::CustomerFeedbackMemory {
    customer_id: String,                        // Customer identifier
    feedback_id: String,                       // Unique feedback identifier
    feedback_type: String DEFAULT "",          // "review", "survey", "complaint", "suggestion" - empty if not specified
    subject: String DEFAULT "",                // What the feedback is about
    rating: I32 DEFAULT 0,                     // Rating (1-5 scale) - 0 if not specified
    sentiment: String DEFAULT "",              // "positive", "negative", "neutral" - empty if not specified
    channel: String DEFAULT "",                // Where feedback was given - empty if not specified
    response_required: Boolean DEFAULT false,  // Whether business response is needed
    resolved: Boolean DEFAULT false,           // Whether feedback has been addressed
    created_at: I64 DEFAULT NOW,              // Timestamp
    updated_at: I64 DEFAULT NOW,              // Last update timestamp
    text_description: String DEFAULT ""       // Natural language description for embeddings
                                              // Example: "Customer left a 5-star review praising the excellent customer service"
}

// Enhanced Vector embedding for Customer Feedback Memory
V::CustomerFeedbackEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text with feedback details
    feedback_subject: String DEFAULT "",           // What feedback is about for direct matching
    sentiment_context: String DEFAULT "",          // "positive negative neutral mixed satisfied dissatisfied"
    rating_context: String DEFAULT "",             // "excellent good average poor terrible outstanding"
    category_context: String DEFAULT "",           // "product service staff location price delivery quality"
    urgency_context: String DEFAULT "",            // "urgent important moderate low routine follow-up"
    resolution_context: String DEFAULT "",         // "resolved pending investigating acknowledged dismissed"
    channel_context: String DEFAULT "",            // "online review phone email in-person social media"
    tone_context: String DEFAULT "",               // "professional casual emotional frustrated grateful"
    detail_context: String DEFAULT "",             // "specific general detailed brief comprehensive vague"
    impact_context: String DEFAULT "",             // "high impact minor issue major problem suggestion praise"
    credibility_context: String DEFAULT "",        // "verified customer regular client new customer anonymous"
    actionable_context: String DEFAULT "",         // "actionable feedback complaint suggestion praise request"
    public_context: String DEFAULT "",             // "public private confidential shareable internal external"
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Link feedback node to its embedding vector
E::HasFeedbackEmbedding {
    From: CustomerFeedbackMemory,
    To: CustomerFeedbackEmbedding,
    Properties: {
        created_at: I64
    }
}

// ============================================================================
// CUSTOMER INTERACTION NODES (For storing detailed interaction data with embeddings)
// ============================================================================

// Customer Product Interaction Node - Stores detailed interaction with reasons
N::CustomerProductInteraction {
    customer_id: String,                          // Customer identifier
    product_id: String,                          // Product identifier
    interaction_id: String,                      // Unique interaction identifier
    interaction_type: String DEFAULT "",         // "liked", "disliked", "purchased", "viewed", "favorited", "reviewed" - empty if not specified
    rating: I32 DEFAULT 0,                       // Rating if applicable (1-5 scale)
    timestamp: I64 DEFAULT NOW,                  // When interaction occurred
    channel: String DEFAULT "",                  // Where interaction happened ("whatsapp", "website", "store") - empty to avoid misleading LLM
    session_duration: I32 DEFAULT 0,             // How long customer engaged (seconds)
    purchase_amount: F64 DEFAULT 0.0,            // Amount spent if purchased
    currency: String DEFAULT "",                 // Currency - empty if not specified
    issue_category: String DEFAULT "",           // For dislikes: "quality", "price", "functionality", "service"
    resolution_status: String DEFAULT "",        // For issues: "resolved", "pending", "escalated" - empty if not specified
    created_at: I64 DEFAULT NOW,                 // Timestamp
    updated_at: I64 DEFAULT NOW,                 // Last update timestamp
    text_reason: String DEFAULT ""               // Natural language reason for interaction
                                                 // Example: "Customer loved the camera quality and battery life"
}

// Enhanced Vector embedding for Customer Product Interaction
V::CustomerProductInteractionEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text with interaction details
    interaction_type: String DEFAULT "",           // Interaction type for direct matching
    sentiment_context: String DEFAULT "",          // "positive negative neutral excited disappointed satisfied"
    engagement_context: String DEFAULT "",         // "brief extended focused casual deep superficial"
    outcome_context: String DEFAULT "",            // "purchased abandoned researched compared bookmarked shared"
    value_context: String DEFAULT "",              // "high value budget premium quality price conscious"
    feature_context: String DEFAULT "",            // "loved hated impressed disappointed surprised expected"
    comparison_context: String DEFAULT "",         // "better worse similar unique different typical"
    timing_context: String DEFAULT "",             // "immediate impulse researched planned seasonal urgent"
    channel_context: String DEFAULT "",            // "online in-store mobile desktop social media referral"
    decision_context: String DEFAULT "",           // "final decision still deciding comparing researching ready"
    influence_context: String DEFAULT "",          // "self decided family recommended expert advised peer"
    experience_context: String DEFAULT "",         // "first time repeat customer experienced novice expert"
    resolution_context: String DEFAULT "",         // "resolved pending needs help satisfied dissatisfied"
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// ============================================================================
// BEHAVIORAL CONNECTION EDGES (Simplified - no vector data in edges)
// ============================================================================

// Link customer to their product interactions
E::CustomerHasProductInteraction {
    From: CustomerBehaviorMemory,
    To: CustomerProductInteraction,
    Properties: {
        created_at: I64
    }
}

// Link product interaction to the specific product
E::InteractionAboutProduct {
    From: CustomerProductInteraction,
    To: BusinessProductMemory,
    Properties: {
        created_at: I64
    }
}

// Link interaction to its embedding
E::HasProductInteractionEmbedding {
    From: CustomerProductInteraction,
    To: CustomerProductInteractionEmbedding,
    Properties: {
        created_at: I64
    }
}

// Customer Service Interaction Node - Stores detailed service interaction with feedback
N::CustomerServiceInteraction {
    customer_id: String,                           // Customer identifier
    service_id: String,                           // Service identifier
    interaction_id: String,                       // Unique interaction identifier
    interaction_type: String DEFAULT "booked",    // "booked", "completed", "reviewed", "canceled"
    satisfaction_rating: I32 DEFAULT 3,           // Rating (1-5 scale)
    timestamp: I64 DEFAULT NOW,                   // When service was used
    duration_actual: I32 DEFAULT 0,               // Actual duration in minutes
    cost_actual: F64 DEFAULT 0.0,                 // Actual cost paid
    currency: String DEFAULT "",                  // Currency - empty if not specified
    outcome: String DEFAULT "",                   // Service outcome/result - empty if not specified
    created_at: I64 DEFAULT NOW,                  // Timestamp
    updated_at: I64 DEFAULT NOW,                  // Last update timestamp
    text_feedback: String DEFAULT ""             // Natural language feedback
                                                 // Example: "Service was excellent but took longer than expected"
}

// Enhanced Vector embedding for Customer Service Interaction
V::CustomerServiceInteractionEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text with service interaction details
    interaction_type: String DEFAULT "",           // Interaction type for direct matching
    satisfaction_context: String DEFAULT "",       // "excellent good satisfactory poor terrible outstanding"
    service_quality_context: String DEFAULT "",    // "professional expert helpful knowledgeable friendly efficient"
    timing_context: String DEFAULT "",             // "on time delayed quick slow fast efficient prompt"
    communication_context: String DEFAULT "",      // "clear confusing helpful responsive attentive patient"
    outcome_context: String DEFAULT "",            // "successful failed partial complete exceeded expectations"
    value_context: String DEFAULT "",              // "worth it expensive reasonable great value overpriced fair"
    experience_context: String DEFAULT "",         // "smooth difficult pleasant stressful easy complicated"
    staff_context: String DEFAULT "",              // "friendly professional rude helpful patient knowledgeable"
    process_context: String DEFAULT "",            // "simple complicated smooth bureaucratic efficient streamlined"
    follow_up_context: String DEFAULT "",          // "excellent poor none needed immediate delayed comprehensive"
    recommendation_context: String DEFAULT "",     // "highly recommend avoid maybe recommend with reservations"
    repeat_context: String DEFAULT "",             // "will use again never again might consider definitely"
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Link customer to their service interactions
E::CustomerHasServiceInteraction {
    From: CustomerPreferenceMemory,
    To: CustomerServiceInteraction,
    Properties: {
        created_at: I64
    }
}

// Link service interaction to the specific service
E::InteractionAboutService {
    From: CustomerServiceInteraction,
    To: BusinessServiceMemory,
    Properties: {
        created_at: I64
    }
}

// Link interaction to its embedding
E::HasServiceInteractionEmbedding {
    From: CustomerServiceInteraction,
    To: CustomerServiceInteractionEmbedding,
    Properties: {
        created_at: I64
    }
}

// Customer Location Visit Node - Stores location visit experiences
N::CustomerLocationVisit {
    customer_id: String,                        // Customer identifier
    location_id: String,                       // Location identifier
    visit_id: String,                          // Unique visit identifier
    visit_type: String DEFAULT "browsing",     // "pickup", "browsing", "service_visit", "purchase"
    timestamp: I64 DEFAULT NOW,                // When visit occurred
    duration_minutes: I32 DEFAULT 0,           // How long customer stayed
    party_size: I32 DEFAULT 1,                 // Number of people with customer
    purchase_made: Boolean DEFAULT false,      // Whether purchase was made
    purchase_amount: F64 DEFAULT 0.0,          // Amount spent
    currency: String DEFAULT "",               // Currency - empty if not specified
    created_at: I64 DEFAULT NOW,               // Timestamp
    updated_at: I64 DEFAULT NOW,               // Last update timestamp
    text_experience: String DEFAULT ""        // Natural language experience description
                                              // Example: "Customer found location easy to access but parking was difficult"
}

// Enhanced Vector embedding for Customer Location Visit
V::CustomerLocationVisitEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text with visit experience details
    visit_type: String DEFAULT "",                 // Visit type for direct matching
    accessibility_context: String DEFAULT "",      // "easy difficult accessible parking elevator stairs"
    atmosphere_context: String DEFAULT "",         // "welcoming intimidating comfortable professional casual"
    navigation_context: String DEFAULT "",         // "easy to find confusing well marked hidden clear signs"
    staff_context: String DEFAULT "",              // "helpful friendly professional busy unavailable knowledgeable"
    crowd_context: String DEFAULT "",              // "busy quiet crowded empty comfortable overwhelming"
    convenience_context: String DEFAULT "",        // "convenient inconvenient fast slow efficient frustrating"
    purchase_context: String DEFAULT "",           // "bought nothing browsed purchased planned impulse considered"
    satisfaction_context: String DEFAULT "",       // "satisfied disappointed impressed neutral exceeded expectations"
    return_context: String DEFAULT "",             // "will return avoid might consider definitely recommend"
    timing_context: String DEFAULT "",             // "good time bad timing rush hour quiet period busy"
    logistics_context: String DEFAULT "",          // "parking transport access entrance exit payment"
    comparison_context: String DEFAULT "",         // "better worse than expected similar different unique"
    value_context: String DEFAULT "",              // "good value expensive reasonable overpriced worth it"
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Link customer behavior to location visit
E::CustomerHasLocationVisit {
    From: CustomerBehaviorMemory,
    To: CustomerLocationVisit,
    Properties: {
        created_at: I64
    }
}

// Link visit to the location
E::VisitAtLocation {
    From: CustomerLocationVisit,
    To: BusinessLocationMemory,
    Properties: {
        created_at: I64
    }
}

// Link visit to its embedding
E::HasLocationVisitEmbedding {
    From: CustomerLocationVisit,
    To: CustomerLocationVisitEmbedding,
    Properties: {
        created_at: I64
    }
}

// ============================================================================
// DATA ISOLATION & COLLABORATION EDGES
// ============================================================================

// Memory Ownership - Links memories to their owning business (DATA ISOLATION)
E::OwnedBy {
    From: BusinessProductMemory,  // Can be any business memory type
    To: Business,
    Properties: {
        created_at: I64,
        visibility: String        // "private", "shared", "public"
    }
}

// Customer Belongs To Business - Links customers to businesses they interact with
E::CustomerOf {
    From: Customer,
    To: Business,
    Properties: {
        first_interaction: I64,
        last_interaction: I64,
        interaction_count: I32,
        customer_status: String,  // "active", "inactive", "vip"
        consent_to_share: Boolean // Whether customer consents to data sharing
    }
}

// Memory About Customer - Links memories to the customer they're about
E::AboutCustomer {
    From: CustomerPreferenceMemory,  // Can be any customer memory type
    To: Customer,
    Properties: {
        created_at: I64,
        confidence: F64,          // Confidence score (0.0-1.0)
        source: String            // "stated", "inferred", "observed"
    }
}

// Business Collaboration - Links businesses that share data (CONTROLLED SHARING)
E::CollaboratesWith {
    From: Business,
    To: Business,
    Properties: {
        collaboration_type: String,  // "partnership", "network", "marketplace"
        share_customer_data: Boolean,
        share_product_data: Boolean,
        requires_consent: Boolean,
        started_at: I64,
        active: Boolean
    }
}

// Shared Access - Explicit sharing of specific memories between businesses
E::SharedWith {
    From: BusinessProductMemory,  // Can be any memory type
    To: Business,
    Properties: {
        shared_by: String,        // Business ID that shared it
        shared_at: I64,
        permission: String,       // "read", "reference"
        expires_at: I64,
        anonymized: Boolean
    }
}


// Customer Business Communication Node - Stores communication interactions
N::CustomerBusinessCommunication {
    customer_id: String,                           // Customer identifier
    business_id: String,                          // Business identifier  
    communication_id: String,                     // Unique communication identifier
    contact_method: String DEFAULT "",            // "phone", "email", "chat", "social_media", "whatsapp" - empty to avoid misleading LLM
    contact_reason: String DEFAULT "inquiry",     // "inquiry", "complaint", "feedback", "support"
    timestamp: I64 DEFAULT NOW,                   // When contact occurred
    duration_seconds: I32 DEFAULT 0,              // How long interaction lasted
    resolution_status: String DEFAULT "pending",  // "resolved", "pending", "escalated", "unresolved"
    agent_id: String DEFAULT "",                  // Agent/staff member who handled
    channel_details: String DEFAULT "",           // Specific channel info (email address, phone number, etc.)
    created_at: I64 DEFAULT NOW,                  // Timestamp
    updated_at: I64 DEFAULT NOW,                  // Last update timestamp
    text_interaction: String DEFAULT ""          // Natural language interaction summary
                                                 // Example: "Customer called about delivery delay, was understanding when explained"
}

// Enhanced Vector embedding for Customer Business Communication
V::CustomerCommunicationEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text with communication details
    contact_method: String DEFAULT "",             // Method for direct matching
    contact_reason: String DEFAULT "",             // Reason for direct matching
    urgency_context: String DEFAULT "",            // "urgent routine emergency immediate non-critical flexible"
    tone_context: String DEFAULT "",               // "friendly professional frustrated angry satisfied calm"
    resolution_context: String DEFAULT "",         // "resolved pending escalated abandoned satisfied dissatisfied"
    complexity_context: String DEFAULT "",         // "simple complex technical straightforward complicated advanced"
    agent_performance_context: String DEFAULT "",  // "excellent good average poor helpful professional"
    customer_mood_context: String DEFAULT "",      // "happy upset frustrated excited satisfied disappointed"
    outcome_context: String DEFAULT "",            // "successful failed partial complete exceeded expectations"
    efficiency_context: String DEFAULT "",         // "quick slow immediate delayed efficient frustrating"
    follow_up_context: String DEFAULT "",          // "required completed none needed scheduled immediate"
    channel_context: String DEFAULT "",            // "preferred accessible difficult convenient immediate delayed"
    expertise_context: String DEFAULT "",          // "knowledgeable expert basic advanced technical simple"
    satisfaction_context: String DEFAULT "",       // "very satisfied neutral disappointed impressed frustrated"
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Link customer behavior to communication
E::CustomerHasCommunication {
    From: CustomerBehaviorMemory,
    To: CustomerBusinessCommunication,
    Properties: {
        created_at: I64
    }
}

// Link communication to business policy
E::CommunicationAboutPolicy {
    From: CustomerBusinessCommunication,
    To: BusinessPolicyMemory,
    Properties: {
        created_at: I64
    }
}

// Link communication to its embedding
E::HasCommunicationEmbedding {
    From: CustomerBusinessCommunication,
    To: CustomerCommunicationEmbedding,
    Properties: {
        created_at: I64
    }
}

// ============================================================================
// UNIVERSAL BUSINESS NAVIGATION SYSTEM (Multi-Tenant Flexible)
// For LLM-powered customer direction and guidance via MCP server
// ============================================================================

// Universal Business Navigation Hub - Flexible for all business types
N::BusinessNavigationHub {
    business_id: String,                              // Required - tenant isolation
    navigation_id: String,                           // Required - unique identifier
    
    // Basic Location (Optional - some may only have internal navigation)
    primary_address: String DEFAULT "",               // Can be empty for internal-only navigation
    secondary_address: String DEFAULT "",             // Alternate address or description
    building_name: String DEFAULT "",                 // "City Mall", "General Hospital", "ABC School"
    building_type: String DEFAULT "",                 // "retail", "educational", "medical", "office", "residential" - empty to avoid misleading LLM
    
    // Flexible Positioning (All Optional)
    latitude: F64 DEFAULT 0.0,                       // GPS coordinates (optional)
    longitude: F64 DEFAULT 0.0,                      // GPS coordinates (optional)
    what3words_code: String DEFAULT "",              // What3Words (optional)
    plus_code: String DEFAULT "",                    // Google Plus Code (optional)
    
    // COMPASS POSITIONING DATA (Optional but Recommended)
    compass_bearing: F64 DEFAULT 0.0,               // Magnetic bearing in degrees (0-360) - which direction building faces
    compass_reference: String DEFAULT "",            // "north_facing", "south_entrance", "east_side"
    magnetic_declination: F64 DEFAULT 0.0,          // Local magnetic declination correction
    
    // Building Characteristics (All Optional)
    building_description: String DEFAULT "",         // "Red brick building", "Glass tower", "Single story"
    building_floors: I32 DEFAULT 0,                  // 0 for single level or not specified, N for multi-floor
    business_floor: I32 DEFAULT 0,                   // Which floor business is on (0 for ground)
    building_color: String DEFAULT "",               // "red", "blue glass", "stone gray"
    building_size: String DEFAULT "",               // "small", "medium", "large", "campus" - empty to avoid misleading LLM
    
    // Entry Information (All Optional)
    main_entrance_description: String DEFAULT "",    // "Front door facing parking"
    alternative_entrances: String DEFAULT "[]",     // JSON array of other entrances
    entrance_restrictions: String DEFAULT "",        // "Staff only after 6pm", "Visitors use side door"
    
    // Accessibility (All Optional)
    wheelchair_accessible: Boolean DEFAULT false,    // false if not specified - safer to assume not accessible
    elevator_available: Boolean DEFAULT false,       // false if not applicable
    stairs_required: Boolean DEFAULT false,          // false if not applicable
    accessibility_notes: String DEFAULT "",          // Detailed accessibility info
    
    // Parking & Transport (All Optional)
    parking_available: Boolean DEFAULT false,        // false if not specified - safer to assume no parking
    parking_description: String DEFAULT "",          // "Behind building", "Street parking only"
    public_transport_notes: String DEFAULT "",       // Bus stops, metro stations nearby
    
    // Business Hours Impact (Optional)
    direction_varies_by_hours: Boolean DEFAULT false, // true if directions change after hours
    after_hours_instructions: String DEFAULT "",     // Different entry after business hours
    
    // Metadata
    created_at: I64 DEFAULT NOW,                     // Required
    updated_at: I64 DEFAULT NOW,                     // Required
    last_verified_at: I64 DEFAULT NOW,               // When directions were last confirmed
    verification_source: String DEFAULT "",          // "owner", "staff", "customer", "automated" - empty if not specified
    
    // Rich Context for LLM (Optional but Recommended)
    navigation_summary: String DEFAULT ""           // Natural language summary for LLM
                                                    // "Small coffee shop in red brick plaza, use main entrance, no stairs"
}

// Enhanced Vector embedding for Business Navigation Hub
V::BusinessNavigationEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text with navigation details
    location_context: String DEFAULT "",           // "downtown plaza mall hospital campus building"
    building_context: String DEFAULT "",           // "red brick glass modern historic two-story ground floor"
    access_context: String DEFAULT "",             // "main entrance side door elevator stairs wheelchair ramp"
    landmark_context: String DEFAULT "",           // "near bank library post office traffic light park"
    transport_context: String DEFAULT "",          // "bus stop metro train taxi uber walking distance"
    parking_context: String DEFAULT "",            // "free parking paid garage street parking valet"
    accessibility_context: String DEFAULT "",      // "wheelchair accessible elevator required stairs optional"
    compass_context: String DEFAULT "",            // "north facing south entrance east side west parking"
    visibility_context: String DEFAULT "",         // "easy to find well marked hidden entrance clear signs"
    timing_context: String DEFAULT "",             // "business hours after hours weekend holiday access"
    navigation_summary: String DEFAULT "",         // Original navigation summary
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Flexible Navigation Waypoint - Replaces rigid landmarks
N::NavigationWaypoint {
    waypoint_id: String,                            // Required
    navigation_id: String,                          // Links to BusinessNavigationHub
    
    // Waypoint Identity (Flexible)
    waypoint_name: String DEFAULT "",               // "Main Entrance", "Parking Lot", "Reception Desk"
    waypoint_type: String DEFAULT "",               // "entrance", "parking", "landmark", "internal", "exit" - empty if not specified
    waypoint_category: String DEFAULT "",           // "external", "internal", "boundary" - empty if not specified
    
    // Description (Optional)
    description: String DEFAULT "",                 // "Large glass doors with company logo"
    visual_cues: String DEFAULT "",                // "Red awning", "Fountain nearby", "Security desk"
    audio_cues: String DEFAULT "",                 // "Construction noise", "Traffic sounds"
    
    // Position with Compass Data (All Optional)
    relative_position: String DEFAULT "",           // "north_of_building", "inside_lobby", "second_floor"
    distance_from_main: I32 DEFAULT 0,             // Meters/feet from main entrance (0 if not applicable)
    floor_level: I32 DEFAULT 0,                    // Which floor (0 for ground level)
    
    // COMPASS POSITIONING FOR WAYPOINTS
    compass_direction: String DEFAULT "",           // "north", "northeast", "south", "southwest", etc. - empty if not specified
    compass_bearing: F64 DEFAULT 0.0,              // Exact bearing in degrees (0-360) from main entrance
    compass_distance: F64 DEFAULT 0.0,             // Distance in meters using compass bearing
    
    // Context (Optional)
    business_specific_notes: String DEFAULT "",     // "Counter 3 for returns", "Room 204", "Pharmacy section"
    accessibility_info: String DEFAULT "",          // "Ramp available", "Staff assistance needed"
    
    // Conditions (Optional)
    seasonal_availability: String DEFAULT "",       // "winter_closed", "summer_only", "always" - empty if not specified
    time_restrictions: String DEFAULT "",           // "9am-5pm only", "weekends_closed"
    weather_dependent: Boolean DEFAULT false,       // true if affected by weather
    
    // Metadata
    created_at: I64 DEFAULT NOW,
    is_active: Boolean DEFAULT false,               // Can be disabled without deletion - safer to default false
    priority_level: I32 DEFAULT 0                   // 0=not set, 1=critical, 2=important, 3=optional
}

// Enhanced Vector embedding for Navigation Waypoint
V::NavigationWaypointEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text with waypoint details
    waypoint_type_context: String DEFAULT "",      // "entrance exit landmark sign building feature"
    location_context: String DEFAULT "",           // "front back side corner ground floor second floor"
    visibility_context: String DEFAULT "",         // "clearly visible hidden marked obvious subtle"
    accessibility_context: String DEFAULT "",      // "accessible barrier stairs ramp elevator required"
    compass_context: String DEFAULT "",            // "north south east west northeast northwest"
    distance_context: String DEFAULT "",           // "close far immediate next to across from"
    recognition_context: String DEFAULT "",        // "distinctive unique common obvious typical landmark"
    seasonal_context: String DEFAULT "",           // "year round seasonal temporary permanent weather dependent"
    traffic_context: String DEFAULT "",            // "busy quiet crowded empty pedestrian vehicle"
    safety_context: String DEFAULT "",             // "safe secure well lit dark private public"
    waypoint_description: String DEFAULT "",       // Original waypoint description
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// Universal Direction Path - Multi-tenant navigation routes
N::DirectionPath {
    path_id: String,                                      // Required
    navigation_id: String,                               // Links to BusinessNavigationHub
    
    // Path Identity
    path_name: String DEFAULT "",                        // "Main Walking Route", "Parking to Entrance", "Internal Navigation"
    path_type: String DEFAULT "",                        // "approach", "internal", "departure", "emergency" - empty if not specified
    transport_mode: String DEFAULT "",                   // "walking", "driving", "wheelchair", "public_transport" - empty if not specified
    
    // Path Characteristics (All Optional)
    estimated_duration_minutes: I32 DEFAULT 0,          // 0 if not specified
    difficulty_level: String DEFAULT "",                 // "easy", "moderate", "difficult" - empty if not specified
    distance_meters: I32 DEFAULT 0,                      // 0 if not applicable
    
    // COMPASS-BASED PATH DATA
    starting_compass_bearing: F64 DEFAULT 0.0,          // Initial bearing when starting path
    ending_compass_bearing: F64 DEFAULT 0.0,            // Final bearing when reaching destination
    path_compass_waypoints: String DEFAULT "[]",         // JSON array of compass bearings for turns
    
    // Suitability (Optional)
    suitable_for_mobility_aids: Boolean DEFAULT false,   // false if not specified - safer assumption
    suitable_for_children: Boolean DEFAULT false,        // false if not specified - safer assumption
    suitable_in_rain: Boolean DEFAULT false,             // false if not specified - safer assumption
    suitable_at_night: Boolean DEFAULT false,            // false if not specified - safer assumption
    
    // Business Type Adaptations (Optional)
    requires_appointment: Boolean DEFAULT false,         // For medical/professional services
    requires_security_clearance: Boolean DEFAULT false, // For secure facilities
    visitor_badge_required: Boolean DEFAULT false,      // For offices/institutions
    
    // Instructions (Optional)
    step_by_step_instructions: String DEFAULT "",       // Detailed directions
    quick_summary: String DEFAULT "",                   // "Enter main door, take elevator to 3rd floor"
    
    // Metadata
    created_at: I64 DEFAULT NOW,
    is_recommended: Boolean DEFAULT false,              // Primary route - safer to default false
    is_active: Boolean DEFAULT false,                   // Can be disabled - safer to default false
    last_used_feedback: String DEFAULT ""              // Latest user feedback
}

// Enhanced Vector embedding for Direction Path
V::DirectionPathEmbedding {
    composite_embedding_text: String DEFAULT "",    // Rich composite text with path details
    transport_context: String DEFAULT "",          // "walking driving cycling public transport wheelchair"
    difficulty_context: String DEFAULT "",         // "easy difficult moderate simple complex straightforward"
    duration_context: String DEFAULT "",           // "quick fast slow immediate minutes hour"
    accessibility_context: String DEFAULT "",      // "accessible barrier-free elevator required stairs involved"
    weather_context: String DEFAULT "",            // "indoor outdoor covered exposed weather dependent sheltered"
    safety_context: String DEFAULT "",             // "safe secure well lit private public monitored"
    time_context: String DEFAULT "",               // "business hours after hours anytime restricted access"
    traffic_context: String DEFAULT "",            // "busy quiet crowded empty pedestrian vehicle"
    landmark_context: String DEFAULT "",           // "clear landmarks obvious signs well marked confusing"
    compass_context: String DEFAULT "",            // "north south straight left right compass bearing"
    convenience_context: String DEFAULT "",        // "convenient direct shortest fastest most comfortable"
    requirements_context: String DEFAULT "",       // "no requirements badge needed appointment security clearance"
    path_instructions: String DEFAULT "",          // Original path instructions
    embedding_model: String DEFAULT "local",       // Model used for embedding
    embedding_date: I64 DEFAULT NOW,               // When embedding was created
    embedding_version: String DEFAULT "1.0"        // Version for tracking updates
}

// ============================================================================
// NAVIGATION SYSTEM EDGES
// ============================================================================

// Link business to navigation hub
E::HasNavigation {
    From: Business,
    To: BusinessNavigationHub,
    Properties: {
        is_primary: Boolean,        // Main navigation (businesses can have multiple)
        created_at: I64
    }
}

// Link navigation hub to waypoints
E::HasWaypoint {
    From: BusinessNavigationHub,
    To: NavigationWaypoint,
    Properties: {
        waypoint_order: I32,        // Sequence order (0 if not sequential)
        is_critical: Boolean,       // Must-have waypoint
        created_at: I64
    }
}

// Link navigation hub to paths
E::HasPath {
    From: BusinessNavigationHub,
    To: DirectionPath,
    Properties: {
        is_default: Boolean,        // Default path for this business type
        context_tags: String,       // JSON: ["first_time", "returning", "delivery"]
        created_at: I64
    }
}

// Connect paths through waypoints (optional - for complex routing)
E::PathThroughWaypoint {
    From: DirectionPath,
    To: NavigationWaypoint,
    Properties: {
        sequence_order: I32,        // Order in path (0 if not sequential)
        is_optional: Boolean,       // Can be skipped
        created_at: I64
    }
}

// Link navigation hub to its embedding
E::HasNavigationEmbedding {
    From: BusinessNavigationHub,
    To: BusinessNavigationEmbedding,
    Properties: {
        created_at: I64
    }
}

// Link waypoint to its embedding
E::HasWaypointEmbedding {
    From: NavigationWaypoint,
    To: NavigationWaypointEmbedding,
    Properties: {
        created_at: I64
    }
}

// Link path to its embedding
E::HasPathEmbedding {
    From: DirectionPath,
    To: DirectionPathEmbedding,
    Properties: {
        created_at: I64
    }
}

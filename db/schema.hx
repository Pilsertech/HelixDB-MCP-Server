// HelixDB Schema for AI Memory Layer
// Business and Customer Memory Nodes with Vector Embeddings

// ============================================================================
// BUSINESS/ORGANIZATION ENTITIES (For Isolation & Collaboration)
// ============================================================================

// Business/Organization Node - Represents a tenant (business, service provider, institution)
N::Business {
    business_id: String,         // Unique business identifier
    business_name: String,       // Business name
    business_type: String,       // "business", "service_provider", "institution", "organization"
    status: String,              // "active", "suspended", "trial"
    allow_collaboration: Boolean, // Whether this business allows data sharing
    created_at: I64,
    metadata: String             // JSON for additional fields
}

// Customer/User Node - Represents a person interacting with businesses
N::Customer {
    customer_id: String,         // Unique customer identifier
    customer_name: String,       // Customer name (can be empty for privacy)
    phone: String,               // Phone number
    email: String,               // Email (optional)
    language: String,            // Preferred language
    created_at: I64,
    metadata: String             // JSON for additional fields
}

// ============================================================================
// BUSINESS MEMORY NODES
// Each business aspect (products, services, location, etc.) gets its own node
// ============================================================================

// Product Memory Node - Each product is a separate memory node
N::BusinessProductMemory {
    business_id: String,           // Business identifier
    product_id: String,           // Unique product identifier
    product_name: String,         // Product name
    product_category: String,     // Category (electronics, clothing, etc.)
    price: F64,                  // Product price
    currency: String,            // Currency code (USD, EUR, etc.)
    availability: String,        // "in_stock", "out_of_stock", "discontinued"
    description: String,         // Detailed product description
    features: [String],          // List of product features
    specifications: String,      // Technical specs as JSON string
    tags: [String],              // Search tags
    seo_keywords: [String],      // SEO keywords for search optimization
    competitor_analysis: String, // Analysis of competitor products
    seasonal_trends: String,     // Seasonal demand trends as JSON string
    created_at: I64,             // Timestamp
    updated_at: I64,             // Last update timestamp
    text_description: String     // RICH Natural language description for embeddings
                                 // Format: "Product: {name}. Category: {category}. Price: {price} {currency}. Description: {description}. Features: {features}. Tags: {tags}. Availability: {availability}"
}

// Vector embedding for Business Product Memory
V::BusinessProductEmbedding {
    text_description: String,    // The exact text that was embedded
    embedding_model: String      // Model used for embedding
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
    business_id: String,          // Business identifier
    service_id: String,          // Unique service identifier
    service_name: String,        // Service name
    service_category: String,    // Category (consulting, repair, etc.)
    price: F64,                  // Service price
    currency: String,            // Currency code
    duration_minutes: I32,       // Service duration in minutes
    availability: String,        // "available", "booked", "unavailable"
    description: String,         // Detailed service description
    requirements: [String],      // Service requirements
    deliverables: [String],      // What customer gets
    tags: [String],              // Search tags
    created_at: I64,             // Timestamp
    updated_at: I64,             // Last update timestamp
    text_description: String     // Natural language description for embeddings
}

// Vector embedding for Business Service Memory
V::BusinessServiceEmbedding {
    text_description: String,    // The text that was embedded
    embedding_model: String      // Model used for embedding
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
    business_id: String,          // Business identifier
    location_id: String,         // Unique location identifier
    location_name: String,       // Location name (main office, branch, etc.)
    address: String,             // Full address
    city: String,               // City
    state: String,              // State/Province
    country: String,            // Country
    postal_code: String,        // Postal/ZIP code
    latitude: F64,              // GPS latitude
    longitude: F64,             // GPS longitude
    location_type: String,      // "headquarters", "branch", "warehouse", etc.
    accessibility: [String],    // Accessibility features
    parking_info: String,       // Parking information
    created_at: I64,            // Timestamp
    updated_at: I64,            // Last update timestamp
    text_description: String    // RICH Natural language description
                                // Format: "Location: {name}. Address: {full_address}. Type: {type}. Accessibility: {features}. Parking: {info}"
}

// Vector embedding for Business Location Memory
V::BusinessLocationEmbedding {
    text_description: String,    // The exact text that was embedded
    embedding_model: String      // Model used for embedding
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
    business_id: String,         // Business identifier
    hours_id: String,           // Unique hours identifier
    schedule_type: String,      // "regular", "holiday", "special_event"
    monday_open: String,        // Opening time (HH:MM format)
    monday_close: String,       // Closing time (HH:MM format)
    tuesday_open: String,       // Opening time
    tuesday_close: String,      // Closing time
    wednesday_open: String,     // Opening time
    wednesday_close: String,    // Closing time
    thursday_open: String,      // Opening time
    thursday_close: String,     // Closing time
    friday_open: String,        // Opening time
    friday_close: String,       // Closing time
    saturday_open: String,      // Opening time
    saturday_close: String,     // Closing time
    sunday_open: String,        // Opening time
    sunday_close: String,       // Closing time
    timezone: String,           // Timezone (America/New_York, etc.)
    exceptions: String,         // Special dates and hours as JSON string
    created_at: I64,            // Timestamp
    updated_at: I64,            // Last update timestamp
    text_description: String    // RICH Natural language description
                                // Format: "Business hours: Monday {open}-{close}, Tuesday {open}-{close}... Timezone: {tz}. Special: {exceptions}"
}

// Vector embedding for Business Hours Memory
V::BusinessHoursEmbedding {
    text_description: String,    // The exact text that was embedded
    embedding_model: String      // Model used for embedding
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
    business_id: String,         // Business identifier
    social_id: String,          // Unique social media identifier
    platform: String,           // "facebook", "twitter", "instagram", etc.
    handle: String,             // @handle or profile URL
    profile_url: String,        // Full profile URL
    follower_count: I64,        // Number of followers
    post_count: I64,           // Number of posts
    description: String,        // Profile description/bio
    contact_info: String,       // Contact information shared on social
    last_updated: I64,          // When social data was last fetched
    created_at: I64,            // Timestamp
    updated_at: I64,            // Last update timestamp
    text_description: String    // RICH Natural language description
                                // Format: "Social media: {platform}. Handle: {handle}. Bio: {description}. Followers: {count}. Contact: {info}"
}

// Vector embedding for Business Social Memory
V::BusinessSocialEmbedding {
    text_description: String,    // The exact text that was embedded
    embedding_model: String      // Model used for embedding
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
    policy_type: String,        // "refund", "shipping", "privacy", "terms", etc.
    policy_name: String,        // Human-readable policy name
    content: String,            // Full policy content
    effective_date: I64,        // When policy takes effect
    version: String,            // Policy version
    is_active: Boolean,         // Whether policy is currently active
    tags: [String],             // Search tags
    created_at: I64,            // Timestamp
    updated_at: I64,            // Last update timestamp
    text_description: String    // RICH Natural language description
                                // Format: "Policy: {name}. Type: {type}. Summary: {content_excerpt}. Effective: {date}. Tags: {tags}"
}

// Vector embedding for Business Policy Memory
V::BusinessPolicyEmbedding {
    text_description: String,    // The exact text that was embedded
    embedding_model: String      // Model used for embedding
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
    event_type: String,          // "promotion", "sale", "webinar", etc.
    start_date: I64,             // Event start timestamp
    end_date: I64,               // Event end timestamp
    description: String,         // Detailed event description
    location: String,            // Event location or virtual link
    capacity: I32,               // Maximum capacity
    registration_required: Boolean, // Whether registration is needed
    tags: [String],              // Search tags
    created_at: I64,             // Timestamp
    updated_at: I64,             // Last update timestamp
    text_description: String     // RICH Natural language description
                                 // Format: "Event: {name}. Type: {type}. Date: {start} to {end}. Location: {location}. Capacity: {capacity}. Description: {description}. Tags: {tags}"
}

// Vector embedding for Business Event Memory
V::BusinessEventEmbedding {
    text_description: String,    // The exact text that was embedded
    embedding_model: String      // Model used for embedding
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
    context: String,            // Context of the behavior
    timestamp: I64,             // When behavior occurred
    channel: String,            // "website", "mobile_app", "phone", "in_store"
    duration_seconds: I32,      // How long the behavior lasted (if applicable)
    metadata: String,           // Additional structured data as JSON string
    created_at: I64,            // Timestamp
    updated_at: I64,            // Last update timestamp
    text_description: String    // Natural language description for embeddings
                               // Example: "Customer spent 15 minutes browsing smartphones on mobile app"
}

// Vector embedding for Customer Behavior Memory
V::CustomerBehaviorEmbedding {
    text_description: String,    // The text that was embedded
    embedding_model: String      // Model used for embedding
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
    customer_id: String,         // Customer identifier
    preference_id: String,      // Unique preference identifier
    preference_type: String,    // "likes", "dislikes", "preferences", "restrictions"
    category: String,           // "products", "services", "communication", "pricing"
    subject: String,            // What the preference is about
    strength: String,           // "strong", "moderate", "weak"
    is_active: Boolean,         // Whether preference is still relevant
    evidence_count: I32,        // How many times this preference was observed
    last_evidence: I64,         // When preference was last observed
    confidence_score: F64,      // How confident we are in this preference (0.0-1.0)
    source_channels: [String],  // Where this preference was observed
    created_at: I64,            // Timestamp
    updated_at: I64,            // Last update timestamp
    text_description: String    // RICH Natural language description for embeddings
                               // Format: "Customer preference: {subject}. Type: {preference_type}. Category: {category}. Strength: {strength}. Evidence: {evidence_count} observations. Details: {original_text}"
}

// Vector embedding for Customer Preference Memory
V::CustomerPreferenceEmbedding {
    text_description: String,    // The exact text that was embedded
    embedding_model: String      // Model used for embedding
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
    customer_id: String,         // Customer identifier
    desire_id: String,          // Unique desire identifier
    desire_type: String,        // "immediate", "future", "aspirational"
    category: String,           // "products", "services", "experiences"
    description: String,        // What the customer wants
    priority: String,           // "high", "medium", "low"
    timeframe: String,          // "immediate", "short_term", "long_term"
    budget_range: String,       // Budget consideration if mentioned
    is_active: Boolean,         // Whether desire is still relevant
    created_at: I64,            // Timestamp
    updated_at: I64,            // Last update timestamp
    text_description: String    // Natural language description for embeddings
                               // Example: "Customer is looking for eco-friendly products under $100"
}

// Vector embedding for Customer Desire Memory
V::CustomerDesireEmbedding {
    text_description: String,    // The text that was embedded
    embedding_model: String      // Model used for embedding
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
    customer_id: String,         // Customer identifier
    rule_id: String,            // Unique rule identifier
    rule_type: String,          // "constraint", "requirement", "preference_rule"
    category: String,           // "communication", "delivery", "payment", "service"
    rule_description: String,   // What rule applies
    enforcement: String,        // "strict", "flexible", "guideline"
    exceptions: [String],       // When rule doesn't apply
    is_active: Boolean,         // Whether rule is still in effect
    created_at: I64,            // Timestamp
    updated_at: I64,            // Last update timestamp
    text_description: String    // Natural language description for embeddings
                               // Example: "Customer prefers email communication only, no phone calls"
}

// Vector embedding for Customer Rule Memory
V::CustomerRuleEmbedding {
    text_description: String,    // The text that was embedded
    embedding_model: String      // Model used for embedding
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
    customer_id: String,         // Customer identifier
    feedback_id: String,        // Unique feedback identifier
    feedback_type: String,      // "review", "survey", "complaint", "suggestion"
    subject: String,            // What the feedback is about
    rating: I32,               // Rating (1-5 scale)
    sentiment: String,          // "positive", "negative", "neutral"
    channel: String,            // Where feedback was given
    response_required: Boolean, // Whether business response is needed
    resolved: Boolean,          // Whether feedback has been addressed
    created_at: I64,            // Timestamp
    updated_at: I64,            // Last update timestamp
    text_description: String    // Natural language description for embeddings
                               // Example: "Customer left a 5-star review praising the excellent customer service"
}

// Vector embedding for Customer Feedback Memory
V::CustomerFeedbackEmbedding {
    text_description: String,    // The text that was embedded
    embedding_model: String      // Model used for embedding
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
// BEHAVIORAL CONNECTION EDGES
// Direct edges between customers and business memories with reasons
// ============================================================================

// Customer Liked Product Edge - Direct connection with reason
E::CustomerLikedProduct {
    From: CustomerPreferenceMemory,
    To: BusinessProductMemory,
    Properties: {
        customer_id: String,         // Customer identifier
        product_id: String,         // Product identifier
        interaction_type: String,   // "purchased", "viewed", "favorited", "reviewed"
        rating: I32,               // Rating if applicable (1-5 scale)
        timestamp: I64,            // When interaction occurred
        channel: String,           // Where interaction happened
        session_duration: I32,     // How long customer engaged
        purchase_amount: F64,      // Amount spent if purchased
        currency: String,          // Currency of purchase
        created_at: I64,           // Timestamp
        updated_at: I64,           // Last update timestamp
        text_reason: String,       // Natural language reason for liking
                                  // Example: "Customer loved the camera quality and battery life"
        embedding: [F32],          // Vector embedding of the reason text
        embedding_model: String    // Model used for embedding
    }
}

// Customer Disliked Product Edge - Direct connection with reason
E::CustomerDislikedProduct {
    From: CustomerPreferenceMemory,
    To: BusinessProductMemory,
    Properties: {
        customer_id: String,        // Customer identifier
        product_id: String,        // Product identifier
        interaction_type: String,  // "returned", "complained", "low_rating"
        rating: I32,              // Rating if applicable (1-5 scale)
        timestamp: I64,           // When interaction occurred
        channel: String,          // Where interaction happened
        issue_category: String,   // "quality", "price", "functionality", "service"
        resolution_status: String,// "resolved", "pending", "escalated"
        created_at: I64,          // Timestamp
        updated_at: I64,          // Last update timestamp
        text_reason: String,      // Natural language reason for disliking
                                 // Example: "Product arrived damaged and customer service was unhelpful"
        embedding: [F32],         // Vector embedding of the reason text
        embedding_model: String   // Model used for embedding
    }
}

// Customer Used Service Edge - Direct connection with feedback
E::CustomerUsedService {
    From: CustomerPreferenceMemory,
    To: BusinessServiceMemory,
    Properties: {
        customer_id: String,       // Customer identifier
        service_id: String,       // Service identifier
        interaction_type: String, // "booked", "completed", "reviewed"
        satisfaction_rating: I32, // Rating (1-5 scale)
        timestamp: I64,           // When service was used
        duration_actual: I32,     // Actual duration in minutes
        cost_actual: F64,         // Actual cost paid
        currency: String,         // Currency
        outcome: String,          // Service outcome/result
        created_at: I64,          // Timestamp
        updated_at: I64,          // Last update timestamp
        text_feedback: String,    // Natural language feedback
                                 // Example: "Service was excellent but took longer than expected"
        embedding: [F32],         // Vector embedding of the feedback text
        embedding_model: String   // Model used for embedding
    }
}

// Customer Visited Location Edge - Direct connection with experience
E::CustomerVisitedLocation {
    From: CustomerBehaviorMemory,
    To: BusinessLocationMemory,
    Properties: {
        customer_id: String,       // Customer identifier
        location_id: String,      // Location identifier
        visit_type: String,       // "pickup", "browsing", "service_visit", "purchase"
        timestamp: I64,           // When visit occurred
        duration_minutes: I32,    // How long customer stayed
        party_size: I32,          // Number of people with customer
        purchase_made: Boolean,   // Whether purchase was made
        purchase_amount: F64,     // Amount spent
        currency: String,         // Currency
        created_at: I64,          // Timestamp
        updated_at: I64,          // Last update timestamp
        text_experience: String,  // Natural language experience description
                                 // Example: "Customer found location easy to access but parking was difficult"
        embedding: [F32],         // Vector embedding of the experience text
        embedding_model: String   // Model used for embedding
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


// Customer Contacted Business Edge - Communication interactions
E::CustomerContactedBusiness {
    From: CustomerBehaviorMemory,
    To: BusinessPolicyMemory,
    Properties: {
        customer_id: String,       // Customer identifier
        contact_method: String,   // "phone", "email", "chat", "social_media"
        contact_reason: String,   // "inquiry", "complaint", "feedback", "support"
        timestamp: I64,           // When contact occurred
        duration_seconds: I32,    // How long interaction lasted
        resolution_status: String,// "resolved", "pending", "escalated", "unresolved"
        agent_id: String,         // Agent/staff member who handled
        channel_details: String,  // Specific channel info (email address, phone number, etc.)
        created_at: I64,          // Timestamp
        updated_at: I64,          // Last update timestamp
        text_interaction: String, // Natural language interaction summary
                                 // Example: "Customer called about delivery delay, was understanding when explained"
        embedding: [F32],         // Vector embedding of the interaction text
        embedding_model: String   // Model used for embedding
    }
}

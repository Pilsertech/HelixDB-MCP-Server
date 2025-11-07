use serde_json::Value;
use crate::BusinessProductMemory;

pub trait RichTextGenerator {
    fn generate_category_context(&self, categories: &[String]) -> Vec<String> {
        let mut contexts = Vec::new();

        // Process each category as a general classification
        for cat in categories {
            // Add the primary category
            contexts.push(format!("Primary category: {}", cat));

            // Add it as a searchable term
            contexts.push(format!("Classification: {}", cat));
            
            // Add as a product/service group
            contexts.push(format!("Group: {}", cat));
            
            // Add as a business segment
            contexts.push(format!("Business segment: {}", cat));
            
            // If there's a hierarchy (categories separated by >), break it down
            if cat.contains('>') {
                let hierarchy: Vec<&str> = cat.split('>').map(str::trim).collect();
                for (i, &level) in hierarchy.iter().enumerate() {
                    contexts.push(format!("Level {} category: {}", i + 1, level));
                }
            }
        }
        
        contexts
    }

    fn generate_price_context(&self, price: f64, currency: &str) -> String {
        if price == 0.0 {
            return String::from("Price information not available");
        }

        let mut contexts = Vec::new();
        
        // Add basic price information
        contexts.push(format!("Price: {} {}", price, currency));
        
        // Add formatted price for better searchability
        contexts.push(format!("Cost: {:.2} {}", price, currency));
        
        // Add price range without specific thresholds
        contexts.push(format!("Price point: {:.2}", price));
        
        // Add pricing type indicators
        if price > 0.0 {
            contexts.push("Paid item/service".to_string());
        } else {
            contexts.push("Free item/service".to_string());
        }

        contexts.join(". ")
    }

    fn generate_availability_context(&self, availability: &str) -> String {
        if availability.is_empty() {
            return String::from("Availability information not provided");
        }

        let lower_availability = availability.to_lowercase();
        let mut contexts = Vec::new();

        // Add basic availability status
        contexts.push(format!("Status: {}", availability));

        // Add general availability context
        let (status_type, time_context) = match lower_availability.as_str() {
            s if s.contains("available") || s.contains("in stock") || s.contains("ready") => (
                "Currently available",
                "Ready for immediate access"
            ),
            s if s.contains("unavailable") || s.contains("out") => (
                "Temporarily unavailable",
                "Contact for expected availability"
            ),
            s if s.contains("coming") || s.contains("soon") || s.contains("pre") => (
                "Coming soon",
                "Advanced booking/reservation possible"
            ),
            s if s.contains("limited") => (
                "Limited availability",
                "Subject to capacity/stock constraints"
            ),
            s if s.contains("seasonal") => (
                "Seasonal availability",
                "Available during specific periods"
            ),
            s if s.contains("appointment") || s.contains("booking") => (
                "By appointment",
                "Requires advance scheduling"
            ),
            _ => (
                "Custom availability",
                "Contact for detailed availability information"
            )
        };

        contexts.push(status_type.to_string());
        contexts.push(time_context.to_string());

        // Add scheduling context if relevant
        if lower_availability.contains("schedule") || 
           lower_availability.contains("appointment") || 
           lower_availability.contains("booking") {
            contexts.push("Scheduling required".to_string());
            contexts.push("Time-based availability".to_string());
        }

        contexts.join(". ")
    }

    fn generate_feature_context(&self, features: &[String], specs: &str) -> String {
        let mut contexts = Vec::new();

        // Process features
        if !features.is_empty() {
            // Add general feature list
            contexts.push(format!("Features: {}", features.join(", ")));
            contexts.push(format!("Characteristics: {}", features.join(", ")));
            
            // Add each feature as a separate searchable term
            for feature in features {
                contexts.push(format!("Offers: {}", feature));
                contexts.push(format!("Includes: {}", feature));
            }
        }

        // Process specifications as generic key-value pairs
        if let Ok(specs_json) = serde_json::from_str::<Value>(specs) {
            if let Some(obj) = specs_json.as_object() {
                let mut details = Vec::new();
                
                for (key, value) in obj {
                    // Add each spec as a searchable attribute
                    details.push(format!("{}: {}", key, value));
                    contexts.push(format!("Attribute - {}: {}", key, value));
                    
                    // Add value as a standalone search term
                    contexts.push(format!("{}: {}", key, value));
                }

                if !details.is_empty() {
                    contexts.push(format!("Detailed Information: {}", details.join(", ")));
                }
            }
        }

        contexts.join(" ")
    }

    fn generate_brand_context(&self, brand: &str) -> Vec<String> {
        let mut contexts = Vec::new();

        // Add generic brand context
        contexts.extend(vec![
            format!("Brand name: {}", brand),
            format!("Provider: {}", brand),
            format!("Business name: {}", brand),
            format!("Merchant: {}", brand)
        ]);
        
        contexts
    }

    fn generate_customer_context(&self, description: &str, tags: &[String]) -> String {
        let mut contexts = Vec::new();

        // Process description for general context
        if !description.is_empty() {
            contexts.push(format!("Description: {}", description));
            contexts.push(format!("Details: {}", description));
        }

        // Process tags as general attributes
        if !tags.is_empty() {
            // Add all tags as searchable terms
            contexts.push(format!("Tags: {}", tags.join(", ")));
            
            // Add each tag individually for better search matching
            for tag in tags {
                contexts.push(format!("Tagged as: {}", tag));
                contexts.push(format!("Attribute: {}", tag));
                
                // Add classification terms
                contexts.push(format!("Category tag: {}", tag));
                contexts.push(format!("Classification: {}", tag));
            }
            
            // Add as combined metadata
            contexts.push(format!("Associated terms: {}", tags.join(", ")));
        }

        contexts.join(" ")
    }

    fn generate_seasonal_context(&self, trends: &str) -> String {
        if trends == "{}" {
            return String::new();
        }

        let mut contexts = Vec::new();

        if let Ok(json) = serde_json::from_str::<Value>(trends) {
            if let Some(obj) = json.as_object() {
                for (season, value) in obj {
                    // Add the period/season as a general time indicator
                    contexts.push(format!("Time period: {}", season));
                    contexts.push(format!("Period details: {}", value));
                    
                    // Add as a temporal classification
                    contexts.push(format!("Temporal category: {}", season));
                    
                    // Add the specific information
                    contexts.push(format!("{} period: {}", season, value));
                }
            }
        }

        contexts.join(" ")
    }

    fn generate_search_context(&self, item: &BusinessProductMemory) -> String {
        let mut contexts = Vec::new();

        // Add basic item/service info
        contexts.push(format!("Name: {}.", item.product_name));
        contexts.push(format!("Type: {}.", item.product_category));
        contexts.push(format!("Description: {}.", item.description));

        // Add provider/source context
        contexts.extend(self.generate_brand_context(&item.product_name));

        // Add pricing information
        contexts.push(self.generate_price_context(item.price, &item.currency));

        // Add availability/status information
        contexts.push(self.generate_availability_context(&item.availability));

        // Add characteristics and details
        contexts.push(self.generate_feature_context(&item.features, &item.specifications));

        // Add target audience/usage context
        contexts.push(self.generate_customer_context(&item.description, &item.tags));

        // Add temporal/periodic context
        contexts.push(self.generate_seasonal_context(&item.seasonal_trends));

        // Add market/competition context
        if !item.competitor_analysis.is_empty() {
            contexts.push(format!("Market context: {}.", item.competitor_analysis));
        }

        // Add related terms and keywords
        if !item.seo_keywords.is_empty() {
            contexts.push(format!("Related terms: {}.", item.seo_keywords.join(", ")));
        }

        // Combine all contexts with spaces
        contexts.join(" ")
    }

    fn generate_rich_composite_text(
        &self,
        name: &str,
        description: &str, 
        features: &[String],
        specs: &str,
        price: f64,
        currency: &str,
        availability: &str,
        tags: &[String],
        trends: &str
    ) -> String {
        let mut sections = Vec::new();

        // Add generic item/service information
        sections.push(format!("Name: {}", name));
        if !description.is_empty() {
            sections.push(format!("Description: {}", description));
        }

        // Add metadata contexts
        sections.extend(self.generate_brand_context(name));
        sections.push(self.generate_feature_context(features, specs));
        sections.push(self.generate_price_context(price, currency));
        sections.push(self.generate_availability_context(availability));
        sections.push(self.generate_customer_context(description, tags));
        sections.push(self.generate_seasonal_context(trends));

        // Filter out empty sections and join
        sections.into_iter()
               .filter(|s| !s.is_empty())
               .collect::<Vec<String>>()
               .join("\n\n")
    }
}
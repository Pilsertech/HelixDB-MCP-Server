use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Session state for managing query results pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySession {
    pub session_id: String,
    pub query: String,
    pub results: Vec<serde_json::Value>,
    pub cursor: usize,
    pub total_count: usize,
    pub created_at: std::time::SystemTime,
}

impl QuerySession {
    pub fn new(query: String, results: Vec<serde_json::Value>) -> Self {
        let total_count = results.len();
        Self {
            session_id: Uuid::new_v4().to_string(),
            query,
            results,
            cursor: 0,
            total_count,
            created_at: std::time::SystemTime::now(),
        }
    }
    
    /// Get next batch of results
    pub fn next(&mut self, limit: usize) -> Vec<serde_json::Value> {
        let start = self.cursor;
        let end = std::cmp::min(start + limit, self.results.len());
        
        let batch = self.results[start..end].to_vec();
        self.cursor = end;
        
        batch
    }
    
    /// Collect all remaining results
    pub fn collect_all(&mut self) -> Vec<serde_json::Value> {
        let remaining = self.results[self.cursor..].to_vec();
        self.cursor = self.results.len();
        remaining
    }
    
    /// Check if more results available
    pub fn has_more(&self) -> bool {
        self.cursor < self.results.len()
    }
}

/// Manages multiple query sessions
pub struct SessionManager {
    sessions: HashMap<String, QuerySession>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
    
    /// Create new session
    pub fn create_session(&mut self, query: String, results: Vec<serde_json::Value>) -> String {
        let session = QuerySession::new(query, results);
        let session_id = session.session_id.clone();
        self.sessions.insert(session_id.clone(), session);
        
        // Cleanup old sessions (keep last 100)
        if self.sessions.len() > 100 {
            self.cleanup_old_sessions();
        }
        
        session_id
    }
    
    /// Get session by ID
    pub fn get_session(&mut self, session_id: &str) -> Option<&mut QuerySession> {
        self.sessions.get_mut(session_id)
    }
    
    /// Remove session
    pub fn remove_session(&mut self, session_id: &str) -> bool {
        self.sessions.remove(session_id).is_some()
    }
    
    /// Cleanup sessions older than 1 hour
    fn cleanup_old_sessions(&mut self) {
        let now = std::time::SystemTime::now();
        let one_hour = std::time::Duration::from_secs(3600);
        
        self.sessions.retain(|_, session| {
            now.duration_since(session.created_at)
                .map(|duration| duration < one_hour)
                .unwrap_or(false)
        });
    }
}

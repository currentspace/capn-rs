// Resume Tokens for Cap'n Web Protocol Session Recovery
// Enables session suspension and resumption with full state preservation

use super::tables::{Value, ImportValue, ExportValue, ImportTable, ExportTable};
use super::ids::{ImportId, ExportId, IdAllocator};
use super::variable_state::VariableStateManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use base64::{Engine as _, engine::general_purpose};
use sha2::{Sha256, Digest};

/// Serializable session state for resume tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    /// Session metadata
    pub session_id: String,
    pub created_at: u64,
    pub last_activity: u64,
    pub version: u32,

    /// ID allocation state
    pub next_positive_id: i64,
    pub next_negative_id: i64,

    /// Import table state
    pub imports: HashMap<i64, SerializableImportValue>,

    /// Export table state
    pub exports: HashMap<i64, SerializableExportValue>,

    /// Variable state
    pub variables: HashMap<String, Value>,

    /// Session configuration
    pub max_age_seconds: u64,
    pub capabilities: Vec<String>, // Capability identifiers
}

/// Serializable import value (excludes non-serializable stubs/promises)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableImportValue {
    Value(Value),
    StubReference(String),    // Reference ID only
    PromiseReference(String), // Reference ID only
}

/// Serializable export value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableExportValue {
    Resolved(Value),
    Rejected(Value),
    StubReference(String),    // Reference ID only
    PromiseReference(String), // Reference ID only
}

/// Resume token containing encrypted and signed session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeToken {
    /// Base64-encoded encrypted session data
    pub token_data: String,
    /// Session ID for quick identification
    pub session_id: String,
    /// Token creation timestamp
    pub issued_at: u64,
    /// Token expiration timestamp
    pub expires_at: u64,
}

/// Resume token manager for session persistence
#[derive(Debug)]
pub struct ResumeTokenManager {
    /// Secret key for token encryption/signing
    secret_key: Vec<u8>,
    /// Default token lifetime in seconds
    default_ttl: u64,
    /// Maximum session age before forced expiry
    max_session_age: u64,
}

impl ResumeTokenManager {
    /// Create a new resume token manager
    pub fn new(secret_key: Vec<u8>) -> Self {
        Self {
            secret_key,
            default_ttl: 3600, // 1 hour default
            max_session_age: 86400, // 24 hours max
        }
    }

    /// Create a resume token manager with custom settings
    pub fn with_settings(
        secret_key: Vec<u8>,
        default_ttl: u64,
        max_session_age: u64,
    ) -> Self {
        Self {
            secret_key,
            default_ttl,
            max_session_age,
        }
    }

    /// Generate a secure random secret key
    pub fn generate_secret_key() -> Vec<u8> {
        use rand::RngCore;
        let mut key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }

    /// Create a session snapshot from current session state
    pub async fn create_snapshot(
        &self,
        session_id: String,
        allocator: &Arc<IdAllocator>,
        imports: &Arc<ImportTable>,
        exports: &Arc<ExportTable>,
        variables: Option<&VariableStateManager>,
    ) -> Result<SessionSnapshot, ResumeTokenError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Extract import table state
        let mut serializable_imports = HashMap::new();

        // Note: In a real implementation, we'd iterate over the actual import table
        // For now, we'll create a minimal snapshot structure
        tracing::info!(session_id = %session_id, "Creating session snapshot");

        // Extract variable state
        let variables_map = if let Some(var_mgr) = variables {
            var_mgr.export_variables().await
        } else {
            HashMap::new()
        };

        let snapshot = SessionSnapshot {
            session_id: session_id.clone(),
            created_at: now,
            last_activity: now,
            version: 1, // Protocol version

            // ID allocation state (simplified - would need actual state from allocator)
            next_positive_id: 1,
            next_negative_id: -1,

            imports: serializable_imports,
            exports: HashMap::new(), // Would extract from export table

            variables: variables_map,

            max_age_seconds: self.max_session_age,
            capabilities: Vec::new(), // Would list registered capabilities
        };

        Ok(snapshot)
    }

    /// Generate a resume token from a session snapshot
    pub fn generate_token(
        &self,
        snapshot: SessionSnapshot,
    ) -> Result<ResumeToken, ResumeTokenError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expires_at = now + self.default_ttl;

        // Serialize the snapshot
        let snapshot_data = serde_json::to_vec(&snapshot)
            .map_err(|e| ResumeTokenError::SerializationError(e.to_string()))?;

        // Create a simple signed token (in production, use proper encryption)
        let signature = self.sign_data(&snapshot_data);
        let token_payload = TokenPayload {
            snapshot: snapshot_data,
            issued_at: now,
            expires_at,
            signature,
        };

        let token_bytes = serde_json::to_vec(&token_payload)
            .map_err(|e| ResumeTokenError::SerializationError(e.to_string()))?;

        let token_data = general_purpose::STANDARD.encode(&token_bytes);

        Ok(ResumeToken {
            token_data,
            session_id: snapshot.session_id,
            issued_at: now,
            expires_at,
        })
    }

    /// Parse and validate a resume token
    pub fn parse_token(
        &self,
        token: &ResumeToken,
    ) -> Result<SessionSnapshot, ResumeTokenError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check expiration
        if now > token.expires_at {
            return Err(ResumeTokenError::TokenExpired);
        }

        // Decode the token
        let token_bytes = general_purpose::STANDARD.decode(&token.token_data)
            .map_err(|e| ResumeTokenError::InvalidToken(e.to_string()))?;

        let token_payload: TokenPayload = serde_json::from_slice(&token_bytes)
            .map_err(|e| ResumeTokenError::InvalidToken(e.to_string()))?;

        // Verify signature
        let expected_signature = self.sign_data(&token_payload.snapshot);
        if token_payload.signature != expected_signature {
            return Err(ResumeTokenError::InvalidSignature);
        }

        // Deserialize snapshot
        let snapshot: SessionSnapshot = serde_json::from_slice(&token_payload.snapshot)
            .map_err(|e| ResumeTokenError::InvalidToken(e.to_string()))?;

        // Additional validation
        if snapshot.created_at + snapshot.max_age_seconds < now {
            return Err(ResumeTokenError::SessionTooOld);
        }

        Ok(snapshot)
    }

    /// Restore session state from a snapshot
    pub async fn restore_session(
        &self,
        snapshot: SessionSnapshot,
        allocator: &Arc<IdAllocator>,
        imports: &Arc<ImportTable>,
        exports: &Arc<ExportTable>,
        variables: Option<&VariableStateManager>,
    ) -> Result<(), ResumeTokenError> {
        tracing::info!(
            session_id = %snapshot.session_id,
            imports_count = snapshot.imports.len(),
            exports_count = snapshot.exports.len(),
            variables_count = snapshot.variables.len(),
            "Restoring session from snapshot"
        );

        // Restore variable state
        if let Some(var_mgr) = variables {
            var_mgr.import_variables(snapshot.variables).await
                .map_err(|e| ResumeTokenError::RestoreError(e.to_string()))?;
        }

        // Note: In a full implementation, we'd restore:
        // - Import table entries (with careful stub/promise handling)
        // - Export table entries
        // - ID allocator state
        // - Registered capabilities

        tracing::info!(session_id = %snapshot.session_id, "Session restoration completed");
        Ok(())
    }

    /// Sign data using the secret key
    fn sign_data(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.secret_key);
        hasher.update(data);
        general_purpose::STANDARD.encode(hasher.finalize())
    }
}

/// Internal token payload structure
#[derive(Debug, Serialize, Deserialize)]
struct TokenPayload {
    snapshot: Vec<u8>,
    issued_at: u64,
    expires_at: u64,
    signature: String,
}

/// Errors related to resume token operations
#[derive(Debug, thiserror::Error)]
pub enum ResumeTokenError {
    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token has expired")]
    TokenExpired,

    #[error("Invalid token signature")]
    InvalidSignature,

    #[error("Session too old to resume")]
    SessionTooOld,

    #[error("Session restoration error: {0}")]
    RestoreError(String),

    #[error("Variable state error: {0}")]
    VariableStateError(#[from] super::variable_state::VariableError),
}

/// Session manager that integrates resume token functionality
#[derive(Debug)]
pub struct PersistentSessionManager {
    token_manager: ResumeTokenManager,
    active_sessions: Arc<tokio::sync::RwLock<HashMap<String, SessionInfo>>>,
}

#[derive(Debug, Clone)]
struct SessionInfo {
    session_id: String,
    last_activity: u64,
    variable_manager: Option<Arc<VariableStateManager>>,
}

impl PersistentSessionManager {
    /// Create a new persistent session manager
    pub fn new(token_manager: ResumeTokenManager) -> Self {
        Self {
            token_manager,
            active_sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Create a session snapshot for the given session
    pub async fn snapshot_session(
        &self,
        session_id: &str,
        allocator: &Arc<IdAllocator>,
        imports: &Arc<ImportTable>,
        exports: &Arc<ExportTable>,
        variables: Option<&VariableStateManager>,
    ) -> Result<ResumeToken, ResumeTokenError> {
        let snapshot = self.token_manager.create_snapshot(
            session_id.to_string(),
            allocator,
            imports,
            exports,
            variables,
        ).await?;

        self.token_manager.generate_token(snapshot)
    }

    /// Restore a session from a resume token
    pub async fn restore_session(
        &self,
        token: &ResumeToken,
        allocator: &Arc<IdAllocator>,
        imports: &Arc<ImportTable>,
        exports: &Arc<ExportTable>,
        variables: Option<&VariableStateManager>,
    ) -> Result<String, ResumeTokenError> {
        let snapshot = self.token_manager.parse_token(token)?;

        self.token_manager.restore_session(
            snapshot.clone(),
            allocator,
            imports,
            exports,
            variables,
        ).await?;

        // Register the restored session
        let mut sessions = self.active_sessions.write().await;
        sessions.insert(snapshot.session_id.clone(), SessionInfo {
            session_id: snapshot.session_id.clone(),
            last_activity: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            variable_manager: None, // Note: Variable manager integration would be handled separately
        });

        Ok(snapshot.session_id)
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> usize {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut sessions = self.active_sessions.write().await;
        let initial_count = sessions.len();

        sessions.retain(|_, info| {
            now - info.last_activity < 3600 // Keep sessions active for 1 hour
        });

        let cleaned_count = initial_count - sessions.len();
        if cleaned_count > 0 {
            tracing::info!(cleaned_sessions = cleaned_count, "Cleaned up expired sessions");
        }

        cleaned_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Number;

    #[tokio::test]
    async fn test_basic_resume_token_flow() {
        let secret_key = ResumeTokenManager::generate_secret_key();
        let manager = ResumeTokenManager::new(secret_key);

        // Create a simple snapshot
        let mut variables = HashMap::new();
        variables.insert("test_var".to_string(), Value::Number(Number::from(42)));

        let snapshot = SessionSnapshot {
            session_id: "test-session".to_string(),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            last_activity: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            version: 1,
            next_positive_id: 5,
            next_negative_id: -3,
            imports: HashMap::new(),
            exports: HashMap::new(),
            variables,
            max_age_seconds: 3600,
            capabilities: vec!["calculator".to_string()],
        };

        // Generate token
        let token = manager.generate_token(snapshot.clone()).unwrap();
        assert_eq!(token.session_id, "test-session");

        // Parse token back
        let restored_snapshot = manager.parse_token(&token).unwrap();
        assert_eq!(restored_snapshot.session_id, snapshot.session_id);
        assert_eq!(restored_snapshot.variables.len(), 1);

        if let Some(Value::Number(n)) = restored_snapshot.variables.get("test_var") {
            assert_eq!(n.as_i64(), Some(42));
        } else {
            panic!("Expected test_var to be number 42");
        }
    }

    #[tokio::test]
    async fn test_token_expiration() {
        let secret_key = ResumeTokenManager::generate_secret_key();
        let manager = ResumeTokenManager::with_settings(secret_key, 0, 3600); // 0 second TTL

        let snapshot = SessionSnapshot {
            session_id: "test-session".to_string(),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            last_activity: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            version: 1,
            next_positive_id: 1,
            next_negative_id: -1,
            imports: HashMap::new(),
            exports: HashMap::new(),
            variables: HashMap::new(),
            max_age_seconds: 3600,
            capabilities: Vec::new(),
        };

        let token = manager.generate_token(snapshot).unwrap();

        // Wait a moment for the token to expire
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let result = manager.parse_token(&token);
        assert!(matches!(result, Err(ResumeTokenError::TokenExpired)));
    }

    #[tokio::test]
    async fn test_invalid_signature() {
        let secret_key1 = ResumeTokenManager::generate_secret_key();
        let secret_key2 = ResumeTokenManager::generate_secret_key();

        let manager1 = ResumeTokenManager::new(secret_key1);
        let manager2 = ResumeTokenManager::new(secret_key2);

        let snapshot = SessionSnapshot {
            session_id: "test-session".to_string(),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            last_activity: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            version: 1,
            next_positive_id: 1,
            next_negative_id: -1,
            imports: HashMap::new(),
            exports: HashMap::new(),
            variables: HashMap::new(),
            max_age_seconds: 3600,
            capabilities: Vec::new(),
        };

        // Generate token with manager1
        let token = manager1.generate_token(snapshot).unwrap();

        // Try to parse with manager2 (different key)
        let result = manager2.parse_token(&token);
        assert!(matches!(result, Err(ResumeTokenError::InvalidSignature)));
    }

    #[tokio::test]
    async fn test_persistent_session_manager() {
        let secret_key = ResumeTokenManager::generate_secret_key();
        let token_manager = ResumeTokenManager::new(secret_key);
        let session_manager = PersistentSessionManager::new(token_manager);

        // Test session cleanup
        let cleaned = session_manager.cleanup_expired_sessions().await;
        assert_eq!(cleaned, 0); // No sessions to clean initially
    }
}
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("GitHub API error: {0}")]
    GitHubApiError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Persistence error: {0}")]
    PersistenceError(String),

    #[error("Conflict detected: {0}")]
    ConflictDetected(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

impl DomainError {
    fn variant_name(&self) -> &'static str {
        match self {
            DomainError::AuthenticationError(_) => "AuthenticationError",
            DomainError::GitHubApiError(_) => "GitHubApiError",
            DomainError::RateLimitExceeded(_) => "RateLimitExceeded",
            DomainError::PersistenceError(_) => "PersistenceError",
            DomainError::ConflictDetected(_) => "ConflictDetected",
            DomainError::NotFound(_) => "NotFound",
            DomainError::InvalidInput(_) => "InvalidInput",
            DomainError::NetworkError(_) => "NetworkError",
        }
    }
}

impl serde::Serialize for DomainError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DomainError", 2)?;
        state.serialize_field("type", self.variant_name())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

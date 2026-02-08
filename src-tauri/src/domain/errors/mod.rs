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

impl serde::Serialize for DomainError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

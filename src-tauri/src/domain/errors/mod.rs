use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("GitHub API error: {0}")]
    Api(String),

    #[error("Rate limited until {reset_at}")]
    RateLimited { reset_at: i64 },

    #[error("Persistence error: {0}")]
    Persistence(String),

    #[error("Conflict detected for operation {operation_id}: current={current_option_id}, expected={expected_option_id}")]
    Conflict {
        operation_id: String,
        current_option_id: String,
        current_option_name: String,
        expected_option_id: String,
    },

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Network error: {0}")]
    Network(String),
}

impl serde::Serialize for DomainError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

use serde::{Deserialize, Serialize};

/// GitHub ProjectV2 node ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub String);

impl ProjectId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ProjectId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ProjectId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// GitHub ProjectV2Item node ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

impl TaskId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for TaskId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for TaskId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// GitHub SingleSelectField node ID (Status field)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatusFieldId(pub String);

impl StatusFieldId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for StatusFieldId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for StatusFieldId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// GitHub SingleSelectOption node ID (Status option)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatusOptionId(pub String);

impl StatusOptionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for StatusOptionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for StatusOptionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Operation UUID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId(pub String);

impl OperationId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for OperationId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for OperationId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}


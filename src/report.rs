use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ToolError {
    pub code: &'static str,
    pub message: String,
}

impl ToolError {
    #[must_use]
    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_input",
            message: message.into(),
        }
    }

    #[must_use]
    pub fn resource_limit(message: impl Into<String>) -> Self {
        Self {
            code: "resource_limit",
            message: message.into(),
        }
    }

    #[must_use]
    pub fn io(message: impl Into<String>) -> Self {
        Self {
            code: "io_error",
            message: message.into(),
        }
    }

    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: "internal_invariant",
            message: message.into(),
        }
    }
}

impl Display for ToolError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ToolError {}

#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut result = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut result, "{byte:02x}").expect("writing to a String cannot fail");
    }
    result
}

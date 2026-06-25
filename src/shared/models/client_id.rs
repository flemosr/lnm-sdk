use std::fmt;

use serde::{Deserialize, Serialize, de};

use super::error::ClientIdValidationError;

/// A validated client identifier for trades and orders.
///
/// `ClientId` represents a user-provided identifier that can be attached to trades and orders for
/// tracking purposes. This type ensures that client IDs meet the required length constraints.
///
/// Client IDs must be:
/// + Non-empty strings (at least 1 character)
/// + At most 64 characters in length
///
/// # Examples
///
/// ```
/// use lnm_sdk::api_v3::models::ClientId;
///
/// // Create a client ID from a string
/// let client_id = ClientId::try_from("my-order-123").unwrap();
/// assert_eq!(client_id.as_str(), "my-order-123");
///
/// // Empty strings are invalid
/// assert!(ClientId::try_from("").is_err());
///
/// // Strings longer than 64 characters are invalid
/// let long_string = "a".repeat(65);
/// assert!(ClientId::try_from(long_string.as_str()).is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientId(String);

impl ClientId {
    /// The minimum allowed length for a client ID (1 character).
    pub const MIN_LEN: usize = 1;

    /// The maximum allowed length for a client ID (64 characters).
    pub const MAX_LEN: usize = 64;

    /// Returns the client ID as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::api_v3::models::ClientId;
    ///
    /// let client_id = ClientId::try_from("my-order-123").unwrap();
    /// assert_eq!(client_id.as_str(), "my-order-123");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the `ClientId` and returns the inner String.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::api_v3::models::ClientId;
    ///
    /// let client_id = ClientId::try_from("my-order-123").unwrap();
    /// let inner: String = client_id.into_inner();
    /// assert_eq!(inner, "my-order-123");
    /// ```
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl TryFrom<String> for ClientId {
    type Error = ClientIdValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() < Self::MIN_LEN {
            return Err(ClientIdValidationError::TooShort { len: value.len() });
        }

        if value.len() > Self::MAX_LEN {
            return Err(ClientIdValidationError::TooLong { len: value.len() });
        }

        Ok(ClientId(value))
    }
}

impl TryFrom<&str> for ClientId {
    type Error = ClientIdValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl From<ClientId> for String {
    fn from(value: ClientId) -> Self {
        value.0
    }
}

impl AsRef<str> for ClientId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for ClientId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ClientId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ClientId::try_from(s).map_err(|e| de::Error::custom(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_client_id() {
        let client_id = ClientId::try_from("my-order-123").unwrap();
        assert_eq!(client_id.as_str(), "my-order-123");
    }

    #[test]
    fn test_min_length_client_id() {
        let client_id = ClientId::try_from("a").unwrap();
        assert_eq!(client_id.as_str(), "a");
    }

    #[test]
    fn test_max_length_client_id() {
        let long_string = "a".repeat(64);
        let client_id = ClientId::try_from(long_string.as_str()).unwrap();
        assert_eq!(client_id.as_str(), long_string);
    }

    #[test]
    fn test_empty_client_id_fails() {
        let result = ClientId::try_from("");
        assert!(matches!(
            result,
            Err(ClientIdValidationError::TooShort { len: 0 })
        ));
    }

    #[test]
    fn test_too_long_client_id_fails() {
        let long_string = "a".repeat(65);
        let result = ClientId::try_from(long_string.as_str());
        assert!(matches!(
            result,
            Err(ClientIdValidationError::TooLong { len: 65 })
        ));
    }

    #[test]
    fn test_into_inner() {
        let client_id = ClientId::try_from("test-id").unwrap();
        let inner: String = client_id.into_inner();
        assert_eq!(inner, "test-id");
    }

    #[test]
    fn test_from_string() {
        let client_id = ClientId::try_from(String::from("my-order")).unwrap();
        assert_eq!(client_id.as_str(), "my-order");
    }

    #[test]
    fn test_display() {
        let client_id = ClientId::try_from("display-test").unwrap();
        assert_eq!(format!("{}", client_id), "display-test");
    }

    #[test]
    fn test_serialize() {
        let client_id = ClientId::try_from("serialize-test").unwrap();
        let json = serde_json::to_string(&client_id).unwrap();
        assert_eq!(json, "\"serialize-test\"");
    }

    #[test]
    fn test_deserialize() {
        let json = "\"deserialize-test\"";
        let client_id: ClientId = serde_json::from_str(json).unwrap();
        assert_eq!(client_id.as_str(), "deserialize-test");
    }

    #[test]
    fn test_deserialize_empty_fails() {
        let json = "\"\"";
        let result: Result<ClientId, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_too_long_fails() {
        let long_string = "a".repeat(65);
        let json = format!("\"{}\"", long_string);
        let result: Result<ClientId, _> = serde_json::from_str(&json);
        assert!(result.is_err());
    }
}

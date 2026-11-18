use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::borrow::Cow;

/// Represents the media type for problem details serialized as JSON.
pub const JSON: &str = "application/problem+json";

/// Represents a type of problem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProblemType {
    /// Gets the URI reference that identifies the problem type.
    pub uri: &'static str,

    /// Gets the short, human-readable summary of the problem type.
    pub title: &'static str,
}

impl ProblemType {
    /// Represents the problem type for an unsupported API version.
    pub const UNSUPPORTED: Self = Self {
        uri: "https://docs.api-versioning.org/problems#unsupported",
        title: "Unsupported API version",
    };

    /// Represents the problem type for an unspecified API version.
    pub const UNSPECIFIED: Self = Self {
        uri: "https://docs.api-versioning.org/problems#unspecified",
        title: "Unspecified API version",
    };

    /// Represents the problem type for an invalid API version.
    pub const INVALID: Self = Self {
        uri: "https://docs.api-versioning.org/problems#invalid",
        title: "Invalid API version",
    };

    /// Represents the problem type for an ambiguous API version.
    pub const AMBIGUOUS: Self = Self {
        uri: "https://docs.api-versioning.org/problems#ambiguous",
        title: "Ambiguous API version",
    };
}

/// Represents the details of a problem.
///
/// # Remarks
///
/// Members are serialized in the order `type`, `title`, `status`, `detail`, and `instance`, followed by any extension
/// members. Members without a value are not serialized.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProblemDetails {
    /// Gets or sets the URI reference that identifies the problem type, if any.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Cow<'static, str>>,

    /// Gets or sets the short, human-readable summary of the problem type, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<Cow<'static, str>>,

    /// Gets or sets the HTTP status code for this occurrence of the problem, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,

    /// Gets or sets the human-readable explanation specific to this occurrence of the problem, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    /// Gets or sets the URI reference that identifies this occurrence of the problem, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,

    /// Gets or sets the extension members of the problem.
    #[serde(flatten)]
    pub extensions: Map<String, Value>,
}

impl ProblemDetails {
    /// Initializes new [ProblemDetails] for the specified problem type.
    ///
    /// # Arguments
    ///
    /// * `kind` - the [type of problem](ProblemType)
    /// * `status` - the HTTP status code for this occurrence of the problem
    /// * `detail` - the explanation specific to this occurrence of the problem
    pub fn new(kind: ProblemType, status: u16, detail: impl Into<String>) -> Self {
        Self {
            r#type: Some(Cow::Borrowed(kind.uri)),
            title: Some(Cow::Borrowed(kind.title)),
            status: Some(status),
            detail: Some(detail.into()),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn new_should_set_standard_members() {
        // arrange
        let kind = ProblemType::UNSUPPORTED;

        // act
        let problem = ProblemDetails::new(kind, 400, "detail");

        // assert
        assert_eq!(problem.r#type.as_deref(), Some(kind.uri));
        assert_eq!(problem.title.as_deref(), Some(kind.title));
        assert_eq!(problem.status, Some(400));
        assert_eq!(problem.detail.as_deref(), Some("detail"));
        assert_eq!(problem.instance, None);
        assert!(problem.extensions.is_empty());
    }

    #[test]
    fn serialize_should_write_members_in_standard_order() {
        // arrange
        let mut problem = ProblemDetails::new(ProblemType::INVALID, 400, "detail");

        problem.instance = Some("/orders".into());
        problem.extensions.insert("requestId".into(), json!("42"));

        // act
        let json = serde_json::to_string(&problem).unwrap();

        // assert
        assert_eq!(
            json,
            r#"{"type":"https://docs.api-versioning.org/problems#invalid","title":"Invalid API version","status":400,"detail":"detail","instance":"/orders","requestId":"42"}"#
        );
    }

    #[test]
    fn serialize_should_skip_members_without_value() {
        // arrange
        let problem = ProblemDetails::default();

        // act
        let json = serde_json::to_string(&problem).unwrap();

        // assert
        assert_eq!(json, "{}");
    }

    #[test]
    fn deserialize_should_read_standard_and_extension_members() {
        // arrange
        let json = r#"{"type":"about:blank","title":"Not Found","status":404,"detail":"missing","code":42}"#;

        // act
        let problem: ProblemDetails = serde_json::from_str(json).unwrap();

        // assert
        assert_eq!(problem.r#type.as_deref(), Some("about:blank"));
        assert_eq!(problem.title.as_deref(), Some("Not Found"));
        assert_eq!(problem.status, Some(404));
        assert_eq!(problem.detail.as_deref(), Some("missing"));
        assert_eq!(problem.extensions["code"], json!(42));
    }
}

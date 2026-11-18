use api_versioning::visible_in_version;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

/// An RFC 3339 date and time.
///
/// # Remarks
///
/// This wrapper is required because `#[serde(with = "time::serde::rfc3339")]` cannot be directly applied when
/// `#[range]` is also used. The wire format doesn't change. `#[serde(transparent)] makes `Rfc3339` read and write
/// exactly like the inner value.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Rfc3339(#[serde(with = "time::serde::rfc3339")] OffsetDateTime);

impl Default for Rfc3339 {
    fn default() -> Self {
        Self(OffsetDateTime::now_utc())
    }
}

impl From<OffsetDateTime> for Rfc3339 {
    fn from(value: OffsetDateTime) -> Self {
        Self(value)
    }
}

impl AsRef<OffsetDateTime> for Rfc3339 {
    fn as_ref(&self) -> &OffsetDateTime {
        &self.0
    }
}

/// Represents an order.
#[visible_in_version]
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Gets or sets the unique identifier for the order.
    #[serde(default)]
    pub id: u32,

    /// Gets or sets the date and time when the order was created.
    #[serde(default = "OffsetDateTime::now_utc", with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub created_date: OffsetDateTime,

    /// Gets or sets the date and time when the order becomes effective.
    #[range(2.0..)]
    #[schema(value_type = String, format = DateTime)]
    pub effective_date: Rfc3339,

    /// Gets or sets the name of the ordering customer.
    pub customer: String,
}

impl Default for Order {
    fn default() -> Self {
        Self {
            id: 0,
            created_date: OffsetDateTime::now_utc(),
            effective_date: OffsetDateTime::now_utc().into(),
            customer: String::new(),
        }
    }
}

/// Represents a person.
#[visible_in_version]
#[derive(Default, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    /// Gets or sets the unique identifier for a person.
    #[serde(default)]
    pub id: u32,

    /// Gets or sets the first name of a person.
    pub first_name: String,

    /// Gets or sets the last name of a person.
    pub last_name: String,

    /// Gets or sets the email address for a person.
    #[range(2.0..)]
    pub email: String,

    /// Gets or sets the telephone number for a person.
    #[range(3.0..)]
    pub phone: String,
}

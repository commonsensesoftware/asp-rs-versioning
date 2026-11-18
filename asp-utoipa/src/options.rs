use api_versioning::{
    headers::Link,
    policy::{DeprecationPolicy, SunsetPolicy},
};
use std::time::SystemTime;
use time::OffsetDateTime;

/// Represents a function that creates a notice for a policy, if any.
pub type Notice<T> = Box<dyn Fn(&T) -> Option<String> + Send + Sync>;

/// Represents the options used to create OpenAPI documents.
///
/// # Remarks
///
/// These options only describe the documents. The API versioning [options](api_versioning::Options) of the router,
/// such as the API version reader and the default API version, are used as configured.
pub struct DocumentOptions {
    /// Gets or sets a value indicating whether the API version is substituted into a URL that contains the API version
    /// parameter, such as `/v1/orders` rather than `/v{api_version}/orders`. The default value is `true`.
    ///
    /// When the API version is substituted, the URL no longer has an API version parameter. A numeric API version whose
    /// minor version is `0` is substituted without it, so `1.0` becomes `/v1/orders`, while `1.1` and `2026-10-01` are
    /// substituted as is.
    pub substitute_api_version_in_url: bool,

    /// Gets or sets a value indicating whether API version parameters are added to version-neutral operations. The
    /// default value is `false`.
    ///
    /// A version-neutral operation does not require an API version, so the API version parameters are optional.
    pub add_api_version_parameters_when_version_neutral: bool,

    /// Gets or sets a value indicating whether the links of deprecation and sunset policies are hidden from the
    /// description of a document. The default value is `false`.
    ///
    /// Only links with the media type `text/html` are shown. The links are always included in the `x-api-versioning`
    /// extension.
    pub hide_policy_links: bool,

    /// Gets or sets the function that creates the deprecation notice for the description of a document.
    ///
    /// When an API version is deprecated without a deprecation policy, the notice is created for the default policy.
    /// No notice is added if the function returns [None]. The default notices are `The API is deprecated.`,
    /// `The API was deprecated on 2026-01-01.`, and `The API will be deprecated on 2026-01-01.`
    pub deprecation_notice: Notice<DeprecationPolicy>,

    /// Gets or sets the function that creates the sunset notice for the description of a document.
    ///
    /// No notice is added if the function returns [None]. The default notices are `The API was sunset on 2026-01-01.`
    /// and `The API will be sunset on 2026-01-01.`; there is no notice for a sunset policy without a date.
    pub sunset_notice: Notice<SunsetPolicy>,
}

impl Default for DocumentOptions {
    fn default() -> Self {
        Self {
            substitute_api_version_in_url: true,
            add_api_version_parameters_when_version_neutral: false,
            hide_policy_links: false,
            deprecation_notice: Box::new(|policy: &DeprecationPolicy| {
                Some(match policy.date() {
                    Some(date) => format!("The API {} deprecated on {}.", tense(date), format_date(date)),
                    None => "The API is deprecated.".into(),
                })
            }),
            sunset_notice: Box::new(|policy: &SunsetPolicy| {
                policy
                    .date()
                    .map(|date| format!("The API {} sunset on {}.", tense(date), format_date(date)))
            }),
        }
    }
}

/// Determines whether a link is rendered in the description of a document.
pub(crate) fn should_render(link: &Link) -> bool {
    link.media_type.eq_ignore_ascii_case("text/html")
}

fn tense(date: SystemTime) -> &'static str {
    if date < SystemTime::now() { "was" } else { "will be" }
}

/// Formats the date of a policy as an ISO 8601 calendar date in UTC, such as `2026-01-01`.
pub(crate) fn format_date(date: SystemTime) -> String {
    OffsetDateTime::from(date).date().to_string()
}

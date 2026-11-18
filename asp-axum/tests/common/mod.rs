// not every test uses every helper
#![allow(dead_code)]

use api_versioning::problem::ProblemDetails;

/// Gets the detail of the problem details in the specified response body.
pub fn detail(body: &str) -> anyhow::Result<String> {
    Ok(serde_json::from_str::<ProblemDetails>(body)?.detail.unwrap_or_default())
}

/// Gets the problem detail for an unsupported or invalid API version.
pub fn unsupported(path: &str, version: &str) -> String {
    format!("The HTTP resource that matches the request URI '{path}' does not support the API version '{version}'.")
}

/// Gets the problem detail for ambiguous API versions.
pub fn ambiguous(versions: &str) -> String {
    format!(
        "The following API versions were requested: {versions}. At most, only a single API version may be specified. \
         Please update the intended API version and retry the request."
    )
}

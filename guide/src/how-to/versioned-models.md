<!-- description: Use #[visible_in_version] to control which API versions a model appears in. -->

# Versioned Models

When an API is versioned, it is often necessary to version the models that are used in the API. This is especially true
when shared models are used in request and response messages.

The `#[visible_in_version]` attribute macro can be used to indicate which API versions a model is visible in. Each field
applies the `#[range]` attribute with a range of API versions to indicate which API versions the field is included.
When no attribute is applied, the field is visible in all APIs.

Consider the following model:

```rust
use api_versioning::visible_in_version;
use serde::{Deserialize, Serialize};

#[visible_in_version]
#[derive(Default, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(default)]
    pub id: u32,

    pub first_name: String,

    #[range(2.0..)]
    pub middle_name: String,

    pub last_name: String,

    #[range(2.0..)]
    pub email: String,

    #[range(3.0..)]
    pub phone: String,
}
```

The `Person` model indicates that:

- `id`, `first_name`, and `last_name` are visible in all API versions
- `middle_name` and `email` are only visible starting in API version `2.0`
- `phone` is only visible starting in API version `3.0`

Each value passed to the `[range]` attribute is a `Range` expression representing the lower and upper `Bounds` of the
range. The rule is parsed into a range that determines if the annotated field applies to an API version. Annotations
never define any API versions.

In rare cases, you might need a split range. The `#[range]` attribute supports multiple entries; for example:

```rust
use api_versioning::visible_in_version;
use serde::{Deserialize, Serialize};

#[visible_in_version]
#[derive(Default, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentalSettings {
    #[range(..2.0, 2.1..)]
    enabled: bool
}
```

These rules would express that `enabled` is included in every API version **except** `2.0`.

## Validation

When a versioned API receives a request the deserialization process will enforce that a client did not _over-post_
more data than is allowed for the requested API version, even if the backing model defines the corresponding field.
If a client attempts to post data that is not visible in the requested API version, the request will be rejected with
HTTP status code `400` (Bad Request). The response body will indicate which properties were not visible in the
requested API version. This is the same behavior as if the property did not exist on the model at all.

## OpenAPI

The OpenAPI extensions will leverage this information to generate version-specific OpenAPI documents with constrained
model fields. A client will not be able to tell whether you used a single model behind the scenes or many. From their
perspective, each model will appear to be unique with its own affinity the API version that defined it.

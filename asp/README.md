<!-- description: Install API Versioning. -->

# Overview

This crate provides the abstractions for API Versioning. Platform-specific crates such as `api-versioning-axum` build
on top of it. You can also use this library to apply versioning to your models without any web framework dependencies.

<!-- cargo-rdme start -->

## Features

| Feature             | Default | Description                                                                       |
| ------------------- | ------- | --------------------------------------------------------------------------------- |
| **http**            | yes     | HTTP readers, selectors, policies, headers, and reporting                         |
| **serde**           | yes     | (de)serializing an [ApiVersion] and the [`visibility`](https://docs.rs/api-versioning/latest/api_versioning/visibility/) of serialized members        |
| **macros**          | yes     | the `#[visible_in_version]` attribute; implies `serde`                            |
| **problem-details** | yes     | [problem details](https://docs.rs/api-versioning/latest/api_versioning/problem/) as defined by [RFC 9457]                               |

[ApiVersion], [`api_version!`](https://docs.rs/api-versioning/latest/api_versioning/macro.api_version.html), and models never require HTTP. A crate that only defines versioned models can use
`default-features = false` with the `macros` feature.

[ApiVersion]: https://docs.rs/api-versioning/latest/api_versioning/struct.ApiVersion.html
[RFC 9457]: https://www.rfc-editor.org/rfc/rfc9457.html

<!-- cargo-rdme end -->

## Getting Started

The simplest way to get started is to install the crate using the default features.

```bash
cargo add api-versioning
```

## Model Versioning

If you define your models in separate crates and you want to use versioning to control which API version ranges data
members appear in, you can reduce the features to: 

```bash
cargo add api-versioning --no-default-features --features macros
```

## Example

The following example demonstrates how to a define a model where some fields are only present in the wire format for
specific API version ranges. Conditional fields must implement `Default`.

```rust
use api_versioning::visible_in_version;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[visible_in_version]
#[derive(Default, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(default)]
    pub id: u32,

    pub first_name: String,

    pub last_name: String,

    // only applies in api versions >= 2.0
    #[range(2.0..)]
    pub email: String,

    // only applies in api versions >= 3.0
    #[range(3.0..)]
    pub phone: String,
}
```

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/LICENSE

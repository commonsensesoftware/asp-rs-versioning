<!-- description: Install API Versioning and explore versioned OpenAPI documents in a few lines of code. -->

<!-- cargo-rdme start -->

# Overview

This crate provides API Versioning support for OpenAPI documents with [utoipa].

A separate OpenAPI document is created for each API version declared by the APIs in a [Router]. Each document
contains only the operations that map to its API version, along with every version-neutral operation.

A handler is described by annotating it with `#[utoipa::path]` and adding it to an API with [`doc!`](https://docs.rs/api-versioning-utoipa/latest/api_versioning_utoipa/macro.doc.html). The path of the
operation comes from the router. [utoipa] requires the `path` argument of `#[utoipa::path]`, but its value is
ignored, so the convention is `path = ""`. The HTTP method is required and must match the method the handler is
added for. A handler that is not added with [`doc!`](https://docs.rs/api-versioning-utoipa/latest/api_versioning_utoipa/macro.doc.html) is not part of any document.

Each document:

- has the API version as its `info.version` and the [name] of the API version appended to its title, such as
  `Orders | v1`
- has a Markdown description with deprecation and sunset notices, followed by the links of their policies
- has the links of the deprecation and sunset policies in the `x-api-versioning` extension
- uses the logical name of each API, if any, as the tag of its operations, which describes the policies defined for
  the API by name and whether the API is deprecated when the API version is not deprecated by every API
- marks the operations that deprecate the API version as deprecated
- has the API version parameters defined by the configured API version reader, which are optional and default to
  the default API version in its document when an unspecified API version is assumed
- has the schemas referenced by its operations, which only have the members visible in the API version when they
  use `#[visible_in_version]`

The documents can be served as JSON with [routes], which serves each document at its [URL]. They can also be served
by any other means, such as [utoipa-scalar] or [utoipa-swagger-ui].

# API Versioning in Action

```rust
use api_versioning::{Options, api_version};
use api_versioning_axum::Router;
use api_versioning_utoipa::{doc, documents};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(info(title = "Example API", description = "An example API."))]
struct ApiDoc;

mod v1 {
    #[utoipa::path(
        get,
        path = "",
        summary = "Get Orders",
        description = "Retrieves all orders.",
        responses((status = 200, body = String)))]
    pub async fn get_orders() -> &'static str {
        "orders"
    }
}

mod v2 {
    #[utoipa::path(
        get,
        path = "",
        summary = "Get Orders",
        description = "Retrieves all orders.",
        responses((status = 200, body = String)))]
    pub async fn get_orders() -> &'static str {
        "orders, version 2"
    }
}

let (app, apis): (axum::Router, _) = Router::default()
    .api("/orders", |api| {
        api.name("Orders")
            .has([api_version!(1.0), api_version!(2.0)])
            .route("/", |e| e.get(doc!(v1::get_orders)).map(api_version!(2.0), |e| e.get(doc!(v2::get_orders))))
    })
    .split();
let documents = documents(&apis, ApiDoc::openapi());
```

[utoipa]: https://crates.io/crates/utoipa
[utoipa-scalar]: https://crates.io/crates/utoipa-scalar
[utoipa-swagger-ui]: https://crates.io/crates/utoipa-swagger-ui
[Router]: https://docs.rs/api-versioning-axum/latest/api_versioning_axum/struct.Router.html
[name]: https://docs.rs/api-versioning-utoipa/latest/api_versioning_utoipa/struct.ApiDocument.html#method.name
[routes]: https://docs.rs/api-versioning-utoipa/latest/api_versioning_utoipa/fn.routes.html
[URL]: https://docs.rs/api-versioning-utoipa/latest/api_versioning_utoipa/struct.ApiDocument.html#method.url

<!-- cargo-rdme end -->

## Examples

### Scalar

Provides an example using [Scalar](examples/scalar.rs).

```bash
cargo run --package api-versioning-utoipa --example scalar
```

### Swagger UI

Provides an example using the [Swagger UI](examples/swagger_ui.rs).

```bash
cargo run --package api-versioning-utoipa --example swagger_ui
```

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/LICENSE

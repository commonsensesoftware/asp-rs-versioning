<!-- description: Install API Versioning and stand up versioned endpoints in a few lines of code. -->

<!-- cargo-rdme start -->

# Overview

This crate provides API Versioning support for the [axum] web framework.

[axum] does not allow the same path and method to be registered more than once. API versioning allows exactly that,
so that all of the versioned handlers for a path and method are registered as a single [axum] route. When a request
is routed, the API version is read from the request and the handler mapped to that version is dispatched. API
version information is purely metadata attached to each handler at the time it is registered.

Versioned APIs are added to a [Router], which wraps an `axum` [Router](https://docs.rs/axum/latest/axum/routing/struct.Router.html). The router knows the complete
path and API version information of every endpoint, which it [describes] when it is [split].

## Getting Started

The simplest way to get started is to install the crate using the default features.

```bash
cargo add api-versioning-axum
```

## API Versioning in Action

API Versioning supports versioning by query string, media type, headers, a URL path segment, a custom method, or any
combination thereof. API versions that are not explicitly mapped to handlers are implicitly applied to all handlers
for that route. A deprecated API version is routable, but is expected to be sunset in the future. A version-neutral
handler will map to any defined API version, including none at all.

## Versioning by Query String

Versioning by query string is the default, recommended method of versioning APIs. It is typically the simplest and
most pragmatic method to use.

```rust
use api_versioning::{Options, api_version};
use api_versioning_axum::Router;
use axum::extract::Path;

async fn get_orders() -> &'static str {
    "orders"
}

async fn get_orders_v2() -> &'static str {
    "orders, version 2"
}

async fn cancel_order(Path(_): Path<u32>) {}

async fn ping() {}

let app: axum::Router = Router::default()
    .api("/orders", |api| {
        api.deprecates(api_version!(0.9))
            .has([api_version!(1.0), api_version!(2.0)])
            .route("/", |e| e.get(get_orders).map(api_version!(2.0), |e| e.get(get_orders_v2)))
            .route("/{id}", |e| e.neutral(|e| e.delete(cancel_order)))
    })
    .api("/ping", |api| api.neutral().route("/", |e| e.get(ping)))
    .into();
```

# Versioning by URL

The route template defines where an API version appears in a URL by capturing a parameter, which is named
`api_version` by default. Like any other parameter captured by [axum], it is part of the [Path](https://docs.rs/axum/latest/axum/extract/path/struct.Path.html)
of a route so it must be accounted for when a handler extracts other parameters. The requested [ApiVersion]
can also be extracted directly.

When the URL is the only source of an API version, a URL with a missing, invalid, or unsupported API version does
not identify a resource and results in HTTP status code 404; otherwise, the configurable status code 400 is used by
default.

> **NOTE**: The `v` prefix in the route template is optional and is never part of the API version itself.

```rust
use api_versioning::{ApiVersion, Options, api_version, readers::UrlReader};
use api_versioning_axum::Router;
use axum::extract::Path;

async fn get_order(Path((version, id)): Path<(ApiVersion<'static>, u32)>) -> String {
    format!("order {id} using API version {version}")
}

let app: axum::Router = Router::default()
    .api("/v{api_version}/orders", |api| api.has(api_version!(1.0)).route("/{id}", |e| e.get(get_order)))
    .into();
```
[axum]: https://crates.io/crates/axum
[ApiVersion]: https://docs.rs/api-versioning/latest/api_versioning/struct.ApiVersion.html
[Router]: https://docs.rs/api-versioning-axum/latest/api_versioning_axum/struct.Router.html
[split]: https://docs.rs/api-versioning-axum/latest/api_versioning_axum/struct.Router.html#method.split
[describes]: https://docs.rs/api-versioning-axum/latest/api_versioning_axum/struct.ApiDescriptions.html

<!-- cargo-rdme end -->

## Examples

### Basic

Provides an example with the [basic](examples/basic.rs) set of features and capabilities.

```bash
cargo run --package api-versioning-axum --example basic
```

Explore the example using [basic.http](examples/basic.http) with the [REST Client].

### Weather Forecast

Provides a simple [weather forecast](examples/forecast.rs) example that illustrates additional features.

```bash
cargo run --package api-versioning-axum --example forecast
```

Explore the example using [forecast.http](examples/forecast.http) with the [REST Client].

### By Module Convention

Provides an example that demonstrates deriving API versions from handler [module path](examples/by_module.rs) rather
than explicitly declaring API versions.

```bash
cargo run --package api-versioning-axum --example by_module
```

Explore the example using [by_module.http](examples/by_module.http) with the [REST Client].

[REST Client]: https://marketplace.visualstudio.com/items?itemName=humao.rest-client

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/LICENSE

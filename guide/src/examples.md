<!-- description: Runnable sample projects covering versioned models, endpoints, and OpenAPI. -->

# Examples

The following are complete, runnable sample projects. These can be run by cloning the repository or by creating a
Codespace.

## Basic

[![examples](https://img.shields.io/badge/example-code-2B91AF)](https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/asp-axum/examples/basic.rs)

Demonstrates the basic features and capabilities.

```bash
cargo run --package api-versioning-axum --example basic
```

Explore the example using [basic.http] with the [REST Client].

## Weather Forecast

[![examples](https://img.shields.io/badge/example-code-2B91AF)](https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/asp-axum/examples/forecast.rs)

A simple weather forecast service that illustrates additional features.

```bash
cargo run --package api-versioning-axum --example forecast
```

Explore the example using [forecast.http] with the [REST Client].

## Module Path Conventions

[![examples](https://img.shields.io/badge/example-code-2B91AF)](https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/asp-axum/examples/by_module.rs)

Demonstrates how to version services by deriving from the handler module path.

```bash
cargo run --package api-versioning-axum --example by_module
```

Explore the example using [by_module.http] with the [REST Client].

## OpenAPI

The following provide OpenAPI examples using [utoipa].

### Scalar

[![examples](https://img.shields.io/badge/example-code-2B91AF)](https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/asp-utoipa/examples/scalar.rs)

Demonstrates OpenAPI integration with Scalar using [utoipa-scalar].

```bash
cargo run --package api-versioning-utoipa --example scalar
```

### Swagger UI

[![examples](https://img.shields.io/badge/example-code-2B91AF)](https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/asp-utoipa/examples/swagger_ui.rs)

Demonstrates OpenAPI integration with the Swagger UI using [utoipa-swagger-ui].

```bash
cargo run --package api-versioning-utoipa --example swagger_ui
```

[basic.http]: https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/asp-axum/examples/basic.http
[forecast.http]: https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/asp-axum/examples/forecast.http
[by_module.http]: https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/asp-axum/examples/by_module.http
[utoipa]: https://crates.io/crates/utoipa
[utoipa-scalar]: https://crates.io/crates/utoipa-scalar
[utoipa-swagger-ui]: https://crates.io/crates/utoipa-swagger-ui
[REST Client]: https://marketplace.visualstudio.com/items?itemName=humao.rest-client

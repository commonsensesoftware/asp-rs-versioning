<img align="right" width="100px" src="logo.svg" /><br />

# API Versioning &emsp; ![CI][ci-badge] [![MIT licensed][mit-badge]][mit-url] 

[ci-badge]: https://github.com/commonsensesoftware/asp-rs-versioning/actions/workflows/ci.yml/badge.svg
[mit-badge]: https://img.shields.io/badge/license-MIT-blueviolet.svg
[mit-url]: https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/LICENSE

The _"Asp"_ project, more formally known as API Versioning, gives you a powerful, but easy-to-use method for adding API
versioning semantics to your new and existing REST services. The API versioning extensions define simple metadata
attributes and conventions that you use to describe which API versions are implemented by your services. You don't need
to learn any new routing concepts or change the way you implement your services today.

You may be looking for:

- [User Guide](https://commonsensesoftware.github.io/asp-rs-versioning)
- API Documentation
  - [API Versioning](https://docs.rs/api-versioning)
  - [API Versioning with Axum](https://docs.rs/api-versioning-axum)
  - [API Versioning with Utoipa](https://docs.rs/api-versioning-utoipa)
- [Release Notes](https://github.com/commonsensesoftware/asp-rs-versioning/releases)

The supported flavors are:

- **API Versioning**
  <div>Provides the API versioning abstractions for your data models and preferred web framework</div>

  [![crates.io](https://img.shields.io/crates/v/asp-rs-versioning.svg)](https://crates.io/crates/api-versioning)

- **API Versioning with Axum**
  <div>Adds API versioning to your [axum] applications</div>

  [![crates.io](https://img.shields.io/crates/v/asp-rs-versioning.svg)](https://crates.io/crates/api-versioning-axum)
  [![examples](https://img.shields.io/badge/example-code-2B91AF)](../../tree/main/asp-axum/examples)

- **API Versioning with Utoipa**
  <div>Adds API versioning support for OpenAPI with [utoipa]</div>

  [![crates.io](https://img.shields.io/crates/v/asp-rs-versioning.svg)](https://crates.io/crates/api-versioning-utoipa)
  [![examples](https://img.shields.io/badge/example-code-2B91AF)](../../tree/main/asp-utoipa/examples)

## Discussion

Have a general question, suggestion, or other feedback? Check out how you can [contribute](CONTRIBUTING.md).

## License

This project is licensed under the [MIT](LICENSE) license.

----
> If you are an existing user, please makes sure you review the [release notes](../../releases) between all major and minor package releases.

<div style="text-align:center;margin-top:32px;font-size:small">Logo by <a href="https://sacramento-design.com" target="_blank">Sacramento Design Works</a></div>

[axum]: https://crates.io/crates/axum
[utoipa]: https://crates.io/crates/utoipa

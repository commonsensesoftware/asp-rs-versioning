# Introduction

Versioning is an important aspect of any mature web service. This ensures that clients can rely on services to be stable
over time, while still enabling service changes and new features. The goal of the API Versioning project is to adhere to
the REST constraints by default, while using the HTTP stack of your choice; however, are numerous extensions and
customizations that allow you to version your APIs however you like.

## Crates

### [asp-versioning](https://crates.io/crates/api-versioning)

The core abstractions provide a common set of traits and types for API versioning across all supported platforms.
These capabilities can also be used to version your data models.

### [asp-versioning-axum](https://crates.io/crates/api-versioning-axum)

Everything you need to add service API versioning to your [axum] applications.

### [asp-versioning-utoipa](https://crates.io/crates/api-versioning-utoipa)

Every you need to document your services with OpenAPI using [utoipa].

## Contributing

`asp-rs-versioning` is free and open source. You can find the source code on [GitHub] and issues and feature requests
can be posted on the [GitHub issue tracker]. `asp-rs-versioning` relies on the community to fix bugs and add features.
If you'd like to contribute, please read the [CONTRIBUTING] guide and consider opening a [pull request].

## License

This project is licensed under the [MIT] license.

[axum]: https://crates.io/crates/axum
[utoipa]: https://crates.io/crates/utoipa
[GitHub]: https://github.com/commonsensesoftware/asp-rs-versioning
[GitHub issue tracker]: https://github.com/commonsensesoftware/asp-rs-versioning/issues
[CONTRIBUTING]: https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/CONTRIBUTING.md
[pull request]: https://github.com/commonsensesoftware/asp-rs-versioning/pulls
[MIT]: https://github.com/commonsensesoftware/asp-rs-versioning/blob/main/LICENSE

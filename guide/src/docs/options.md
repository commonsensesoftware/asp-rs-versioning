<!-- description: The DocumentOptions settings that describe each generated OpenAPI document. -->

# OpenAPI Options

The OpenAPI options allows you to configure, customize, and extend the default behaviors when you add OpenAPI support.

The `DocumentOptions` have the following configuration settings:

- [substitute_api_version_in_url](#substitute-in-url)
- [add_api_version_parameters_when_version_neutral](#add-parameter-when-version-neutral)
- [api_version_parameter_description](#default-api-version-parameter-description)
- [hide_policy_links](#hide-policy-links)
- [deprecation_notice](#deprecation)
- [sunset_notice](#sunset)

### Substitute in URL

This option will substitute API version parameters that are in the route template with the corresponding API version
value. When an API version parameter value is substituted, that parameter is also removed from the parameters associated
with the API description. This option is useful for service authors that version by URL segment and wants the API
version value automatically populated. For example, the route template `api/v{api_version}/resource` for API version
1.0 will become `api/v1/resource` and the API version parameter will be removed. The default value is `true`.

### Add Parameter When Version-Neutral

This option let's you define whether an API version parameter is generated for version-neutral APIs. A version-neutral
API does not require an API version; however, you may not want a client to know the API is version-neutral. By setting
`add_api_version_parameters_when_version_neutral` to `true`, an API version parameter will be documented, even though
it is not required. The default value is `false`.

### Default API Version Parameter Description

This option defines the description of a generated API version parameter. The default value is
`"The requested API version"`. The description is only applied to a parameter that does not already have one. If an
operation declares the API version parameter with a description, such as through `#[utoipa::path(params(...))]`, then
that description is retained.

> [!NOTE]
> A description that is not known until runtime, such as one read from configuration, can be provided once at startup
> with `String::leak`.

```rust
use api_versioning_utoipa::DocumentOptions;

let options = DocumentOptions {
    api_version_parameter_description: "The API version of the request",
    ..Default::default()
};
```

### Hide Policy Links

This option controls whether deprecation or sunset policy links are displayed. The default value is `false`, which
means any defined policies links are displayed as bulleted list of hyperlinks. Policy links rendered for user interface
purposes must have the link type `text/html`.

Policies may define other types of links, but these are not shown in the user interface. These links are rendered in
OpenAPI documents as the `x-api-versioning` document extension.

### Deprecation

If an API has an applicable deprecation policy, this setting defines the callback function used to generate the message
text based on the provided policy. The default setting returns a generic message in English indicating that the API is
deprecated if no date is specified or a date-specific message in English when the API became, or will become,
deprecated.

### Sunset

If an API has an applicable sunset policy, this setting defines the callback function used to generate the message text
based on the provided policy. If the policy does not have a date, then no message is generated; otherwise, a
date-specific message in English is generated indicating when the API became, or will become, sunset. A sunset policy is
for an API that is sunset and will no longer exist. The effective sunset policy date should always be in the future as
there should be no information about the API after it is sunset.

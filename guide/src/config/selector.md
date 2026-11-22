<!-- description: Choose the API version for a request that did not specify one. -->

# API Version Selector

The `Selector` trait defines the behavior of how an API version is selected for a given request context. This service is
typically only used when a client has not requested an explicit API version and the
`assume_default_version_when_unspecified` option is enabled. The role of the API version selector is to select the
appropriate API version given the current request and a model of available API versions.

> [!NOTE]
> Although the `Selector` can be used for other scenarios, it is currently only utilized when no API version is
> requested by a client and the server allows this behavior. The selector provides the rules that selects the most
> appropriate API version according to the server. There is no built-in capability to ignore an API version explicitly
> requested by a client.

There are several API version selectors provided out-of-the-box or you can implement your own. The default, configured
API version selector is `ConstantSelector`. The selector can be configured with:

```rust
let mut options = Options::default();

options.selector = Some(Box::new(ConstantSelector::new(api_version!(2022-01-01))));
```

## Constant

The `ConstantSelector` always selects a user-defined API version, regardless of the request or available API version
information. The default behavior uses `Options::default_version`.

## Current

The `CurrentVersionSelector` selects the maximum API version available which does not have a version status. If no match
is found, it falls back to the configured `Options::default_version`. An an example, if the versions `1.0`,
`2.0`, and `3.0-alpha` are available, then `2.0` will be selected because it's the highest, implemented or released API
version.

## Lowest

The `LowestVersionSelector` selects the minimum API version available which does not have a version status. If no match
is found, it falls back to the configured `Options::default_version`. As an example, if the versions
`0.9-beta`, `1.0`, `2.0`, and `3.0-alpha` are available, then `1.0` will be selected because it's the lowest,
implemented or released API version. Your endpoints must declare one or more API versions for the selector to work
effectively or it will always select the configured `Options::default_version`.

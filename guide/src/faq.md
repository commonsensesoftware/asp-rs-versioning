<!-- description: Common questions about API versioning. -->

# FAQ

## What is the difference between the `default_version` and `selector` options?

There are subtle differences between these two [options]. Typically, you only need to configure one or the other, but
not both.

The `default_version` has the following uses:

- The default or initial API version shared by all services
- The fallback API version used when no other API version can be resolved

The `default_version` value is `1.0`, but that may not be your starting API version. For example, you might use the
date-only API versioning scheme. This configuration option prevents the value from being hard-coded and makes it easy
to change the API version for your initial set of services.

The `selector` option has a similar, but different purpose. Any implementation of the [Selector] is used to select the
best API version given the current HTTP request and API version model. The provided API version model will already be
aggregated across all known service versions.

While this component could be used for a number of different purposes, it is currently only used to select the API
version that should be used when a client does not provide an API version. This option is thus only used when the
`assume_default_version_when_unspecified` option is also `true`. The default, configured value for this option is a
instance of the `ConstantSelector`, which always returns the value of `default_version`. Other [Selector]
implementations use the `default_version` as the final fallback value.

It's recommended that the [Selector] implementation you use provides stable, deterministic results. This is particularly
important for existing clients that may not be aware that you have introduced API versioning. Contrary to this guidance,
a number of service authors have requested granular control over how the API versions should be selected. As an example,
a service author might want to allow a client to never specify an API version and use an internal _client-to-version_
mapping that is maintained on the server after the first client connects. How this is implemented in a [Selector] is up
to the service author, but it likely requires information from the current HTTP request and the available API versions
for a service.

[options]: config/options.md
[Selector]: config/selector.md
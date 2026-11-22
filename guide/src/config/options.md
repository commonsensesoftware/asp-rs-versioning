<!-- description: The API Versioning settings: readers, selectors, defaults, reporting, and policies. -->

# API Versioning Options

The API Versioning options allow you to configure, customize, and extend the default behaviors when you add API
versioning to your application.

`Options` has the following configuration settings:

- [reader](reader.md)
- [selector][Selector]
- [default_version](#default-api-version)
- [report_versions](#report-api-versions)
- [policies]
- [unsupported_status_code](#unsupported-api-version-status-code)
- [assume_default_version_when_unspecified](#assume-default-version-when-unspecified)

## Default API Version

This option defines the default `ApiVersion` for the application. This is useful for services that use implicit API
versioning in their initial release. The configured, default value is `1.0`.

```rust
use api_versioning::{Options, api_version};

let mut options = Options::default();

options.default_versio = api_version!(2.0);
```

## Report API Versions

This option enables sending the API versioning information in HTTP header in responses. The default header names are
`api-supported-versions` and `api-deprecated-versions`. If there are any deprecation or sunset [policies](#policies)
defined, they will also be included in the response headers. This option is disabled by default.

```rust
use api_versioning::{Options, api_version};

let mut options = Options::default();

options.report_versions.enabled = true;
```

The default HTTP header names can be changed as follows:

```rust
use api_versioning::{Options, api_version};

let mut options = Options::default();

options.report_versions.enabled = true;
options.report_versions.header.supported = "supported-versions";
options.report_versions.header.deprecated = "deprecated-versions";
```

## Policies

This option allows you to define [API versioning policies][policies]. This is primarily used to define _deprecation_ and
_sunset_ policies about when an API. Related links, such as to a public policy web page, can also be reported that may
be useful to clients for more information about your API policies.

## Unsupported API Version Status Code

This option allows you to configure the HTTP status code used when an unsupported API version is requested. The default
value is `400` (Bad Request).

While any HTTP status code can be used, the following are the most sensible:

| Status Code | Meaning         | Description                          |
| ----------- | --------------  | ------------------------------------ |
| 400         | Bad Request     | The API doesn't support this version |
| 404         | Not Found       | The API doesn't exist                |
| 501         | Not Implemented | The API isn't implemented            |

### Remarks

Regardless of the configured option, when versioning by:

- URL segment, `404` is always returned
- media type, `406` or `415` is always returned

## Assume Default Version When Unspecified

This option enables support for clients to make requests with implicit API versioning. This option is disabled by
default, which means that all clients must send requests with an explicit API version. Services will respond to client
requests that do not specify an API version with either HTTP status code `400` (Bad Request) or HTTP status code `404`
(Not Found), depending whether the requested route exists.

This option should only be enabled when supporting legacy services that did not previously support API versioning.
Forcing existing clients to specify an explicit API version for an existing service introduces a breaking change.
Conceptually, clients in this situation are bound to some API version of a service, but they don't know what it is and
never explicit request it.

When this option is enabled, clients will be able to make a request without specifying a specific API version. The API
version of the service that is selected will be based on the configured [Selector]. The default behavior always matches
the [default_version](#default-api-version).

[policies]: ../version-policies.md
[Selector]: selector.md

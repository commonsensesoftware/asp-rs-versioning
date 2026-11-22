<!-- description: The RFC 9457 problem details returned for API version errors. -->

# Error Responses

There are several built-in error responses. The body of each error response complies with [RFC 9457] _Problem Details_.

### Unspecified

All versioned services require that an API version be specified. When a client makes a request without providing an API
version, then the server will respond with a bad request. This behavior is typically not exhibited when the API is
version-neutral or the `assume_default_version_when_unspecified` option is configured to true.

| | |
| - | - |
| **Title**  | Unspecified API version |
| **Type**   | https://docs.api-versioning.org/problems#unspecified |
| **Status** | 400 |
| **Detail** | An API version is required, but was not specified |
| **Code**   | ApiVersionUnspecified |

### Unsupported

When a client requested API version does not match any of the defined handlers, then the server will respond with a
problem. If the `report_versions` option is true, then the supported versions will be returned to the client in the
`api-supported-versions` HTTP header.

| | |
| - | - |
| **Title**  | Unsupported API version |
| **Type**   | https://docs.api-versioning.org/problems#unsupported |
| **Status** | 400<sup>1</sup> <sup>2</sup> |
| **Detail** | The specified API version is not supported |
| **Code**   | UnsupportedApiVersion |

> <sub>1: Defined by `Options::unsupported_status_code`</sub><br/>
> <sub>2: The value is always `404` when versioning by URL segment</sub>

### Invalid

When a client makes a request with an API version, but the value is malformed or cannot be parsed, then the server will
respond with a bad request. This typically occurs where the value contains incomplete version components or the
date-only form is invalid (ex: 2016-02-30).

| | |
| - | - |
| **Title**  | Invalid API version |
| **Type**   | https://docs.api-versioning.org/problems#invalid |
| **Status** | 400 |
| **Detail** | An API version was specified, but it is invalid |
| **Code**   | InvalidApiVersion |

### Ambiguous

When a client requests a specific API version, the specified API version must be unambiguous to the server. A client is
allowed to specify an API version more than once, but if the values are not identical, then the server will respond
with a bad request.

| | |
| - | - |
| **Title**  | Ambiguous API version |
| **Type**   | https://docs.api-versioning.org/problems#ambiguous |
| **Status** | 400 |
| **Detail** | An API version was specified multiple times with different values |
| **Code**   | AmbiguousApiVersion |

#### Examples

```http
GET /resource?api-version=1.0 HTTP/2
api-version: 1.0
```
_Figure 1: Multiple, unambiguous API versions requested_

```http
GET /resource?api-version=1.0 HTTP/2
api-version: 2.0
```
_Figure 2: Ambiguous API versions requested between in query string and headers_

```http
GET /resource?api-version=1.0&api-version=2.0 HTTP/2
```
_Figure 3: Ambiguous API versions requested in the query string_

```http
GET /resource HTTP/2
api-version: 1.0
api-version: 2.0
```
_Figure 4: Ambiguous API versions requested in the headers_

## Customization

Error responses can be customized or extended in several ways.

## No Error

To remove errors and only respond with a status code, use `default-features = false` and omit the **problem-details**
feature.

## Custom Error

You can provide your own custom errors by implementing a custom `error::Responder` and wire it up to the `Router`. A
built-in conversion is also support for a compatible function or closure.

```rust
use api_versioning_axum::{Router, error::Context};

let router = Router::default().with_error_responder(|ctx: &Context<'_>| {
    (ctx.status(), ctx.error().to_string()).into_response()
});
```

[RFC 9457]: https://www.rfc-editor.org/rfc/rfc9457.html

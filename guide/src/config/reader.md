<!-- description: Read the requested API version from the query string, header, media type, or URL segment. -->

# API Version Reader

The `Reader` trait defines the behavior of how an API version is read in its raw, unparsed form from the current HTTP
request. There are multiple methods for reading an API version provided out-of-the-box or you can implement your own.
The default, configured API version reader is a composed instance `QueryStringReader` and `UrlReader`.

## Query String

The `QueryStringReader` reads the requested API version from the requested query string. The default query string
parameter name is **api-version**. `QueryStringReader::new` accepts the name of a query string parameter so that an
alternate query string parameter can be used. It also accepts an array or `Vec` of parameter names if you support more
than one, which might be the case for legacy reasons.

```rust
use api_versioning::{Options, readers::QueryStringReader};

let mut options = Options::default();

// svc?api-version=2.0
options.reader = Box::new(QueryStringReader::default());
```

```rust
use api_versioning::{Options, readers::QueryStringReader};

let mut options = Options::default();

// svc?v=2.0
options.reader = Box::new(QueryStringReader::new("v"));
```

## Media Type

The `MediaTypeReader` reads the requested API version from a HTTP media type request header. The supported headers are **Content-Type** and **Accept**. If both headers are present, then **Content-Type** is preferred. If the **Accept**
header specifies qualities, then the API version associated with the highest quality is selected. This behavior is
independent of media type negotiation. The default media type parameter is `"v"`, but you may specify an alternate name.

```rust
use api_versioning::{Options, readers::MediaTypeReader};

let mut options = Options::default();

// Content-Type: application/json;v=2.0
options.reader = Box::new(MediaTypeReader::default());
```

```rust
use api_versioning::{Options, readers::MediaTypeReader};

let mut options = Options::default();

// Content-Type: application/json;version=2.0
options.reader = Box::new(MediaTypeReader::new("version"));
```

The `MediaTypeReader` also supports reading an API version from the MIME type rather than a media type parameter. This
approach requires using the `template!` macro which allows you to specify any pattern you like, but it must match
exactly one segment with a name of your choosing.

```rust
use api_versioning::{Options, pattern, readers::MediaTypeReader};

let mut options = Options::default();

// Accept: application/vnd.my.company.v1+json
options.reader = Box::new(MediaTypeReader::template(pattern!(application/vnd.{v}+json)));
```

## Header

The `HeaderReader` reads the requested API version from a HTTP request header. There is no default or standard HTTP
header. You must use `HeaderReader::new` to define which HTTP header name, or names, contain the API version
information.

```rust
use api_versioning::{Options, readers::HeaderReader};

let mut options = Options::default();

// api-version: 1.0
options.reader = Box::new(HeaderReader::new("api-version"));
```

## URL Path Segment

The `UrlReader` reads the requested API version from a URL path segment. The name provided is the name that matches the
parameter name you specified in your route templates. The default name is `api_version`.

```rust
use api_versioning::{Options, readers::UrlReader};

let mut options = Options::default();

// v{api_version}/svc
options.reader = Box::new(UrlReader::default());
```

> [!WARNING]
> This method of API versioning violates the REST _Uniform Interface_ constraint. If you're creating a new API, consider
> using query string or media type versioning instead.

## Composition

Multiple `Reader` implementations can be combined using composition. For convenience, the `Combine` trait provides
ergonomic composition through the library prelude to compose multiple API version reading styles.

```rust
use api_versioning::{Options, prelude::*};
use api_versioning::readers::{HeaderReader, QueryStringReader};

let mut options = Options::default();

options.reader = Box::new(QueryStringReader::default().combine(HeaderReader::new("x-api-version")));
```

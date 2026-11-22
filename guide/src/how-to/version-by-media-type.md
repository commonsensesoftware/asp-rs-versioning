<!-- description: Version an API with a media type parameter through content negotiation. -->

# Media Type Versioning

Content negotiation is the defined method in REST for reasoning about the content expectations between a client and
server. The parameters used in media types for content negotiation can contain custom input that can be used to drive
API versioning.

## Example

The following example updates the configuration to read API versions from a media type.

```rust
use api_versioning::{Options, api_version};
use api_versioning::readers::MediaTypeReader;
use api_versioning_axum::Router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // note: this is equivalent to MediaTypeReader::default()
    let options = Options {
        reader: Box::new(MediaTypeReader::new("v")),
        ..Default::default()
    };
    let app: axum::Router = Router::new(options)
        .api("/helloworld", |api| {
            api.has([api_version!(1.0), api_version!(2.0)])
               .route("/", |e| {
                 e.get(|| async { "Hello world!" })
                  .map(api_version!(2.0), |h| h.post(|body: String| body))
                  
            })
        })
        .into();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}
```

## Configuration

The `MediaTypeReader::default()` uses the media type parameter name `v`, but you can specify any name you like. The
default behavior will require that clients always specify an API version, so service authors will likely want their
configuration to be:

```rust
use api_versioning::{Options, api_version};
use api_versioning::readers::MediaTypeReader;

let options = Options {
    reader: Box::new(MediaTypeReader::default()),
    assume_default_version_when_unspecified: true,
    selector: Box::new(CurrentSelector::new(api_version!(1.0))),
    ..Default::default()
};
```

This will allow clients to request a specific API version by media type, but if they don't specify anything, they will
receive the current implementation (e.g. API version). For example:

```http
GET api/helloworld HTTP/2
```
_Figure 1: returns the result from API version 2.0 because it's the current version_

```http
GET api/helloworld HTTP/2
accept: text/plain;v=1.0
```
_Figure 2: returns the result from API version 1.0_

```http
POST api/helloworld HTTP/2
content-type: text/plain;v=2.0
content-length: 12

Hello there!
```
_Figure 3: explicitly posts the content to API version 2.0, even though it would be implicitly matched_

## Multiple Media Types

The `MediaTypeReader` matches the configured media type parameter of **any** incoming request. This might be undesirable
if you support multiple media types or there is ambiguity in matching a media type.

Consider the following request:

```http
GET api/helloworld HTTP/2
accept: application/json;v=1.0;q=0.8,application/signed-exchange;v=b3;q=0.9
```

In this scenario, a client has specified multiple media types and they both have the media type parameter `v`. The
`MediaTypeReader` will honor quality (e.g. `q`) when specified. If multiple media types have the same quality, the first
one is selected. In this example `application/signed-exchange` is selected because it has the highest quality. When the
`v` parameter is parsed, the value `b3` is not a valid API version and will return HTTP status code `406` (Not
Acceptable).

Additional capabilities allow building media type matching rules that enable to you configure how you would like things
to match. You can specify and combine any of the following behaviors:

- Define multiple media type parameters
- Match media types by template
- Disambiguate between multiple API versions

## Custom Media Types

Defining new, custom media types (ex: `application/vnd.my.company.1+json`) to drive API versioning is another variant of
this approach that is compliant with the constraints of REST. `MediaTypeReader` and the `pattern!` macro provide way
to support it.

The most natural approach is to a use a template to match an API version in the media type. The specified template uses
syntax and matching analogous to route templates.

```rust
use api_versioning::{Options, api_version};
use api_versioning::readers::{MediaTypeReader, pattern};

let options = Options {
    reader: Box::new(MediaTypeReader::template(pattern!(application/vnd.my.company.{version}+json))),
    ..Default::default()
};
```

This allows matching the API version the same way as if it were in a URL segment. All of the same format and parsing
rules apply. In most cases, this is sufficient; however, the template expects **exactly one** parameter and that will be
assumed to the API version parameter.

```http
GET api/helloworld HTTP/2
accept: application/vnd.my.company.v1+json
```


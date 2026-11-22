<!-- description: Configure API versioning. -->

# Configuring Your Application

Careful consideration was taken to make the API versioning configuration defaults sensible and as simple as possible. In
most cases, you do not need to configure anything and `Options::default()` will suffice.

The two most common configuration changes are updating the `Selector` and enable reporting API versions, which is
disabled by default. The default `Selector` configuration combines the `QueryStringReader` and `UrlReader` to match
the most common versioning methods. Aligning to a specific method will remove any unexpected behaviors and improve
runtime performance.

```rust
use api_versioning::Options;
use api_versioning::readers::QueryStringReader;

let mut options = Options::default();

options.report_versions.enabled = true;
options.reader = Box::new(QueryStringReader::default());
```

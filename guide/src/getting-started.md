<!-- description: Install API Versioning and stand up a versioned Hello World endpoint in a few lines of code. -->

# Getting Started

The simplest way to get started is to install the crate using the default features with `axum`.

```bash
cargo add api-versioning-axum
```

## API Versioning in Action

The following example sets up the ubiquitous "Hello World" service with two versions of the same endpoint. The `version`
parameter resolves to the requested API version and echoes it back to illustrate different endpoints were reached.



## Examples
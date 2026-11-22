<!-- description: Document a versioned service with OpenAPI. -->

# API Documentation

Adding documentation is often the final, pivotal step in making your versioned services available to clients and fosters
their utilization. While there are many approaches to documenting your services, OpenAPI (formerly Swagger) has quickly
become the de facto method for describing REST services.

The API versioning provides several extensions that make it easy to add versioning into your OpenAPI configurations,
which do most of the heavy lifting to discover and collate your REST services by API version. [utoipa] is currently the
only supported OpenAPI generator.

[utoipa]: https://crates.io/crates/utoipa

## Example

The following is a simple, versioned service for people. The `email` field only appears after version 2.0 and the `phone`
field only appears after version 3.0.

> [!NOTE]
> The `utoipa::path` attribute macro requires the `path` parameter, but this will automatically be derived from
> the corresponding versioned route group. `path = ""` is specified, but is not used. If an explicit path is provided it
> will be ignored.

```rust
use api_versioning::{ApiVersion, api_version, visible_in_version};
use api_versioning_axum::{Router, VersionedApi};
use api_versioning_utoipa::{ApiDocument, doc, documents};
use axum::{Json, extract::Path};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(info(title = "Example API", description = "An example API."))]
struct ApiDoc;

/// Represents a person.
#[visible_in_version]
#[derive(Default, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    /// Gets or sets the unique identifier for a person.
    #[serde(default)]
    pub id: u32,

    /// Gets or sets the first name of a person.
    pub first_name: String,

    /// Gets or sets the last name of a person.
    pub last_name: String,

    /// Gets or sets the email address for a person.
    #[range(2.0..)]
    pub email: String,

    /// Gets or sets the telephone number for a person.
    #[range(3.0..)]
    pub phone: String,
}

pub mod v1 {
    use super::*;

    pub mod people {
        use super::*;

        #[utoipa::path(
            get,
            path = "",
            summary = "Get Person",
            description = "Gets a single person.",
            params(
                ("id" = u32, Path, description = "The requested person identifier.")
            ),
            responses(
                (status = 200, description = "The person was successfully retrieved.", body = [Person]),
                (status = 404, description = "The person does not exist.")
            ))]
        pub async fn by_id(Path(id): Path<u32>) -> Json<Person> {
            Json(Person {
                id,
                first_name: "John".into(),
                last_name: "Doe".into(),
                ..Default::default()
            })
        }
    }
}

pub mod v2 {
    use super::*;
    
    pub mod people {
        use super::*;

        #[utoipa::path(
            get,
            path = "",
            summary = "Get Person",
            description = "Gets a single person.",
            params(
                ("id" = u32, Path, description = "The requested person identifier.")
            ),
            responses(
                (status = 200, description = "The person was successfully retrieved.", body = [Person]),
                (status = 404, description = "The person does not exist.")
            ))]
        pub async fn by_id(Path(id): Path<u32>) -> Json<Person> {
            Json(Person {
                id,
                first_name: "John".into(),
                last_name: "Doe".into(),
                email: "john.doe@somewhere.come".into(),
                ..Default::default()
            })
        }
    }
}

pub mod v3 {
    use super::*;
    
    pub mod people {
        use super::*;

        #[utoipa::path(
            get,
            path = "",
            summary = "Get Person",
            description = "Gets a single person.",
            params(
                ("id" = u32, Path, description = "The requested person identifier.")
            ),
            responses(
                (status = 200, description = "The person was successfully retrieved.", body = [Person]),
                (status = 404, description = "The person does not exist.")
            ))]
        pub async fn by_id(Path(id): Path<u32>) -> Json<Person> {
            Json(Person {
                id,
                first_name: "John".into(),
                last_name: "Doe".into(),
                email: "john.doe@somewhere.com".into,
                phone: "555-987-1234".into(),
            })
        }
    }
}

fn people(api: VersionedApi) -> VersionedApi {
    api.name("People")
       .has([api_version!(1.0), api_version!(2.0), api_version!(3.0)])
       .route("/{id}", |e| {
           e.get(doc!(v1::people::by_id))
               .map(api_version!(2.0), |h| h.get(doc!(v2::people::by_id)))
               .map(api_version!(3.0), |h| h.get(doc!(v3::people::by_id)))
       })
}

/// Creates the application and an OpenAPI document for each API version.
pub fn app() -> (axum::Router, Vec<ApiDocument>) {
    let (app, apis) = Router::default().api("/api/people", people).split();
    (app, documents(&apis, ApiDoc::openapi()))
}
```

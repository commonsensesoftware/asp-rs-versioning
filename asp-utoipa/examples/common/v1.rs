use super::models::{Order, Person};
use super::{Created, location};
use api_versioning::ApiVersion;
use axum::Json;
use axum::extract::{OriginalUri, Path};
use http::StatusCode;

pub mod orders {
    use super::*;

    #[utoipa::path(
        get,
        path = "",
        summary = "Get Order",
        description = "Gets a single order.",
        params(
            ("id" = u32, Path, description = "The requested order identifier.")
        ),
        responses(
            (status = 200, description = "The order was successfully retrieved.", body = [Order]),
            (status = 404, description = "The order does not exist.")
        ))]
    pub async fn by_id(Path(id): Path<u32>) -> Json<Order> {
        Json(Order {
            id,
            customer: "John Doe".into(),
            ..Default::default()
        })
    }

    #[utoipa::path(
        post,
        path = "",
        summary = "Place Order",
        description = "Places a new order.",
        params(
            ("id" = u32, Path, description = "The requested order identifier.")
        ),
        responses(
            (status = 201, description = "The order was successfully placed.", body = [Order]),
            (status = 422, description = "The order is invalid.")
        ))]
    pub async fn place(OriginalUri(url): OriginalUri, Json(mut order): Json<Order>) -> Created<Json<Order>> {
        order.id = 42;
        Created::at(location(&url, 42)).with(Json(order))
    }

    #[utoipa::path(
        patch,
        path = "",
        summary = "Update Order",
        description = "Updates an existing order.",
        params(
            ("id" = u32, Path, description = "The requested order identifier."),
        ),
        request_body(
            content = Order,
            description = "The order to update."
        ),
        responses(
            (status = 204, description = "The order was successfully updated."),
            (status = 422, description = "The order is invalid."),
            (status = 404, description = "The order does not exist.")
        ))]
    pub async fn update(Path(_): Path<u32>, Json(_): Json<Order>) -> StatusCode {
        StatusCode::NO_CONTENT
    }
}

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
    pub async fn by_id(Path((_, id)): Path<(ApiVersion<'static>, u32)>) -> Json<Person> {
        Json(Person {
            id,
            first_name: "John".into(),
            last_name: "Doe".into(),
            ..Default::default()
        })
    }
}

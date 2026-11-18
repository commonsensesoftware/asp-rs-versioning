use super::models::{Order, Person};
use super::{Created, location};
use api_versioning::ApiVersion;
use axum::Json;
use axum::extract::{OriginalUri, Path};
use http::StatusCode;
use std::ops::Add;
use time::{Duration, OffsetDateTime};

pub mod orders {
    use super::*;

    #[utoipa::path(
        get,
        path = "",
        summary = "Get Orders",
        description = "Retrieves all orders.",
        responses(
            (status = 200, description = "The successfully retrieved orders", body = [Order]),
        ))]
    pub async fn all() -> Json<Vec<Order>> {
        Json(vec![
            Order {
                id: 1,
                customer: "John Doe".into(),
                ..Default::default()
            },
            Order {
                id: 2,
                customer: "Bob Smith".into(),
                ..Default::default()
            },
            Order {
                id: 3,
                customer: "Jane Doe".into(),
                effective_date: OffsetDateTime::now_utc().add(Duration::days(7)).into(),
                ..Default::default()
            },
        ])
    }

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
        delete,
        path = "",
        summary = "Cancel Order",
        description = "Cancels an order.",
        params(
            ("id" = u32, Path, description = "The requested order identifier.")
        ),
        responses(
            (status = 204, description = "The order was successfully canceled.", body = [Order]),
            (status = 404, description = "The order does not exist.")
        ))]
    pub async fn cancel(Path(_): Path<u32>) -> StatusCode {
        StatusCode::NO_CONTENT
    }
}

pub mod people {
    use super::*;

    #[utoipa::path(
        get,
        path = "",
        summary = "Get People",
        description = "Retrieves all people.",
        responses(
            (status = 200, description = "The successfully retrieved people", body = [Person]),
        ))]
    pub async fn all(Path(_): Path<ApiVersion<'static>>) -> Json<Vec<Person>> {
        Json(vec![
            Person {
                id: 1,
                first_name: "John".into(),
                last_name: "Doe".into(),
                email: "john.doe@somewhere.com".into(),
                phone: "555-987-1234".into(),
            },
            Person {
                id: 2,
                first_name: "Bob".into(),
                last_name: "Smith".into(),
                email: "bob.smith@somewhere.com".into(),
                phone: "555-654-4321".into(),
            },
            Person {
                id: 3,
                first_name: "Jane".into(),
                last_name: "Doe".into(),
                email: "jane.doe@somewhere.com".into(),
                phone: "555-789-3456".into(),
            },
        ])
    }

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
            email: "john.doe@somewhere.com".into(),
            phone: "555-987-1234".into(),
        })
    }

    #[utoipa::path(
        post,
        path = "",
        summary = "Add Person",
        description = "Adds a new person.",
        responses(
            (status = 201, description = "The person was successfully added.", body = [Person]),
            (status = 422, description = "The person was invalid.")
        ))]
    pub async fn add(
        OriginalUri(url): OriginalUri,
        Path(_): Path<ApiVersion<'static>>,
        Json(mut person): Json<Person>,
    ) -> Created<Json<Person>> {
        person.id = 42;
        Created::at(location(&url, 42)).with(Json(person))
    }
}

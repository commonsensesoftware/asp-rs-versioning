use api_versioning::{api_version, visible_in_version};
use api_versioning_axum::Router;
use axum::{Json, body::Body};
use http::{Request, StatusCode, header::CONTENT_TYPE};
use rstest::rstest;
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

#[visible_in_version]
#[derive(Deserialize, Serialize)]
struct Order {
    id: u32,
    #[range(2.0..)]
    customer: Option<String>,
}

async fn get_order() -> Json<Order> {
    Json(Order {
        id: 1,
        customer: Some("Bill".into()),
    })
}

async fn read_customer(Json(order): Json<Order>) -> String {
    format!("{:?}", order.customer)
}

async fn get_health() -> Json<Order> {
    get_order().await
}

fn app() -> axum::Router {
    let router = Router::default();

    router
        .api("/orders", |api| {
            api.has([api_version!(1.0), api_version!(2.0)])
                .route("/", |e| e.get(get_order).post(read_customer))
        })
        .api("/health", |api| api.neutral().route("/", |e| e.get(get_health)))
        .into()
}

async fn send(request: Request<Body>) -> Result<(StatusCode, String)> {
    let response = app().oneshot(request).await?;
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;

    Ok((status, String::from_utf8(body.to_vec())?))
}

#[rstest]
#[case("/orders?api-version=1.0", r#"{"id":1}"#)]
#[case("/orders?api-version=2.0", r#"{"id":1,"customer":"Bill"}"#)]
#[case("/health", r#"{"id":1,"customer":"Bill"}"#)]
#[tokio::test]
async fn response_should_only_contain_members_visible_in_api_version(
    #[case] uri: &str,
    #[case] expected: &str,
) -> Result {
    // arrange
    let request = Request::get(uri).body(Body::empty())?;

    // act
    let (status, body) = send(request).await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, expected);
    Ok(())
}

#[rstest]
#[case("1.0", r#"{"id":1}"#, "None")]
#[case("2.0", r#"{"id":1}"#, "None")]
#[case("2.0", r#"{"id":1,"customer":"Bill"}"#, r#"Some("Bill")"#)]
#[tokio::test]
async fn request_should_read_members_visible_in_api_version(
    #[case] version: &str,
    #[case] json: &'static str,
    #[case] expected: &str,
) -> Result {
    // arrange
    let request = Request::post(format!("/orders?api-version={version}"))
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(json))?;

    // act
    let (status, body) = send(request).await?;

    // assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, expected);
    Ok(())
}

#[tokio::test]
async fn request_should_reject_member_hidden_in_api_version() -> Result {
    // arrange
    let request = Request::post("/orders?api-version=1.0")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"id":1,"customer":"Bill"}"#))?;

    // act
    let (status, body) = send(request).await?;

    // assert
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("customer: not supported in API version 1.0"), "{body}");
    Ok(())
}

use api_versioning::{Options, api_version, visibility::scope, visible_in_version};
use api_versioning_axum::{ApiDescriptions, Router};
use api_versioning_utoipa::{ApiDocument, doc, documents};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use utoipa::{
    PartialSchema, ToSchema,
    openapi::{InfoBuilder, OpenApiBuilder},
};

#[derive(Deserialize, Serialize, ToSchema)]
struct Customer {
    name: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
struct Address {
    city: String,
}

/// An order.
#[visible_in_version]
#[derive(Default, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct Order {
    id: u32,

    /// The customer that placed the order.
    #[range(2.0..)]
    customer: Option<Customer>,

    #[range(..2.0)]
    #[serde(rename = "code")]
    legacy_code: Option<String>,

    #[range(3.0..)]
    #[schema(example = "rush")]
    priority_level: String,
}

#[visible_in_version]
#[derive(Serialize, ToSchema)]
struct Page<T> {
    items: Vec<T>,

    #[range(2.0..)]
    total: Option<u32>,
}

#[visible_in_version]
#[derive(Serialize, ToSchema)]
struct Shipment {
    id: u32,

    #[range(2.0..)]
    address: Option<Address>,

    #[range(3.0..)]
    #[schema(inline)]
    origin: Option<Address>,
}

#[utoipa::path(get, path = "", responses((status = 200, body = Order)))]
async fn get_order() {}

#[utoipa::path(get, path = "", responses((status = 200, body = Page<Order>)))]
async fn get_orders() {}

#[utoipa::path(post, path = "", request_body = inline(Shipment), responses((status = 201)))]
async fn ship() {}

fn apis() -> ApiDescriptions {
    Router::<()>::new(Options::default())
        .api("/orders", |api| {
            api.has([api_version!(1.0), api_version!(2.0), api_version!(3.0)])
                .route("/", |e| e.get(doc!(get_orders)))
                .route("/{id}", |e| e.get(doc!(get_order)))
                .route("/{id}/shipments", |e| e.post(doc!(ship)))
        })
        .split()
        .1
}

fn json(document: &ApiDocument) -> Value {
    serde_json::to_value(document.openapi()).unwrap()
}

fn keys(value: &Value) -> Vec<&str> {
    value
        .as_object()
        .map_or_else(Vec::new, |map| map.keys().map(String::as_str).collect())
}

fn base() -> utoipa::openapi::OpenApi {
    OpenApiBuilder::new()
        .info(InfoBuilder::new().title("Orders").version("0.0.0"))
        .build()
}

#[test]
fn documents_should_only_have_members_visible_in_api_version() {
    // arrange
    let apis = apis();

    // act
    let documents = documents(&apis, base());

    // assert
    let properties: Vec<_> = documents
        .iter()
        .map(|document| json(document)["components"]["schemas"]["Order"]["properties"].clone())
        .collect();
    assert_eq!(keys(&properties[0]), ["code", "id"]);
    assert_eq!(keys(&properties[1]), ["customer", "id"]);
    assert_eq!(keys(&properties[2]), ["customer", "id", "priorityLevel"]);
}

#[test]
fn documents_should_retain_attributes_of_visible_members() {
    // arrange
    let apis = apis();

    // act
    let documents = documents(&apis, base());

    // assert
    let order = &json(&documents[2])["components"]["schemas"]["Order"];
    assert_eq!(order["description"], "An order.");
    assert_eq!(order["required"], json!(["id"]));
    assert_eq!(order["properties"]["priorityLevel"]["example"], "rush");
    assert_eq!(
        order["properties"]["customer"]["oneOf"][0]["$ref"],
        "#/components/schemas/Customer"
    );
}

#[test]
fn documents_should_only_have_schemas_referenced_by_visible_members() {
    // arrange
    let apis = apis();

    // act
    let documents = documents(&apis, base());

    // assert
    let schemas: Vec<_> = documents
        .iter()
        .map(|document| {
            keys(&json(document)["components"]["schemas"])
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(schemas[0], ["Order", "Page_Order"]);
    assert_eq!(schemas[1], ["Address", "Customer", "Order", "Page_Order"]);
    assert_eq!(schemas[2], ["Address", "Customer", "Order", "Page_Order"]);
}

#[test]
fn documents_should_only_have_visible_members_of_generic_schema() {
    // arrange
    let apis = apis();

    // act
    let documents = documents(&apis, base());

    // assert
    let page = |document: &ApiDocument| json(document)["components"]["schemas"]["Page_Order"]["properties"].clone();
    assert_eq!(keys(&page(&documents[0])), ["items"]);
    assert_eq!(keys(&page(&documents[1])), ["items", "total"]);
}

#[test]
fn documents_should_only_have_visible_members_of_inline_schema() {
    // arrange
    let apis = apis();

    // act
    let documents = documents(&apis, base());

    // assert
    let body = |document: &ApiDocument| {
        json(document)["paths"]["/orders/{id}/shipments"]["post"]["requestBody"]["content"]["application/json"]["schema"]
            ["properties"]
            .clone()
    };
    assert_eq!(keys(&body(&documents[0])), ["id"]);
    assert_eq!(keys(&body(&documents[1])), ["address", "id"]);
    assert_eq!(keys(&body(&documents[2])), ["address", "id", "origin"]);
}

#[test]
fn schema_should_have_every_member_without_api_version() {
    // arrange

    // act
    let schema = serde_json::to_value(Order::schema()).unwrap();

    // assert
    assert_eq!(keys(&schema["properties"]), ["code", "customer", "id", "priorityLevel"]);
}

#[test]
fn schema_should_only_have_members_visible_in_current_api_version() {
    // arrange
    let version = Some(api_version!(1.0));

    // act
    let schema = scope(version, || serde_json::to_value(Order::schema()).unwrap());

    // assert
    assert_eq!(keys(&schema["properties"]), ["code", "id"]);
    assert_eq!(<Order as ToSchema>::name(), "Order");
}

#[test]
fn serialize_should_still_only_write_visible_members() {
    // arrange
    let order = Order {
        id: 1,
        legacy_code: Some("A1".into()),
        ..Default::default()
    };

    // act
    let json = scope(Some(api_version!(1.0)), || serde_json::to_string(&order).unwrap());

    // assert
    assert_eq!(json, r#"{"id":1,"code":"A1"}"#);
}

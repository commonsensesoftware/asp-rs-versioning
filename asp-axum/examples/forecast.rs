//! Run with `cargo run --package api-versioning-axum --example forecast`, then test with `forecast.http`

use api_versioning::{Options, api_version};
use api_versioning_axum::Router;
use axum::Json;
use http::StatusCode;
use rand::random_range;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

static SUMMARIES: &[&str] = &[
    "Freezing",
    "Bracing",
    "Chilly",
    "Cool",
    "Mild",
    "Warm",
    "Balmy",
    "Hot",
    "Sweltering",
    "Scorching",
];

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Forecast {
    summary: String,

    #[serde(with = "time::serde::rfc3339")]
    date: OffsetDateTime,

    temperature_c: i32,

    temperature_f: i32,
}

fn new_forecast(days: usize) -> Forecast {
    let summary = SUMMARIES[random_range(0..SUMMARIES.len())].to_string();
    let temperature_c = random_range(-22..=55);
    let date = OffsetDateTime::now_utc() + Duration::days(days as i64);

    Forecast {
        summary,
        date,
        temperature_c,
        temperature_f: temperature_c * 9 / 5 + 32,
    }
}

// GET /forecast?api-version=1.0
async fn fetch() -> Json<Vec<Forecast>> {
    Json((1..=5).map(new_forecast).collect::<Vec<Forecast>>())
}

// GET /forecast?api-version=2.0
async fn fetch_v2() -> Json<Vec<Forecast>> {
    Json((0..SUMMARIES.len()).map(new_forecast).collect::<Vec<Forecast>>())
}

// POST /forecast?api-version=2.0
async fn submit(Json(_): Json<Forecast>) -> StatusCode {
    StatusCode::OK
}

// DELETE /forecast
async fn remove() -> StatusCode {
    StatusCode::NO_CONTENT
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut options = Options::default();

    // enable api versioning and return the headers "api-supported-versions" and "api-deprecated-versions"
    options.report_versions.enabled = true;

    let app: axum::Router = Router::new(options)
        .api("/forecast", |api| {
            api.has([api_version!(1.0), api_version!(2.0)]).route("/", |e| {
                e.get(fetch)
                    .map(api_version!(2.0), |e| e.get(fetch_v2).post(submit))
                    .neutral(|h| h.delete(remove))
            })
        })
        .into();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;
    Ok(())
}

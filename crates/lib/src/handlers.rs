use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::csv_ops::{read_csv_to_str, str_to_csv};

/// Others
#[derive(Serialize)]
pub struct PingPong {
    response: String,
}

#[derive(Serialize)]
pub struct IntakesElm {
    line: String,
}

#[derive(Deserialize)]
pub struct SIntakes {
    line: String,
}

#[derive(Serialize, Debug)]
pub struct SResponse {
    line: String,
}

pub struct StateParams {
    pub filepath: Arc<String>,
}

pub async fn csv_to_intakes(State(state): State<Arc<StateParams>>) -> impl IntoResponse {
    let path = state.filepath.as_str();

    let ret = match read_csv_to_str(path.into()) {
        Err(_) => "".into(),
        Ok(s) => s,
    };

    let body = IntakesElm { line: ret };
    Json(body)
}

pub async fn pong() -> impl IntoResponse {
    let body = PingPong {
        response: "pong".into(),
    };
    Json(body)
}

pub async fn intakes_to_csv(Json(payload): Json<SIntakes>) -> (StatusCode, Json<SResponse>) {
    let this = SResponse { line: payload.line };

    // println!("resp -> {:#?}", this);
    str_to_csv(this.line.clone(), "./test/out.csv".into());
    (StatusCode::CREATED, Json(this))
}

pub async fn root() -> &'static str {
    "Hello, World!"
}

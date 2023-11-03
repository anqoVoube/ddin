pub mod media;

use axum::body::Body;
use axum::Json;
use serde::Serialize;
use axum::response::{IntoResponse, Response};
use http::StatusCode;

#[derive(Serialize)]
pub struct ErrorResponse{
    pub message: String
}


#[derive(Serialize)]
pub struct SuccessResponse{
    pub message: String
}

pub fn ok<T: Serialize>(data: T) -> impl IntoResponse {
    Json(data)
}

pub fn created<T: Serialize>(data: T) -> impl IntoResponse {
    let mut response = Json(data).into_response();
    *response.status_mut() = StatusCode::CREATED;
    response
}


pub fn default_ok() -> Response{
    (
        StatusCode::OK,
        Json(
            SuccessResponse{
                message: "Operation is successful".to_string()
            }
        )
    ).into_response()
}


pub fn default_created() -> Response{
    (
        StatusCode::CREATED,
        Json(
            SuccessResponse{
                message: "Operation is successful".to_string()
            }
        )
    ).into_response()
}

pub fn bad_request(message: &str) -> Response{
    (
        StatusCode::BAD_REQUEST,
        Json(
            ErrorResponse{
                message: message.to_string()
            }
        )
    ).into_response()
}

pub fn default_missing_header() -> Response{
    (
        StatusCode::NOT_ACCEPTABLE
    ).into_response()
}


pub fn not_found() -> Response{
    (
        StatusCode::NOT_FOUND
    ).into_response()
}


pub fn internal_server_error() -> Response{
    (
        StatusCode::INTERNAL_SERVER_ERROR
    ).into_response()
}

pub fn response_builder(status: StatusCode, body_text: &str) -> Response<Body>{
    Response::builder()
        .status(status)
        .body(Body::from(body_text.to_owned()))
        .unwrap()
}
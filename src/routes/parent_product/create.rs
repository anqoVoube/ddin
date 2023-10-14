use axum::{Extension, Json, debug_handler};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use serde::{Deserialize, Serialize};
use chrono::NaiveTime;
use sea_orm::ActiveValue::Set;
use crate::database::parent_product;
use rust_decimal::Decimal;
use log::error;

fn default_as_false() -> bool {
    false
}


#[derive(Clone, Serialize, Deserialize)]
pub struct Body {
    code: String,
    title: String,
    description: String,
}

#[debug_handler]
pub async fn create(
    Extension(database): Extension<DatabaseConnection>,
    Json(Body { code, title, description }): Json<Body>
) -> Response {
    let new_parent_product = parent_product::ActiveModel {
        code: Set(code),
        title: Set(title),
        description: Set(description),
        ..Default::default()
    };
    let result = new_parent_product.save(&database).await;

    match result{
        Ok(idk) => {
            println!("{:?}", idk);
            (
                StatusCode::CREATED
            ).into_response()
        },
        Err(error) => {
            error!("Unable to create {:?}. Original error was {}", 1, error);
            (
                StatusCode::INTERNAL_SERVER_ERROR
            ).into_response()
        }
    }
}

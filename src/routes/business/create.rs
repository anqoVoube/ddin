use axum::{Extension, Json, debug_handler};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use serde::{Deserialize, Serialize};
use chrono::NaiveTime;
use sea_orm::ActiveValue::Set;
use crate::database::business;
use rust_decimal::Decimal;
use log::error;

fn default_as_false() -> bool {
    false
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Body {
    title: String,
    location: Vec<Decimal>,
    works_from: NaiveTime,
    works_until: NaiveTime,
    #[serde(default="default_as_false")]
    is_closed: bool
}



#[debug_handler]
pub async fn create(
    Extension(database): Extension<DatabaseConnection>,
    Json(Body {title, location, works_from, works_until, is_closed}): Json<Body>
) -> Response {
    let new_business = business::ActiveModel {
        title: Set(title),
        location: Set(location),
        works_from: Set(works_from),
        works_until: Set(works_until),
        is_closed: Set(is_closed),
        owner_id: Set(1),
        ..Default::default()
    };
    let result = new_business.save(&database).await;
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

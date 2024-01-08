use axum::{Extension, Json, debug_handler};
use axum::response::{IntoResponse, Response};
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use serde::{Deserialize, Serialize};
use chrono::NaiveTime;
use http::StatusCode;
use sea_orm::ActiveValue::Set;
use rust_decimal::Decimal;
use log::{error, info};
use crate::core::auth::middleware::{Auth, CustomHeader};
use crate::create_model;
use crate::routes::utils::{internal_server_error};
use crate::database::business::ActiveModel;
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
    is_closed: bool,
}


#[derive(Serialize, Deserialize)]
pub struct ResponseBody{
    business_id: i32
}

#[debug_handler]
pub async fn create(
    Extension(database): Extension<DatabaseConnection>,
    Extension(auth): Extension<Auth>,
    Json(Body {title, location, works_from, works_until, is_closed}): Json<Body>
) -> Response{
    let owner_id = auth.user_id;
    match create_model!(
        ActiveModel, &database, title, location, works_from, works_until, is_closed, owner_id) {
        Ok(instance) => {
            let business_id = instance.id.clone().unwrap();
            info!("{:?}", instance);
            (
                StatusCode::CREATED,
                Json(ResponseBody{business_id})
            ).into_response()
        },

        Err(e) => {
            println!("{:?}", e);
            error!("Unable to create");
            internal_server_error()
        }
    }
}

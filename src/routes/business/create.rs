use axum::{Extension, Json, debug_handler};
use axum::response::{IntoResponse, Response};
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use serde::{Deserialize, Serialize};
use chrono::NaiveTime;
use http::StatusCode;
use sea_orm::ActiveValue::Set;
use crate::database::business;
use rust_decimal::Decimal;
use log::{error, info};
use crate::core::auth::middleware::Auth;
use crate::routes::AppConnections;
use crate::routes::utils::{internal_server_error};

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
    phone_number: String
}



#[derive(Serialize, Deserialize)]
pub struct ResponseBody{
    business_id: i32
}


#[debug_handler]
pub async fn create(
    Extension(AppConnections{redis, database, scylla}): Extension<AppConnections>,
    Extension(auth): Extension<Auth>,
    Json(Body {title, location, works_from, works_until, is_closed, phone_number}): Json<Body>
) -> Response{
    let new_business = business::ActiveModel {
        title: Set(title),
            location: Set(location),
            works_from: Set(works_from),
            works_until: Set(works_until),
            is_closed: Set(is_closed),
            owner_id: Set(auth.user_id),
        ..Default::default()
    };
    println!("{}", phone_number);
    match new_business.save(&database).await {
        Ok(instance) => {
            info!("{:?}", instance);
            println!("{}", instance.clone().id.unwrap());
            (
                StatusCode::CREATED,
                Json(ResponseBody{business_id: instance.id.unwrap()})
            ).into_response()
        },

        _ => {
            error!("Unable to create");
            internal_server_error()
        }
    }
}

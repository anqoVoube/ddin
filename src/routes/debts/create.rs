use std::string::ToString;
use std::sync::Arc;
use axum::{Extension, Json};
use axum::extract::Query;
use axum::response::{Response, IntoResponse};
use chrono::NaiveDate;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use http::StatusCode;
use log::{error, info};
use crate::core::auth::middleware::{Auth, CustomHeader};

use sea_orm::entity::*;
use sea_orm::query::*;
use serde::{Deserialize, Serialize};
use crate::database::rent;
use crate::routes::utils::{default_created, internal_server_error};

#[derive(Deserialize, Serialize)]
pub struct Body {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseBody {
    id: i32,
    name: String,
}

pub async fn create(
    Extension(auth): Extension<Auth>,
    Extension(headers): Extension<CustomHeader>,
    Extension(database): Extension<DatabaseConnection>,
    Json(Body{name}): Json<Body>
) -> Response{
    let new_debt_user = rent::ActiveModel {
        name: Set(name),
        price: Set(0f64),
        business_id: Set(headers.business_id),
        ..Default::default()
    };

    match new_debt_user.save(&database).await {
        Ok(instance) => {
            info!("{:?}", instance);
            let response = ResponseBody{
                id: instance.id.unwrap(),
                name: instance.name.unwrap()
            };
            (
                StatusCode::CREATED,
                Json(response)
            ).into_response()
        },
        Err(error) => {
            error!("Unable to create {:?}. Original error was {}", 1, error);
            internal_server_error()
        }
    }
}


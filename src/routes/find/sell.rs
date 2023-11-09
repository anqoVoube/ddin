use std::string::ToString;
use axum::{Extension, Json};
use axum::extract::Query;
use axum::response::{Response, IntoResponse};
use chrono::NaiveDate;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use http::StatusCode;
use crate::core::auth::middleware::Auth;

use sea_orm::entity::*;
use sea_orm::query::*;
use crate::database::prelude::ParentWeightItem;
use crate::routes::AppConnections;
use crate::routes::utils::condition::starts_with;
use crate::routes::find::{find_weight_item, find_no_code_product, find_product, Search, Types};


pub async fn search(
    Extension(auth): Extension<Auth>,
    Extension(connections): Extension<AppConnections>,
    Query(query): Query<Search>
) -> Response{
        println!("{} {:?}", query.search, query.r#type);
        match query.r#type{
            Types::Product => {
                let data = find_product(query.search, auth.business_id, &connections.database).await;
                ().into_response()
            },
            Types::WeightItem => {
                let data = find_weight_item(
                    query.search,
                    auth.business_id,
                    &connections.database
                ).await;
                (
                    StatusCode::OK,
                    Json(data)
                ).into_response()
            },
            Types::NoCodeProduct => {
                let data = find_no_code_product(
                    query.search,
                    auth.business_id,
                    &connections.database
                ).await;
                (
                    StatusCode::OK,
                    Json(data)
                ).into_response()
            }
        }
}


use std::string::ToString;
use std::sync::Arc;
use axum::{Extension, Json};
use axum::extract::Query;
use axum::response::{Response, IntoResponse};
use chrono::NaiveDate;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use http::StatusCode;
use scylla::Session;
use crate::core::auth::middleware::Auth;

use sea_orm::entity::*;
use sea_orm::query::*;
use crate::database::prelude::ParentWeightItem;
use crate::RedisPool;
use crate::routes::utils::condition::starts_with;
use crate::routes::find::{find_weight_item, find_no_code_product, find_product, Search, Types};


pub async fn search(
    // Extension(Auth{ user_id, business_id }): Extension<Auth>,
    Extension(database): Extension<DatabaseConnection>,
    Query(query): Query<Search>
) -> Response{
        println!("{} {:?}", query.search, query.r#type);
        let business_id: i32 = 1;
        match query.r#type{
            Types::Product => {
                let data = find_product(query.search, business_id, &database).await;
                ().into_response()
            },
            Types::WeightItem => {
                let data = find_weight_item(
                    query.search,
                    business_id,
                    &database
                ).await;
                (
                    StatusCode::OK,
                    Json(data)
                ).into_response()
            },
            Types::NoCodeProduct => {
                let data = find_no_code_product(
                    query.search,
                    business_id,
                    &database
                ).await;
                (
                    StatusCode::OK,
                    Json(data)
                ).into_response()
            }
        }
}


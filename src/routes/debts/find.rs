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
use serde::{Deserialize, Serialize};
use crate::database::prelude::Rent;
use crate::database::rent;
use crate::routes::utils::condition::starts_with;

#[derive(Deserialize, Serialize)]
pub struct Search {
    search: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Debt{
    id: i32,
    name: String,
}


#[derive(Serialize,  Deserialize, Debug)]
pub struct Debts{
    debts: Vec<Debt>
}

pub async fn small_serializer_search(
    Extension(auth): Extension<Auth>,
    Extension(database): Extension<DatabaseConnection>,
    Query(query): Query<Search>
) -> Response{
    let debts = Rent::find()
        .filter(
            Condition::all()
                .add(rent::Column::BusinessId.eq(auth.business_id))
                .add(starts_with(&query.search, rent::Column::Name, false))
        )
        .all(&database)
        .await
        .unwrap();
    let mut debts_schema = Debts{debts: vec![]};
    for debt in debts{
        debts_schema.debts.push(Debt{
            id: debt.id,
            name: debt.name
        })
    }
    (
        StatusCode::OK,
        Json(debts_schema)
    ).into_response()
}

pub async fn full_serializer_search(
    Extension(auth): Extension<Auth>,
    Extension(database): Extension<DatabaseConnection>,
    Query(query): Query<Search>
) -> Response{
    let debts = Rent::find()
        .filter(
            Condition::all()
                .add(rent::Column::BusinessId.eq(auth.business_id))
                .add(starts_with(&query.search, rent::Column::Name, false))
        )
        .all(&database)
        .await
        .unwrap();
    let mut debts_schema = Debts{debts: vec![]};
    for debt in debts{
        debts_schema.debts.push(Debt{
            id: debt.id,
            name: debt.name
        })
    }
    (
        StatusCode::OK,
        Json(debts_schema)
    ).into_response()
}



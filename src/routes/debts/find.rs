use axum::{Extension, Json};
use axum::extract::Query;
use axum::response::{Response, IntoResponse};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use http::StatusCode;
use crate::core::auth::middleware::Auth;

use sea_orm::entity::*;
use sea_orm::query::*;
use serde::{Deserialize, Serialize};
use crate::database::prelude::Rent;
use crate::database::rent;
use crate::routes::utils::condition::starts_with;

#[derive(Deserialize, Serialize)]
pub struct Search {
    search: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SmallDebt{
    id: i32,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FullDebt{
    id: i32,
    name: String,
    price: i32
}

#[derive(Serialize,  Deserialize, Debug)]
pub struct SmallDebts{
    debts: Vec<SmallDebt>
}

#[derive(Serialize,  Deserialize, Debug)]
pub struct FullDebts{
    debts: Vec<FullDebt>
}


pub async fn full_serializer_search(
    Extension(auth): Extension<Auth>,
    Extension(database): Extension<DatabaseConnection>,
    Query(query): Query<Search>
) -> Response{
    let mut condition = Condition::all()
        .add(rent::Column::BusinessId.eq(auth.business_id));
    if let Some(search) = query.search{
        condition = condition.add(starts_with(&search, rent::Column::Name, false))
    }
    let debts = Rent::find()
        .filter(
            condition
        )
        .all(&database)
        .await
        .unwrap();
    let mut debts_schema = FullDebts{debts: vec![]};
    for debt in debts{
        debts_schema.debts.push(FullDebt{
            id: debt.id,
            name: debt.name,
            price: debt.price
        })
    }
    (
        StatusCode::OK,
        Json(debts_schema)
    ).into_response()
}

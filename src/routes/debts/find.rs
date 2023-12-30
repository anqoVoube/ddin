use axum::{Extension, Json};
use axum::extract::Query;
use axum::response::{Response, IntoResponse};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use http::StatusCode;
use multipart::server::nickel::nickel::hyper::header::q;
use crate::core::auth::middleware::{Auth, CustomHeader};

use sea_orm::entity::*;
use sea_orm::query::*;
use serde::{Deserialize, Serialize};
use crate::database::prelude::Rent;
use crate::database::rent;
use crate::database::rent::Model;
use crate::routes::utils::condition::contains;

const DEFAULT_PAGE_SIZE: i32 = 15;
const DEFAULT_PAGE: i32 = 1;

#[derive(Deserialize, Serialize)]
pub struct Search {
    search: Option<String>,
    page: Option<i32>,
    page_size: Option<i32>,
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
    debts: Vec<FullDebt>,
    next_page: bool
}


pub async fn full_serializer_search(
    Extension(auth): Extension<Auth>,
    Extension(headers): Extension<CustomHeader>,
    Extension(database): Extension<DatabaseConnection>,
    Query(query): Query<Search>
) -> Response{
    let mut debts: Vec<Model> = vec![];
    let mut condition = Condition::all()
        .add(rent::Column::BusinessId.eq(headers.business_id));
    if let Some(search) = query.search{
        condition = condition.add(contains(&search, rent::Column::Name, false));
        debts = Rent::find()
            .filter(
                condition
            )
            .all(&database)
            .await
            .unwrap();
    } else {
        debts = Rent::find()
            .filter(
                condition
            )
            .offset(((query.page.unwrap_or(DEFAULT_PAGE) - 1) * query.page_size.unwrap_or(DEFAULT_PAGE_SIZE)) as u64)
            .limit(query.page_size.unwrap_or(DEFAULT_PAGE_SIZE + 1) as u64)
            .all(&database)
            .await
            .unwrap();
    }
    let mut debts_schema = FullDebts{debts: vec![], next_page: true};
    let debts_length = debts.len();
    if debts_length as i32 <= DEFAULT_PAGE_SIZE{
        debts_schema.next_page = false;
    }

    for debt in debts.drain(0..std::cmp::min(DEFAULT_PAGE_SIZE as usize, debts_length)){
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

use std::collections::BTreeMap;
use std::error::Error;
use axum::{debug_handler, Extension, Json};
use axum::response::{IntoResponse, Response};
use chrono::{Datelike, NaiveDate};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use crate::routes::statistics::{get_date_range, Types};
use axum::extract::Query;
use sea_orm::{DatabaseConnection, EntityTrait};
use crate::core::auth::middleware::{Auth, CustomHeader};
use crate::database::prelude::ProductStatistics;
use crate::database::product_statistics;
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;

#[derive(Serialize)]
pub struct Statistics{
    profits: Vec<f64>,
    quantities: Vec<i32>,
    namings: Vec<String>,
}

#[derive(Deserialize)]
pub struct Search {
    r#type: Types,
    prev: u8,
    parent_id: i32,
    item_type: i16
}


#[debug_handler]
pub async fn product_stats(
    Extension(Auth{user_id}): Extension<Auth>,
    Extension(CustomHeader{business_id}): Extension<CustomHeader>,
    Extension(database): Extension<DatabaseConnection>,
    Query(Search{r#type, prev, parent_id, item_type}): Query<Search>,
) -> Response{
    match get_product_stats(
        &database, parent_id,
        business_id, item_type,
        r#type, prev
    ).await{
        Ok(stats) => {
            let (quantities, profits, namings) = stats;
            let statistics = Statistics{
                quantities,
                profits,
                namings
            };
            (
                StatusCode::OK, Json(statistics)
            ).into_response()
        },
        Err(error) => {
            println!("Error: {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR
            ).into_response()
        }
    }
}


pub async fn get_product_stats(
    database: &DatabaseConnection,
    parent_id: i32,
    business_id: i32,
    item_type: i16,
    r#type: Types,
    prev: u8
) -> Result<(Vec<i32>, Vec<f64>, Vec<String>), Box<dyn Error>> {
    let (start_date, end_date, namings) = get_date_range(&r#type, prev);
    // convert to rust sea-orm below query
    // SELECT date, quantity, profit FROM statistics.products WHERE parent_id = ? AND business_id = ? AND date >= ? AND date <= ? AND item_type = ?

    let products = ProductStatistics::find()
        .filter(
            product_statistics::Column::ParentId.eq(parent_id)
                .and(product_statistics::Column::BusinessId.eq(business_id))
                .and(product_statistics::Column::Date.gte(start_date))
                .and(product_statistics::Column::Date.lte(end_date))
                .and(product_statistics::Column::ItemType.eq(item_type))
        )
        .all(database)
        .await
        .unwrap();


    let mut profit_by_date: BTreeMap<NaiveDate, [f64; 2]> = BTreeMap::new();

    for product in products {
        println!("{} {} {}", product.date, product.quantity, product.profit);
        profit_by_date.entry(product.date).or_insert([0f64, 0f64]);
        if let Some(one_stats) = profit_by_date.get_mut(&product.date) {
            one_stats[0] += product.quantity as f64;
            one_stats[1] += product.profit;
        }
    }

    let mut quantities: Vec<i32> = (0..namings.len()).map(|_| 0).collect();
    let mut profits: Vec<f64> = (0..namings.len()).map(|_| 0f64).collect();
    for (date, total_stats) in profit_by_date {
        let [total_quantity, total_profit] = total_stats;
        println!("{} {}", total_quantity, total_profit);
        println!("DATE!!! {}", date);
        match r#type {
            Types::Year => {
                quantities[date.month0() as usize] += total_quantity as i32;
                profits[date.month0() as usize] += total_profit;
            },
            _ => {
                let index = date.signed_duration_since(start_date).num_days() as usize;
                quantities[index] += total_quantity as i32;
                profits[index] += total_profit;
            }
        }
    }
    Ok((quantities, profits, namings))
}
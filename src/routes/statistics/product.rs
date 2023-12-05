use std::collections::BTreeMap;
use std::error::Error;
use std::sync::Arc;
use axum::{debug_handler, Extension, Json};
use axum::response::{IntoResponse, Response};
use chrono::{Datelike, NaiveDate};
use http::StatusCode;
use scylla::{IntoTypedRows, Session};
use serde::{Deserialize, Serialize};
use crate::routes::ScyllaDBConnection;
use crate::routes::statistics::{get_date_range, Types};
use axum::extract::Query;
use crate::core::auth::middleware::Auth;

#[derive(Serialize)]
pub struct Statistics{
    profits: Vec<i32>,
    quantities: Vec<i32>,
    namings: Vec<String>,
}

#[derive(Deserialize)]
pub struct Search {
    r#type: Types,
    prev: u8,
    parent_id: i32,
    item_type: i8
}


#[debug_handler]
pub async fn product_stats(
    Extension(Auth{user_id, business_id}): Extension<Auth>,
    Extension(ScyllaDBConnection{scylla}): Extension<ScyllaDBConnection>,
    Query(Search{r#type, prev, parent_id, item_type}): Query<Search>,
) -> Response{
    match get_product_stats(
        scylla, parent_id,
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
    scylla: Arc<Session>,
    parent_id: i32,
    business_id: i32,
    item_type: i8,
    r#type: Types,
    prev: u8
) -> Result<(Vec<i32>, Vec<i32>, Vec<String>), Box<dyn Error>>{
    let (start_date, end_date, namings) = get_date_range(&r#type, prev);
    let query = "SELECT date, quantity, profit FROM statistics.products WHERE parent_id = ? AND business_id = ? AND date >= ? AND date <= ? AND item_type = ? ALLOW FILTERING";

    let results = scylla.query(
        query,
        (parent_id, business_id, start_date, end_date, item_type)
    ).await?;


    let mut profit_by_date: BTreeMap<NaiveDate, [i32; 2]> = BTreeMap::new();

    for row in results.rows.ok_or("Unable to fetch rows")?.into_typed::<(NaiveDate, i32, i32)>() {
        if let Ok(result) = row{
            let (date, quantity, profit) = result;
            println!("{} {} {}", date, quantity, profit);
            profit_by_date.entry(date).or_insert([0, 0]);
            if let Some(one_stats) = profit_by_date.get_mut(&date) {
                one_stats[0] += quantity;
                one_stats[1] += profit;
            }
        }
    }

    println!("{:?}", profit_by_date);
    let mut quantities: Vec<i32> = (0..namings.len()).map(|_| 0).collect();
    let mut profits: Vec<i32> = (0..namings.len()).map(|_| 0).collect();
    for (date, total_stats) in profit_by_date {
        let [total_quantity, total_profit] = total_stats;
        println!("{} {}", total_quantity, total_profit);
        println!("DATE!!! {}", date);
        match r#type{
            Types::Year => {
                quantities[date.month0() as usize] += total_quantity;
                profits[date.month0() as usize] += total_profit;
            },
            _ => {
                let index = date.signed_duration_since(start_date).num_days() as usize;
                quantities[index] += total_quantity;
                profits[index] += total_profit;
            }
        }
    }
    Ok((quantities, profits, namings))
}
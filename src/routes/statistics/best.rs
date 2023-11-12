use std::collections::BTreeMap;
use std::sync::Arc;
use axum::{debug_handler, Extension, Json};
use axum::extract::Query;
use axum::response::{Response, IntoResponse};
use chrono::{NaiveDate, NaiveDateTime, Local, Utc, Datelike, TimeZone};
use http::StatusCode;
use scylla::{IntoTypedRows, Session as ScyllaDBSession};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use crate::core::auth::middleware::Auth;
use crate::routes::parent_product::fetch::get_object_by_id;
use crate::routes::ScyllaDBConnection;


#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Types{
    #[serde(rename = "W")]
    Week,
    #[serde(rename = "M")]
    Month,
    #[serde(rename = "Y")]
    Year
}

#[derive(Deserialize, Serialize)]
pub struct Search {
    r#type: Types,
    prev: u8
}

trait NaiveDateExt {
    fn days_in_month(&self) -> u32;
    fn days_in_year(&self) -> i32;
    fn is_leap_year(&self) -> bool;
}

impl NaiveDateExt for chrono::NaiveDate {
    fn days_in_month(&self) -> u32 {
        let month = self.month();
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => if self.is_leap_year() { 29 } else { 28 },
            _ => panic!("Invalid month: {}" , month),
        }
    }

    fn days_in_year(&self) -> i32 {
        if self.is_leap_year() { 366 } else { 365 }
    }

    fn is_leap_year(&self) -> bool {
        let year = self.year();
        return year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }
}

fn create_vector(min: u32, max: u32) -> Vec<String> {
    let mut vector: Vec<String> = Vec::new();
    for number in min..=max {
        vector.push(number.to_string());
    }
    vector
}

fn week_vector() -> Vec<String> {
    vec!("Mon".to_string(), "Tue".to_string(), "Wed".to_string(), "Thu".to_string(), "Fri".to_string(), "Sat".to_string(), "Sun".to_string())
}

fn month_vector() -> Vec<String> {
    vec!(
        "1".to_string(),
        "2".to_string(),
        "3".to_string(),
        "4".to_string(),
        "5".to_string(),
        "6".to_string(),
        "7".to_string(),
        "8".to_string(),
        "9".to_string(),
        "10".to_string(),
        "11".to_string(),
        "12".to_string()
    )
}

#[derive(Serialize)]
pub struct BestQuantity{
    title: String,
    main_image: Option<String>,
    overall_quantity: i32
}

#[derive(Serialize)]
pub struct BestProfit{
    title: String,
    main_image: Option<String>,
    overall_profit: i32
}

#[derive(Serialize)]
pub struct StatisticsResponse{
    best_quantity: BestQuantity,
    best_profit: BestProfit,
    prices: Vec<i32>,
    namings: Vec<String>
}

#[debug_handler]
pub async fn full(
    Extension(ScyllaDBConnection{scylla}): Extension<ScyllaDBConnection>,
    Extension(database): Extension<DatabaseConnection>,
    Extension(Auth {user_id, business_id}): Extension<Auth>,
    Query(Search {r#type, prev}): Query<Search>
) -> Response{

    println!("type {:?}", r#type);
    println!("prev {:?}", prev);

    let current_date = Utc::now();

    let (start_date, end_date, namings) = match r#type{
        Types::Week => {
            let days_number_from_monday = current_date.weekday().number_from_monday();
            (
                (current_date + chrono::Duration::days((1 - days_number_from_monday as i32 - (7 * prev) as i32) as i64)).naive_utc().date(),
                (current_date + chrono::Duration::days((7 - days_number_from_monday as i32 - (7 * prev) as i32) as i64)).naive_utc().date(),
                week_vector()
            )

        },
        Types::Month => {
            current_date.naive_utc().date().days_in_month();
            if prev % 12 > (current_date.month() - 1) as u8{
                let working_date = NaiveDate::from_ymd_opt(current_date.year() - (prev / 12) as i32 - 1, 13 - ((prev % 12) - current_date.month() as u8 + 1) as u32, 1).expect("Couldn't convert date");
                (
                    working_date,
                    NaiveDate::from_ymd_opt(working_date.year(), working_date.month(), working_date.days_in_month()).expect("Couldn't convert date"),
                    create_vector(1, working_date.days_in_month())
                )
            } else {
                let working_date = NaiveDate::from_ymd_opt(current_date.year() - (prev / 12) as i32, current_date.month() - (prev % 12) as u32, 1).expect("Couldn't convert date");
                (
                    working_date,
                    NaiveDate::from_ymd_opt(working_date.year(), working_date.month(), working_date.days_in_month()).expect("Couldn't convert date"),
                    create_vector(1, working_date.days_in_month())
                )

            }
        },
        Types::Year => {
            let working_date = NaiveDate::from_ymd_opt(current_date.year() - prev as i32, 1, 1).expect("Couldn't convert date");
            (
                working_date,
                NaiveDate::from_ymd_opt(working_date.year(), 12, 31).expect("Couldn't convert date"),
                month_vector()
            )
        }
    };

    let query = "SELECT parent_id, item_type, SUM(quantity) FROM statistics.products WHERE business_id = ? AND date >= ? AND date <= ? AND item_type IN (1, 2) GROUP BY parent_id, business_id, item_type ALLOW FILTERING";

    let results = scylla.query(
        query,
        (business_id, start_date, end_date)
    ).await.expect("Failed to query");

    let mut max_quantity_parent_id = 8;
    let mut max_quantity = 0;
    let brah = results.rows.expect("");
    println!("{:?}", brah);
    println!("{}", brah.len());
    for row in brah.into_typed::<(i32, i8, i32)>() {
        if let Ok(result) = row{
            let (parent_id, item_type, quantity_sum) = result;
            if quantity_sum > max_quantity{
                max_quantity_parent_id = parent_id;
                max_quantity = quantity_sum;
            }
        }
    }

    let query = "SELECT parent_id, date, item_type, SUM(profit) FROM statistics.products WHERE business_id = ? AND date >= ? AND date <= ? GROUP BY parent_id, business_id, item_type ALLOW FILTERING";

    let results = scylla.query(
        query,
        (business_id, start_date, end_date)
    ).await.expect("Failed to query via scylla");

    let mut max_profit_parent_id = 8;
    let mut max_profit = 0;
    let mut profit_by_date: BTreeMap<NaiveDate, i32> = BTreeMap::new();
    for row in results.rows.expect("failed to get rows").into_typed::<(i32, NaiveDate, i8, i32)>() {
        if let Ok(result) = row{
            let (parent_id, date, item_type, profit) = result;
            if profit > max_profit{
                max_profit_parent_id = parent_id;
                max_profit = profit;
            }
        }
    }

    let query = "SELECT date, SUM(profit) FROM statistics.profits WHERE business_id = ? AND date >= ? AND date <= ? GROUP BY date, business_id ALLOW FILTERING";

    let results = scylla.query(
        query,
        (business_id, start_date, end_date)
    ).await.expect("Failed to query");

    let mut prices: Vec<i32> = Vec::new();
    for row in results.rows.expect("failed to get rows").into_typed::<(NaiveDate, i32)>() {
        let result = row.expect("Raw error in ScyllaDB");
        let (date, profit) = result;
        *profit_by_date.entry(date).or_insert(0) += profit;
    }

    let max_quantity_parent_product = get_object_by_id(&database, max_quantity_parent_id).await.unwrap();
    let max_profit_parent_product = get_object_by_id(&database, max_profit_parent_id).await.unwrap();
    let mut prices: Vec<i32> = (0..namings.len()).map(|_| 0).collect();
    for (date, total_profit) in profit_by_date {
        println!("DATE!!! {}", date);
        match r#type{
            Types::Year => {
                prices[date.month0() as usize] += total_profit
            },
            _ => {
                prices[date.signed_duration_since(start_date).num_days() as usize] += total_profit
            }
        }
    }

    (
        StatusCode::OK,
        Json(
            StatisticsResponse{
                best_quantity: BestQuantity{
                    title: max_quantity_parent_product.title,
                    main_image: max_quantity_parent_product.main_image,
                    overall_quantity: max_quantity
                },
                best_profit: BestProfit{
                    title: max_profit_parent_product.title,
                    main_image: max_profit_parent_product.main_image,
                    overall_profit: max_profit
                },
                prices,
                namings
            }
        )
    ).into_response()

}

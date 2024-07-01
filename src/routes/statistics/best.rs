use std::collections::BTreeMap;
use std::sync::Arc;
use axum::{debug_handler, Extension, Json};
use axum::extract::Query;
use axum::response::{Response, IntoResponse};
use chrono::{NaiveDate, NaiveDateTime, Local, Utc, Datelike, TimeZone};
use http::StatusCode;
use sea_orm::{ColumnTrait, DatabaseConnection, DeriveColumn, DeriveModel, DerivePartialModel, EntityTrait, FromQueryResult, QuerySelect};
use serde::{Deserialize, Serialize};
use crate::core::auth::middleware::{Auth, CustomHeader};
use crate::database::prelude::{Business, ProfitStatistics};
use crate::database::{product_statistics, profit_statistics};
use crate::routes::sell::{EnumValue, ItemType};
use crate::routes::utils::get_parent::{BestProfit, BestQuantity, get_parent_by_id, Parent, ParentGetter, Stats, StatsType};
use crate::routes::statistics::{get_date_range, Search, Types};
use crate::database::prelude::ProductStatistics;
use sea_orm::QueryFilter;


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


#[derive(Serialize)]
pub struct StatisticsResponse{
    best_max_quantity: Vec<MaxQuantity>,
    best_max_profit: Vec<MaxProfit>,
    prices: Vec<f64>,
    namings: Vec<String>
}


#[derive(Debug, PartialEq, FromQueryResult)]
struct PartialProductStats {
    pub parent_id: i32,
    pub quantity_sum: i64,
    pub item_type: i16,
}


#[derive(Debug, PartialEq, FromQueryResult)]
struct PartialProfitProductStats {
    pub parent_id: i32,
    pub total_profit: f64,
    pub item_type: i16,
}


#[derive(Debug, PartialEq, FromQueryResult)]
struct PartialProfitStats {
    pub date: NaiveDate,
    pub profit: f64,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone, Serialize)]
struct MaxQuantity{
    title: String,
    main_image: Option<String>,
    parent_id: i32,
    quantity: i64
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone, Serialize)]
struct MaxProfit{
    title: String,
    main_image: Option<String>,
    parent_id: i32,
    profit: i64
}

// fn main(){
//     let a = HelloWorld{
//         number: 1,
//         title: "Hello".to_owned()
//     };
//     let b = HelloWorld{
//         number: 2,
//         title: "Bye".to_owned()
//     };
//
//
//     println!("{:?}", c);
// }


#[debug_handler]
pub async fn full(
    Extension(database): Extension<DatabaseConnection>,
    Extension(Auth {user_id}): Extension<Auth>,
    Extension(CustomHeader {business_id}): Extension<CustomHeader>,
    Query(Search {r#type, prev, quantity }): Query<Search>
) -> Response{
    let mut has_access = true;
    if let Ok(Some(business)) = Business::find_by_id(business_id).one(&database).await{
        has_access = business.has_full_access;
    }
    println!("type {:?}", r#type);
    println!("prev {:?}", prev);

    let (start_date, end_date, namings) = get_date_range(&r#type, prev);

    // rust query below query
    // SELECT parent_id, item_type, SUM(quantity) FROM statistics.products WHERE business_id = ? AND date >= ? AND date <= ? AND item_type IN (1, 3) GROUP BY parent_id, business_id, item_type
    let products = ProductStatistics::find()
        .filter(
            product_statistics::Column::BusinessId.eq(business_id)
                .and(product_statistics::Column::Date.gte(start_date))
                .and(product_statistics::Column::Date.lte(end_date))
                .and(product_statistics::Column::ItemType.eq(1).or(product_statistics::Column::ItemType.eq(3)))
        )
        .select_only()
        .column(product_statistics::Column::ParentId)
        .column(product_statistics::Column::ItemType)
        .column_as(product_statistics::Column::Quantity.sum(), "quantity_sum")
        .group_by(product_statistics::Column::ParentId)
        .group_by(product_statistics::Column::ItemType)
        .into_model::<PartialProductStats>()
        .all(&database)
        .await
        .unwrap();

    let mut best_max_quantity = vec![];
    for product in products{
        let parent: Parent = get_parent_by_id(&database, product.parent_id, ItemType::from_value(product.item_type)).await.unwrap();
        let (title, main_image) = match parent {
            Parent::Product(model) => {
                (model.title, model.main_image)
            },
            Parent::WeightItem(model) => {
                (model.title, model.main_image)
            },
            Parent::NoCodeProduct(model) => {
                (model.title, model.main_image)
            }
        };
        best_max_quantity.push(MaxQuantity {
            title,
            main_image,
            parent_id: product.parent_id,
            quantity: product.quantity_sum,
        });
    }

    best_max_quantity.sort_by(|a, b| b.quantity.cmp(&a.quantity));




    // convert the below query to rust sea-orm
    // SELECT parent_id, date, item_type, SUM(profit) FROM statistics.products WHERE business_id = ? AND date >= ? AND date <= ? GROUP BY parent_id, business_id, item_type

    let products = ProductStatistics::find()
        .filter(
            product_statistics::Column::BusinessId.eq(business_id)
                .and(product_statistics::Column::Date.gte(start_date))
                .and(product_statistics::Column::Date.lte(end_date))
        )
        .group_by(product_statistics::Column::ParentId)
        .group_by(product_statistics::Column::ItemType)
        .select_only()
        .column(product_statistics::Column::ParentId)
        .column(product_statistics::Column::ItemType)
        .column_as(product_statistics::Column::Profit.sum(), "total_profit")
        .into_model::<PartialProfitProductStats>()
        .all(&database)
        .await
        .unwrap();

    let mut best_max_profit = vec![];
    let mut profit_by_date: BTreeMap<NaiveDate, f64> = BTreeMap::new();
    for product in products{
        let parent: Parent = get_parent_by_id(&database, product.parent_id, ItemType::from_value(product.item_type)).await.unwrap();
        let (title, main_image) = match parent {
            Parent::Product(model) => {
                (model.title, model.main_image)
            },
            Parent::WeightItem(model) => {
                (model.title, model.main_image)
            },
            Parent::NoCodeProduct(model) => {
                (model.title, model.main_image)
            }
        };
        best_max_profit.push(
            MaxProfit{
                title,
                main_image,
                parent_id: product.parent_id,
                profit: product.total_profit as i64
            }
        )
    }

    best_max_profit.sort_by(|a, b| b.profit.cmp(&a.profit));

    // convert the below query to rust sea-orm
    // SELECT date, SUM(profit) FROM statistics.profits WHERE business_id = ? AND date >= ? AND date <= ? GROUP BY date, business_id ALLOW FILTERING

    let profits = ProfitStatistics::find()
        .filter(
            profit_statistics::Column::BusinessId.eq(business_id)
                .and(profit_statistics::Column::Date.gte(start_date))
                .and(profit_statistics::Column::Date.lte(end_date))
        )
        .select_only()
        .column(profit_statistics::Column::Date)
        .column(profit_statistics::Column::Profit)
        .into_model::<PartialProfitStats>()
        .all(&database)
        .await
        .unwrap();

    for profit in profits {
        *profit_by_date.entry(profit.date).or_insert(0f64) += profit.profit;
    }


    let mut prices: Vec<f64> = (0..namings.len()).map(|_| 0f64).collect();
    if has_access {
        for (date, total_profit) in profit_by_date {
            println!("DATE!!! {}", date);
            match r#type {
                Types::Year => {
                    prices[date.month0() as usize] += total_profit
                },
                _ => {
                    prices[date.signed_duration_since(start_date).num_days() as usize] += total_profit
                }
            }
        }
    }


    (
        StatusCode::OK,
        Json(
            StatisticsResponse {
                best_max_quantity: best_max_quantity[..best_max_quantity.len().min(quantity as usize)].to_vec(),
                best_max_profit: best_max_profit[..best_max_profit.len().min(quantity as usize)].to_vec(),
                prices,
                namings
            }
        )
    ).into_response()

}

use axum::{debug_handler, Extension, Json};
use axum::extract::Path;
use http::StatusCode;
use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder, QuerySelect};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::core::auth::middleware::{Auth, CustomHeader};
use crate::database::prelude::RentHistory;
use crate::database::rent_history;
use axum::response::{IntoResponse, Response};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use crate::routes::sell::{ItemType, History as RentHistoryProducts};
use crate::routes::utils::get_parent::{get_parent_by_id, Parent};
use axum::extract::Query;



const DEFAULT_PAGE_SIZE: i32 = 5;
const DEFAULT_PAGE: i32 = 1;

#[derive(Serialize, Deserialize, Debug)]
struct History{
    id: i32,
    purchase_products: DetailedHistory,
    buy_date: DateTimeWithTimeZone,
    grant_total: f64
}

#[derive(Serialize, Deserialize, Debug)]
struct DetailedProduct{
    id: i32,
    title: String,
    main_image: Option<String>,
    quantity: i32,
    sell_price: f64
}
#[derive(Serialize, Deserialize, Debug)]
struct DetailedWeightItem{
    id: i32,
    title: String,
    main_image: Option<String>,
    kg_weight: f64,
    sell_price: f64
}

#[derive(Serialize, Deserialize, Debug)]
struct DetailedNoCodeProduct{
    id: i32,
    title: String,
    main_image: Option<String>,
    quantity: i32,
    sell_price: f64
}

#[derive(Serialize, Deserialize, Debug)]
struct DetailedHistory{
    products: Vec<DetailedProduct>,
    weight_items: Vec<DetailedWeightItem>,
    no_code_products: Vec<DetailedNoCodeProduct>
}


#[derive(Serialize, Deserialize, Debug)]
struct Histories{
    histories: Vec<History>
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Pagination{
    page: Option<i32>,
    page_size: Option<i32>
}

#[debug_handler]
pub async fn get_history(
    Extension(Auth{user_id}): Extension<Auth>,
    Extension(database): Extension<DatabaseConnection>,
    Path(id): Path<i32>,
    Query(Pagination{page, page_size}): Query<Pagination>
) -> Response{
    // ToDo: Check RentUser for business_id for security purposes
    let histories = RentHistory::find()
        .filter(
            rent_history::Column::RentUserId.eq(id)
        )
        .order_by_desc(rent_history::Column::BuyDate)
        .offset(((page.unwrap_or(DEFAULT_PAGE) - 1) * page_size.unwrap_or(DEFAULT_PAGE_SIZE)) as u64)
        .limit(page_size.unwrap_or(DEFAULT_PAGE_SIZE) as u64)
        .all(&database)
        .await
        .unwrap();

    let mut response_body = Histories{histories: vec![]};

    for history in histories{
        let products_str = history.products.to_string();
        // Now, parse the string into the Products struct
        let products = match serde_json::from_str::<RentHistoryProducts>(&products_str) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to parse products JSON: {:?}", e);
                continue; // Handle the error as needed
            }
        };

        let mut detail_products: Vec<DetailedProduct> = vec![];
        for product in products.products{
            if let Ok(Parent::Product(parent_product)) = get_parent_by_id(&database, product.parent_id, ItemType::Product).await{
                detail_products.push(DetailedProduct{
                    id: parent_product.id,
                    title: parent_product.title,
                    main_image: parent_product.main_image,
                    quantity: product.quantity,
                    sell_price: product.sell_price
                })
            }
        }

        let mut detail_weight_items: Vec<DetailedWeightItem> = vec![];
        for product in products.weight_items{
            if let Ok(Parent::WeightItem(parent_product)) = get_parent_by_id(&database, product.parent_id, ItemType::WeightItem).await{
                detail_weight_items.push(DetailedWeightItem{
                    id: parent_product.id,
                    title: parent_product.title,
                    main_image: parent_product.main_image,
                    kg_weight: product.kg_weight,
                    sell_price: product.sell_price
                })
            }
        }

        let mut detail_no_code_products: Vec<DetailedNoCodeProduct> = vec![];
        for product in products.no_code_products{
            if let Ok(Parent::WeightItem(parent_product)) = get_parent_by_id(&database, product.parent_id, ItemType::NoCodeProduct).await{
                detail_no_code_products.push(DetailedNoCodeProduct{
                    id: parent_product.id,
                    title: parent_product.title,
                    main_image: parent_product.main_image,
                    quantity: product.quantity,
                    sell_price: product.sell_price
                })
            }
        }

        let detailed_history = DetailedHistory{
            products: detail_products,
            weight_items: detail_weight_items,
            no_code_products: detail_no_code_products
        };

        response_body.histories.push(History{
            id: history.id,
            purchase_products: detailed_history,
            buy_date: history.buy_date,
            grant_total: history.grand_total
        })
    }

    (
        StatusCode::OK,
        Json(response_body)
    ).into_response()
}
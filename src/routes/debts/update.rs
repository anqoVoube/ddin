use std::sync::Arc;
use axum::{Extension, Json, debug_handler};
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use log::{error, info};

use scylla::{IntoTypedRows, Session};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::{DateTimeUtc, DateTimeWithTimeZone};
use serde_json::json;
use crate::core::auth::middleware::{Auth, CustomHeader};
use crate::database::prelude::{NoCodeProduct, Rent, WeightItem};
use crate::database::product::Entity as Product;
use crate::database::{no_code_product, product, rent, rent_history, weight_item};
use crate::routes::utils::{not_found, bad_request, internal_server_error, default_created, default_ok};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductBody {
    id: i32,
    quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentProductBody {
    parent_id: i32,
    quantity: i32,
    sell_price: f64
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoCodeProductBody {
    id: i32,
    quantity: i32,
    sell_price: f64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentNoCodeProductBody {
    parent_id: i32,
    quantity: i32,
    sell_price: f64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightItemBody {
    id: i32,
    kg_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentWeightItemBody {
    parent_id: i32,
    kg_weight: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebtUserBody{
    id: i32,
    paid_price: f64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayBody {
    id: i32,
    paid_price: f64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RentHistoryProducts{
    weight_items: Vec<ParentWeightItemBody>,
    products: Vec<ParentProductBody>,
    no_code_products: Vec<ParentNoCodeProductBody>,
}

pub enum ItemType{
    Product,
    WeightItem,
    NoCodeProduct
}

pub trait EnumValue{
    fn get_value(&self) -> i8;
    fn from_value(value: i8) -> Self;
}

impl EnumValue for ItemType{
    fn get_value(&self) -> i8 {
        match self {
            ItemType::Product => 1,
            ItemType::WeightItem => 2,
            ItemType::NoCodeProduct => 3
        }
    }

    fn from_value(value: i8) -> Self {
        match value {
            1 => ItemType::Product,
            2 => ItemType::WeightItem,
            3 => ItemType::NoCodeProduct,
            _ => panic!("Invalid item type")
        }
    }
}
#[debug_handler]
pub async fn update(
    Extension(database): Extension<DatabaseConnection>,
    Extension(Auth{user_id}): Extension<Auth>,
    Json(PayBody{id, paid_price}): Json<PayBody>
) -> Response {
    // TODO: auth check for security purposes

    match Rent::find_by_id(id).one(&database).await{
        Ok(Some(pear)) => {
            let mut pear: rent::ActiveModel = pear.into();
            let total = pear.price.clone().unwrap();
            let new_debt = total - paid_price;
            pear.price = Set(new_debt);
            if let Err(err) = pear.update(&database).await {
                println!("{:?}", err);
                return internal_server_error();
            }
            let new_rent_history = rent_history::ActiveModel {
                grand_total: Set(0f64),
                paid_amount: Set(paid_price),
                products: Set(json!(RentHistoryProducts{
                    products: vec![],
                    weight_items: vec![],
                    no_code_products: vec![],
                })),
                buy_date: Set(DateTimeWithTimeZone::from(chrono::Utc::now())),
                ..Default::default()
            };
            match new_rent_history.save(&database).await {
                Ok(instance) => {
                    info!("{:?}", instance);
                },
                Err(error) => {
                    error!("Unable to create {:?}. Original error was {}", 1, error);
                }
            }
        },
        Ok(None) => {
            return not_found();
        },
        Err(err) => {
            println!("{:?}", err);
            return internal_server_error();
        }
    }

    default_ok()
}

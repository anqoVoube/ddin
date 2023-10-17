use axum::{Extension, Json, debug_handler};
use axum::response::Response;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use log::{error, info};
use sea_orm::ActiveValue::Set;
use crate::database::product;
use crate::routes::parent_product::fetch::get_object_by_id;
use crate::routes::utils::{default_created, internal_server_error};

fn default_as_false() -> bool {
    false
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    parent_id: i64,
    quantity: Option<u16>,
    orig_price: i64,
    price: i64,
    produced_date: NaiveDate,
    business_id: Option<i64>,
}

#[debug_handler]
pub async fn create(
    Extension(database): Extension<DatabaseConnection>,
    Json(product): Json<Product>
) -> Response {
    println!("{:#?}", product);
    match get_object_by_id(&database, product.parent_id).await{
        Ok(parent_product) => {
            println!("{:?}", parent_product);
            let new_product = product::ActiveModel {
                price: Set(product.price),
                expiration_date: Set(product.produced_date + chrono::Duration::days(parent_product.expiration_in_days as i64)),
                business_id: Set(product.business_id.unwrap_or(2)),
                quantity: Set(product.quantity.unwrap_or(1) as i32),
                parent_product_id: Set(product.parent_id),
                ..Default::default()
            };

            match new_product.save(&database).await {
                Ok(instance) => {
                    info!("{:?}", instance);
                    return default_created();
                },
                Err(error) => {
                    error!("Unable to create {:?}. Original error was {}", 1, error);
                    return internal_server_error();
                }
            }
        },
        Err(_) => {
            return internal_server_error();
        }
    }
}

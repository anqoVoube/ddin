use axum::{Extension, Json, debug_handler};
use axum::response::Response;
use sea_orm::{ActiveModelTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use log::{error, info};
use sea_orm::ActiveValue::Set;
use crate::core::auth::middleware::{Auth, CustomHeader};
use crate::database::prelude::Product;
use crate::database::product;
use crate::routes::parent_product::fetch::get_object_by_id;
use crate::routes::utils::{default_created, internal_server_error};
use sea_orm::ColumnTrait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    parent_id: i32,
    quantity: Option<u16>,
    orig_price: i32,
    price: i32,
    produced_date: Option<NaiveDate>,
    expiration_date: Option<NaiveDate>,
}


#[debug_handler]
pub async fn create(
    Extension(database): Extension<DatabaseConnection>,
    Extension(auth): Extension<Auth>,
    Extension(CustomHeader{business_id}): Extension<CustomHeader>,
    Json(Body {parent_id, quantity, orig_price, price, produced_date, expiration_date}): Json<Body>
) -> Result<Response, Response> {
    println!("{} {:?} {} {} {:?}", parent_id, quantity, orig_price, price, produced_date);
    match get_object_by_id(&database, parent_id).await{
        Ok(parent_product) => {
            let expiration_date: NaiveDate = match expiration_date {
                Some(date) => date,
                _ => produced_date.unwrap() + chrono::Duration::days(parent_product.expiration_in_days as i64)
            };
            match Product::find()
                .filter(
                    Condition::all()
                        .add(product::Column::BusinessId.eq(business_id))
                        .add(product::Column::ExpirationDate.eq(expiration_date))
                        .add(product::Column::ParentId.eq(parent_product.id))
                )
                .one(&database)
                .await.unwrap()
            {
                Some(product) => {
                    let adding_quantity = product.clone().quantity;
                    let mut product: product::ActiveModel = product.into();
                    product.quantity = Set(adding_quantity + quantity.unwrap_or(1) as i32);
                    match product.update(&database).await {
                        Ok(_) => {
                            Ok(default_created())
                        },
                        Err(error) => {
                            error!("Couldn't update product with id. Original error is: {}", error);
                            Err(internal_server_error())
                        }
                    }
                },
                None => {
                    let new_product = product::ActiveModel {
                        price: Set(price),
                        profit: Set(price - orig_price),
                        expiration_date: Set(Some(expiration_date)),
                        business_id: Set(business_id),
                        quantity: Set(quantity.unwrap_or(1) as i32),
                        parent_id: Set(parent_id),
                        ..Default::default()
                    };

                    match new_product.save(&database).await {
                        Ok(instance) => {
                            info!("{:?}", instance);
                            Ok(default_created())
                        },
                        Err(error) => {
                            error!("Unable to create {:?}. Original error was {}", 1, error);
                            Err(internal_server_error())
                        }
                    }
                }
            }
        },
        Err(error) => {
            error!("Couldn't fetch parent_product with id: {}. Original error is: {}", parent_id, error);
            Err(internal_server_error())
        }
    }
}

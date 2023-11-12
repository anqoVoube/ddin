use axum::{debug_handler, Extension, Json};
use axum::response::{Response, IntoResponse};
use chrono::NaiveDate;
use http::StatusCode;
use log::{error, info};
use sea_orm::{ActiveModelTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use crate::database::parent_no_code_product::Model as ParentNoCodeProductModel;
use crate::database::prelude::{ParentNoCodeProduct, NoCodeProduct};
use crate::database::no_code_product;
use sea_orm::ColumnTrait;
use crate::core::auth::middleware::Auth;
use crate::database::weight_item;
use sea_orm::ActiveValue::Set;
use crate::routes::utils::{default_created, internal_server_error};

#[derive(Clone, Deserialize)]
pub struct Body {
    parent_id: i32,
    price: i32,
    orig_price: i32,
    quantity: i32,
    produced_date: Option<NaiveDate>
}

pub async fn get_object_by_id(database: &DatabaseConnection, id: i32) -> Result<ParentNoCodeProductModel, StatusCode> {
    let product = ParentNoCodeProduct::find_by_id(id).one(database).await
        .map_err(|_error| {error!("Couldn't fetch parent_product with id: {}", id); StatusCode::INTERNAL_SERVER_ERROR})?;

    if let Some(value) = product{
        Ok(value)
    }
    else{
        info!("Not found parent_product with id: {}", &id);
        Err(StatusCode::NOT_FOUND)
    }
}

#[debug_handler]
pub async fn create(
    Extension(database): Extension<DatabaseConnection>,
    Extension(auth): Extension<Auth>,
    Json(Body {parent_id, price, orig_price, quantity, produced_date}): Json<Body>
) -> Response {
    println!("{} {:?} {} {} {:?}", parent_id, quantity, orig_price, price, produced_date);
    match get_object_by_id(&database, parent_id).await{
        Ok(parent) => {
            let mut expiration_date = None;
            if let Some(produced_date) = produced_date{
                expiration_date = Some(produced_date + chrono::Duration::days(parent.expiration_in_days as i64));
            }
            println!("{:?}", parent);
            match NoCodeProduct::find()
                .filter(
                    Condition::all()
                        .add(no_code_product::Column::BusinessId.eq(auth.business_id))
                        .add(no_code_product::Column::ExpirationDate.eq(expiration_date))
                        .add(no_code_product::Column::ParentId.eq(parent.id))
                )
                .one(&database)
                .await.unwrap()
            {
                Some(item) => {
                    let adding_quantity = item.clone().quantity;
                    let mut item: no_code_product::ActiveModel = item.into();
                    item.quantity = Set(adding_quantity + quantity);
                    match item.update(&database).await {
                        Ok(_) => {
                            default_created()
                        },
                        Err(error) => {
                            error!("Couldn't update weight item with id. Original error is: {}", error);
                            internal_server_error()
                        }
                    }
                },
                None => {
                    let new_product = no_code_product::ActiveModel {
                        price: Set(price),
                        expiration_date: Set(expiration_date),
                        business_id: Set(auth.business_id),
                        quantity: Set(quantity),
                        parent_id: Set(parent_id),
                        profit: Set(price - orig_price),
                        ..Default::default()
                    };

                    match new_product.save(&database).await {
                        Ok(instance) => {
                            info!("{:?}", instance);
                            default_created()
                        },
                        Err(error) => {
                            error!("Unable to create {:?}. Original error was {}", 1, error);
                            internal_server_error()
                        }
                    }
                }
            }
        },
        Err(error) => {
            error!("Couldn't fetch parent_weight_item with id. Original error is: {}", error);
            internal_server_error()
        }
    }
}

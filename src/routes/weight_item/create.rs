use axum::{debug_handler, Extension, Json};
use axum::response::{Response, IntoResponse};
use chrono::NaiveDate;
use http::StatusCode;
use log::{error, info};

use sea_orm::{ActiveModelTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use crate::routes::{AppConnections};
use crate::database::parent_weight_item::Model as ParentWeightItemModel;
use crate::database::prelude::{ParentWeightItem, WeightItem};
use sea_orm::ColumnTrait;
use crate::core::auth::middleware::{Auth, CustomHeader};
use crate::database::weight_item;
use sea_orm::ActiveValue::Set;
use crate::routes::utils::{default_created, internal_server_error};

#[derive(Clone, Deserialize)]
pub struct Body {
    parent_id: i32,
    price: f64,
    orig_price: f64,
    kg_weight: f64,
    expiration_date: Option<NaiveDate>
}

pub async fn get_object_by_id(database: &DatabaseConnection, id: i32) -> Result<ParentWeightItemModel, StatusCode> {
    let parent_weight_item = ParentWeightItem::find_by_id(id).one(database).await
        .map_err(|_error| {error!("Couldn't fetch parent_product with id: {}", id); StatusCode::INTERNAL_SERVER_ERROR})?;

    if let Some(value) = parent_weight_item{
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
    Extension(headers): Extension<CustomHeader>,
    Json(Body {parent_id, price, orig_price, kg_weight, expiration_date}): Json<Body>
) -> Response {
    println!("{} {:?} {} {} {:?}", parent_id, kg_weight, orig_price, price, expiration_date);
    match get_object_by_id(&database, parent_id).await{
        Ok(parent_weight_item) => {
            println!("{:?}", parent_weight_item);
            match WeightItem::find()
                .filter(
                    Condition::all()
                        .add(weight_item::Column::BusinessId.eq(headers.business_id))
                        .add(weight_item::Column::ExpirationDate.eq(expiration_date))
                        .add(weight_item::Column::ParentId.eq(parent_weight_item.id))
                )
                .one(&database)
                .await.unwrap()
            {
                Some(item) => {
                    let adding_weight = item.clone().kg_weight;
                    let mut item: weight_item::ActiveModel = item.into();
                    item.kg_weight = Set(adding_weight + kg_weight);
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
                    let new_product = weight_item::ActiveModel {
                        price: Set(price),
                        expiration_date: Set(expiration_date),
                        business_id: Set(headers.business_id),
                        kg_weight: Set(kg_weight),
                        parent_id: Set(parent_id),
                        profit: Set(price - orig_price),
                        is_accessible: Set(true),
                        discount: Set(0),
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
            println!("104");
            internal_server_error()
        }
    }
}

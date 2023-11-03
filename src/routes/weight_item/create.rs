use axum::{Extension, Json};
use axum::response::{Response, IntoResponse};
use chrono::NaiveDate;
use http::StatusCode;
use log::{error, info};
use sea_orm::{Condition, DatabaseConnection, EntityTrait};
use serde::Serialize;
use crate::routes::{AppConnections};
use crate::database::parent_weight_item::Model as ParentWeightItemModel;
use crate::database::prelude::{ParentWeightItem, WeightItem};
use sea_orm::ColumnTrait;
use crate::core::auth::middleware::Auth;
use crate::database::weight_item;

#[derive(Clone, Serialize)]
pub struct Body {
    parent_weight_item_id: i32,
    price: i32,
    orig_price: i32,
    kg_weight: f32,
    produced_date: Option<NaiveDate>
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
//
// pub async fn create(
//     Extension(connections): Extension<AppConnections>,
//     Extension(auth): Extension<Auth>,
//     Json(Body {parent_weight_item_id, price, orig_price, kg_weight, produced_date}): Json<Body>
// ) -> Response {
//     println!("{} {:?} {} {} {:?}", parent_weight_item_id, kg_weight, orig_price, price, produced_date);
//     match get_object_by_id(&connections.database, parent_weight_item_id).await{
//         Ok(parent_product) => {
//             let expiration_date = produced_date + chrono::Duration::days(parent_product.expiration_in_days as i64);
//             println!("{:?}", parent_product);
//             match WeightItem::find()
//                 .filter(
//                     Condition::all()
//                         .add(weight_item::Column::BusinessId.eq(auth.business_id))
//                         .add(weight_item::Column::ExpirationDate.eq(expiration_date))
//                         .add(weight_item::Column::ParentWeightItemId.eq(parent_product.id))
//                 )
//                 .one(&connections.database)
//                 .await.unwrap()
//             {
//                 Some(item) => {
//                     let adding_weight = item.clone().weight;
//                     let mut item: weight_item::ActiveModel = item.into();
//                     item.kg_weight = Set(adding_weight +  kg_weight.unwrap_or(1) as f32);
//                     match product.update(&database).await {
//                         Ok(_) => {
//                             return Ok(default_created());
//                         },
//                         Err(error) => {
//                             error!("Couldn't update product with id. Original error is: {}", error);
//                             return Err(internal_server_error());
//                         }
//                     }
//                 },
//                 None => {
//                     let new_product = product::ActiveModel {
//                         price: Set(price),
//                         expiration_date: Set(produced_date + chrono::Duration::days(parent_product.expiration_in_days as i64)),
//                         business_id: Set(auth.business_id),
//                         quantity: Set(quantity.unwrap_or(1) as i32),
//                         parent_product_id: Set(parent_id),
//                         ..Default::default()
//                     };
//
//                     match new_product.save(&database).await {
//                         Ok(instance) => {
//                             info!("{:?}", instance);
//                             return Ok(default_created());
//                         },
//                         Err(error) => {
//                             error!("Unable to create {:?}. Original error was {}", 1, error);
//                             return Err(internal_server_error());
//                         }
//                     }
//                 }
//             }
//         },
//         Err(error) => {
//             error!("Couldn't fetch parent_product with id: {}. Original error is: {}", parent_id, error);
//             return Err(internal_server_error());
//         }
//     }
// }

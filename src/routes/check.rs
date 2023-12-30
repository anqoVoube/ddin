// use axum::Extension;
// use sea_orm::{Condition, DatabaseConnection, EntityTrait};
// use crate::core::auth::middleware::{Auth, CustomHeader};
// use axum::response::{IntoResponse, Response};
// use axum::{extract::Query, debug_handler};
// use http::StatusCode;
// use sea_orm::sea_query::IntoColumnRef;
// use crate::database::{parent_no_code_product, parent_product, parent_weight_item, product};
// use crate::database::prelude::{ParentNoCodeProduct, ParentProduct, ParentWeightItem};
// use crate::routes::utils::bad_request;
//
// pub struct Search{
//     pub title: String,
//     pub item_type: i8
// }
//
// #[debug_handler]
// pub async fn check_title_uniqueness(
//     Extension(Auth{user_id}): Extension<Auth>,,
//     Extension(database): Extension<DatabaseConnection>,
//     Query(Search{title, item_type}): Query<Search>
// ) -> Response {
//     match item_type{
//         1 => {
//             let condition = get_condition(&title, business_id, parent_product::Column::Title, parent_product::Column::BusinessId);
//             let product = ParentProduct::find()
//                 .filter(condition)
//                 .one(&database).await;
//             match product{
//                 Ok(Some(_)) => {
//                     (
//                         StatusCode::NOT_ACCEPTABLE,
//                         "Product with this title already exists"
//                     ).into_response()
//                 },
//                 _ => {
//                     (
//                         StatusCode::OK,
//                         "Product with this title doesn't exist"
//                     ).into_response()
//                 }
//             }
//         },
//
//         2 => {
//             let condition = get_condition(&title, business_id, parent_weight_item::Column::Title, parent_weight_item::Column::BusinessId);
//             let parent_weight_item = ParentWeightItem::find()
//                 .filter(condition)
//                 .one(&database).await;
//
//             match parent_weight_item{
//                 Ok(Some(_)) => {
//                     (
//                         StatusCode::NOT_ACCEPTABLE,
//                         "Parent weight item with this title already exists"
//                     ).into_response()
//                 },
//                 _ => {
//                     (
//                         StatusCode::OK,
//                         "Parent weight item with this title doesn't exist"
//                     ).into_response()
//                 }
//             }
//         },
//         3 => {
//             let condition = get_condition(&title, business_id, parent_no_code_product::Column::Title, parent_no_code_product::Column::BusinessId);
//             let parent_no_code_product = ParentNoCodeProduct::find()
//                 .filter(condition)
//                 .one(&database).await;
//             match parent_no_code_product{
//                 Ok(Some(_)) => {
//                     (
//                         StatusCode::NOT_ACCEPTABLE,
//                         "Parent no code product with this title already exists"
//                     ).into_response()
//                 },
//                 _ => {
//                     (
//                         StatusCode::OK,
//                         "Parent no code product with this title doesn't exist"
//                     ).into_response()
//                 }
//             }
//         },
//         _ => {
//             return bad_request("Item type is not valid");
//         }
//     }
// }
//
//
// pub fn get_condition<T>(title: &str, business_id: i32, title_column: T, business_column: T) -> Condition
//     where T: IntoColumnRef + sea_orm::ColumnTrait{
//     let condition = Condition::all()
//             .add(
//             Condition::any()
//                 .add(business_column.eq(business_id))
//                 .add(business_column.is_null())
//             )
//             .add(
//                 Condition::all()
//                     .add(title_column.eq(title))
//             );
//
//     condition
// }


use sea_orm::QueryFilter;
use axum::{Extension, extract::Query, response::{IntoResponse, Response}, http::StatusCode, debug_handler};
use sea_orm::{DatabaseConnection, EntityTrait, Condition, sea_query::IntoColumnRef};
use serde::{Deserialize, Serialize};
use crate::core::auth::middleware::{Auth, CustomHeader};
use crate::database::{parent_no_code_product, parent_product, parent_weight_item};
use crate::database::prelude::{ParentNoCodeProduct, ParentProduct, ParentWeightItem};
use crate::routes::utils::{bad_request, default_ok, not_acceptable};
use crate::routes::utils::item_type::ItemType;

#[derive(Serialize, Deserialize, Debug)]
pub struct Search {
    pub title: String,
    pub item_type: ItemType,
}


#[debug_handler]
pub async fn check_title_uniqueness(
    Extension(Auth { user_id}): Extension<Auth>,
    Extension(CustomHeader {business_id}): Extension<CustomHeader>,
    Extension(database): Extension<DatabaseConnection>,
    Query(Search { title, item_type }): Query<Search>
) -> Response {
    match item_type {
        ItemType::ParentProduct => {
            check_uniqueness::<ParentProduct, _>(
                &database,
                &title,
                business_id,
                parent_product::Column::Title,
                parent_product::Column::BusinessId
            ).await
        },
        ItemType::ParentWeightItem => {
            check_uniqueness::<ParentWeightItem, _>(
                &database,
                &title,
                business_id,
                parent_weight_item::Column::Title,
                parent_weight_item::Column::BusinessId
            ).await
        },
        ItemType::ParentNoCodeProduct => {
            check_uniqueness::<ParentNoCodeProduct, _>(
                &database,
                &title,
                business_id,
                parent_no_code_product::Column::Title,
                parent_no_code_product::Column::BusinessId
            ).await
        },
    }
}

async fn check_uniqueness<T, C>(
    database: &DatabaseConnection,
    title: &str,
    business_id: i32,
    title_column: C,
    business_column: C
) -> Response
    where
        T: EntityTrait + 'static,
        C: IntoColumnRef + sea_orm::ColumnTrait,
{
    let condition = get_condition(title, business_id, title_column, business_column);
    let exists = T::find().filter(condition).one(database).await.is_ok();

    if exists {
        not_acceptable("Title already exists")
    } else {
        default_ok()
    }
}

pub fn get_condition<T>(title: &str, business_id: i32, title_column: T, business_column: T) -> Condition
    where T: IntoColumnRef + sea_orm::ColumnTrait{
    let condition = Condition::all()
        .add(
            Condition::any()
                .add(business_column.eq(business_id))
                .add(business_column.is_null())
        )
        .add(
            Condition::all()
                .add(title_column.eq(title))
        );

    condition
}
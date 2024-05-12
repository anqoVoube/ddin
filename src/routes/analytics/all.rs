use axum::{Extension, Json};
use axum::response::{Response, IntoResponse};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, FromJsonQueryResult, FromQueryResult, QueryFilter};
use http::StatusCode;
use crate::core::auth::middleware::{Auth, CustomHeader};

use sea_orm::entity::*;
use sea_orm::query::*;
use serde::{Deserialize, Serialize};
use crate::database::prelude::{Rent, WeightItem, NoCodeProduct, Product};
use crate::database::{no_code_product, product, rent, weight_item};

const DEFAULT_PAGE_SIZE: i32 = 15;
const DEFAULT_PAGE: i32 = 1;

#[derive(Serialize,  Deserialize, Debug)]
pub struct Analytics{
    total_debts: f32,
    evaluation: f32
}

#[derive(FromQueryResult)]
pub struct Sum {
    sum: f32
}

#[derive(FromQueryResult)]
pub struct ProductInfo {
    price: f32,
    quantity: f32
}

#[derive(FromQueryResult)]
pub struct WeightItemInfo {
    price: f32,
    kg_weight: f32
}


pub async fn get_all_analytics(
    Extension(auth): Extension<Auth>,
    Extension(headers): Extension<CustomHeader>,
    Extension(database): Extension<DatabaseConnection>,
) -> Response{
    let debt_condition = Condition::all()
        .add(rent::Column::BusinessId.eq(headers.business_id));

    let weight_item_condition = Condition::all()
        .add(weight_item::Column::BusinessId.eq(headers.business_id));

    let no_code_product_condition = Condition::all()
        .add(no_code_product::Column::BusinessId.eq(headers.business_id));

    let product_condition = Condition::all()
        .add(product::Column::BusinessId.eq(headers.business_id));


    let total_amount: Option<Sum> = Rent::find()
        .filter(
            debt_condition
        )
        .select_only()
        .column_as(rent::Column::Price.sum(), "sum")
        .into_model::<Sum>()
        .one(&database)
        .await
        .unwrap();

    let weight_items: Vec<WeightItemInfo> = WeightItem::find()
        .filter(
            weight_item_condition
        )
        .select_only()
        .column_as(weight_item::Column::Price.sum(), "price")
        .column_as(weight_item::Column::KgWeight.sum(), "kg_weight")
        .into_model::<WeightItemInfo>()
        .all(&database)
        .await
        .unwrap();

    let no_code_products: Vec<ProductInfo> = NoCodeProduct::find()
        .filter(
            no_code_product_condition
        )
        .select_only()
        .column_as(no_code_product::Column::Price.sum(), "price")
        .column_as(no_code_product::Column::Quantity.sum(), "quantity")
        .into_model::<ProductInfo>()
        .all(&database)
        .await
        .unwrap();

    let products: Vec<ProductInfo> = Product::find()
        .filter(
            product_condition
        )
        .select_only()
        .column_as(product::Column::Price.sum(), "price")
        .column_as(product::Column::Quantity.sum(), "quantity")
        .into_model::<ProductInfo>()
        .all(&database)
        .await
        .unwrap();


    let total_weight_item_evaluation = weight_items.iter().fold(0.0, |acc, item| acc + item.price * item.kg_weight);
    let total_no_code_product_evaluation = no_code_products.iter().fold(0.0, |acc, item| acc + item.price * item.quantity);
    let total_product_evaluation = products.iter().fold(0.0, |acc, item| acc + item.price * item.quantity);

    let json_response = Analytics{
        total_debts: total_amount.unwrap_or(Sum {sum: 0f32}).sum,
        evaluation: total_weight_item_evaluation + total_no_code_product_evaluation + total_product_evaluation
    };

    (
        StatusCode::OK,
        Json(json_response)
    ).into_response()
}

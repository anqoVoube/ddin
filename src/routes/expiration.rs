use axum::{debug_handler, Extension, Json};
use serde::{Serialize, Deserialize};
use sea_orm::{Condition, DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use http::StatusCode;

use crate::core::auth::middleware::{Auth, CustomHeader};
use crate::database::prelude::{Product, WeightItem, NoCodeProduct, ParentProduct, ParentWeightItem, ParentNoCodeProduct};
use crate::database::{no_code_product, product, weight_item};

#[derive(Serialize, Debug)]
pub struct AllExpired{
    products: Vec<ExpiredProducts>,
    weight_items: Vec<ExpiredWeightItems>,
    no_code_products: Vec<ExpiredNoCodeProducts>,
}

#[derive(Serialize, Debug)]
pub struct ExpiredProducts{
    id: i32,
    title: String,
    quantity: i32,
    main_image: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ExpiredWeightItems{
    id: i32,
    title: String,
    kg_weight: f64,
    main_image: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ExpiredNoCodeProducts{
    id: i32,
    title: String,
    quantity: i32,
    main_image: Option<String>,
}


#[debug_handler]
pub async fn get_expirations(
    Extension(Auth{user_id}): Extension<Auth>,
    Extension(CustomHeader{business_id}): Extension<CustomHeader>,
    Extension(database): Extension<DatabaseConnection>,
) -> Response{
    let mut all_expired = AllExpired{
        products: vec!(),
        weight_items: vec!(),
        no_code_products: vec!(),
    };
    // product, weight item, no code product
    let today = Utc::now().naive_utc().date();
    println!("{}", today);
    let condition = Condition::all()
        .add(product::Column::BusinessId.eq(business_id))
        .add(product::Column::ExpirationDate.eq(today));
    let products = Product::find()
        .find_with_related(ParentProduct)
        .filter(condition)
        .all(&database)
        .await
        .unwrap();

    for (product, vec_parent) in products {
        let parent = vec_parent.first().unwrap();
        all_expired.products.push(ExpiredProducts{
            id: product.id,
            title: parent.title.clone(),
            quantity: product.quantity,
            main_image: parent.main_image.clone(),
        })
    }


    let condition = Condition::all()
        .add(weight_item::Column::BusinessId.eq(business_id))
        .add(weight_item::Column::ExpirationDate.eq(today));
    let weight_items = WeightItem::find()
        .find_with_related(ParentWeightItem)
        .filter(condition)
        .all(&database)
        .await
        .unwrap();

    for (weight_item, vec_parent) in weight_items{
        let parent = vec_parent.first().unwrap();
        all_expired.weight_items.push(ExpiredWeightItems{
            id: weight_item.id,
            title: parent.title.clone(),
            kg_weight: weight_item.kg_weight,
            main_image: parent.main_image.clone()
        })
    }


    let condition = Condition::all()
        .add(no_code_product::Column::BusinessId.eq(business_id))
        .add(no_code_product::Column::ExpirationDate.eq(today));
    let no_code_products = NoCodeProduct::find()
        .find_with_related(ParentNoCodeProduct)
        .filter(condition)
        .all(&database)
        .await
        .unwrap();

    for (no_code_product, vec_parent) in no_code_products{
        let parent = vec_parent.first().unwrap();
        all_expired.no_code_products.push(ExpiredNoCodeProducts{
            id: no_code_product.id,
            title: parent.title.clone(),
            quantity: no_code_product.quantity,
            main_image: parent.main_image.clone(),
        })
    }
    println!("{:?}", all_expired);
    (StatusCode::OK, Json(all_expired)).into_response()
}
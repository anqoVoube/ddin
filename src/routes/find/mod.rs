use serde_repr::{Deserialize_repr, Serialize_repr};

pub mod sell;
pub mod purchase;
pub mod router;
mod all;

use std::string::ToString;
use axum::{Extension, Json};
use axum::extract::Query;
use axum::response::{Response, IntoResponse};
use chrono::NaiveDate;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use http::StatusCode;
use sea_orm::sea_query::{Expr, Func};
use crate::core::auth::middleware::Auth;
use crate::database::prelude::{NoCodeProduct, ParentNoCodeProduct, Product};
use crate::database::prelude::ParentProduct;
use crate::database::{no_code_product, parent_no_code_product, parent_product, parent_weight_item, product, weight_item};
use crate::database::prelude::WeightItem;
use sea_orm::entity::*;
use sea_orm::query::*;
use crate::database::prelude::ParentWeightItem;
use crate::routes::utils::condition::starts_with;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum Types{
    Product = 1,
    WeightItem = 2,
    NoCodeProduct = 3
}
#[derive(Deserialize, Serialize)]
pub struct Search {
    search: String,
    r#type: Types,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProductSchema{
    id: i32,
    title: String,
    price: i32,
    main_image: Option<String>,
    max_quantity: i32,
    expiration_date: Option<NaiveDate>
}


#[derive(Serialize,  Deserialize, Debug)]
pub struct ProductsSchema{
    products: Vec<ProductSchema>
}

#[derive(Serialize,  Deserialize, Debug)]
pub struct WeightItemSchema{
    id: i32,
    title: String,
    price: i32,
    main_image: Option<String>,
    max_kg_weight: f64,
    expiration_date: Option<NaiveDate>,
}

#[derive(Serialize,  Deserialize, Debug)]

pub struct WeightItemsSchema{
    weight_items: Vec<WeightItemSchema>
}

#[derive(Serialize,  Deserialize, Debug)]

pub struct NoCodeProductSchema{
    id: i32,
    title: String,
    price: i32,
    main_image: Option<String>,
    max_quantity: i32,
    expiration_date: Option<NaiveDate>
}

#[derive(Serialize,  Deserialize, Debug)]
pub struct NoCodeProductsSchema{
    result: Vec<NoCodeProductSchema>
}

pub async fn find_product(search: String, business_id: i32, database: &DatabaseConnection) -> ProductsSchema{
    let products = Product::find()
        .find_with_related(ParentProduct)

        .filter(
            Condition::all()
                .add(product::Column::BusinessId.eq(business_id))
                .add(starts_with(&search, parent_product::Column::Title, false))
        )

        .all(database)

        .await.unwrap();

    let mut response_body = ProductsSchema{
        products: vec![]
    };

    for (product, vec_parent_product) in products {
        let parent_product = vec_parent_product.first().unwrap();
        let product = ProductSchema {
            id: product.id,
            title: parent_product.title.clone(),
            price: product.price,
            max_quantity: product.quantity,
            expiration_date: product.expiration_date,
            main_image: parent_product.main_image.clone()
        };

        response_body.products.push(product);
    }
    println!("{:?}", response_body);
    response_body
}

pub async fn find_weight_item(
    search: String,
    business_id: i32,
    database: &DatabaseConnection
) -> WeightItemsSchema{
    let like = format!("{}%", search.to_lowercase());
    let weight_items = WeightItem::find()
        .find_with_related(ParentWeightItem)

        .filter(

            Condition::all()
                .add(weight_item::Column::BusinessId.eq(business_id))
                .add(
                    Condition::any()
                        // .add(Expr::expr(Func::lower(Expr::col(parent_weight_item::Column::Title))).like(&like))
                        .add(starts_with(&search, parent_weight_item::Column::Title, false))
                )
        )
        .all(database)

        .await.unwrap();
    let mut response_body = WeightItemsSchema{
        weight_items: vec![]
    };

    for (weight_item, vec_parent_weight_item) in weight_items {
        let parent_weight_item = vec_parent_weight_item.first().unwrap();
        let weight_item_body = WeightItemSchema {
            id: weight_item.id,
            title: parent_weight_item.title.clone(),
            price: weight_item.price,
            max_kg_weight: weight_item.kg_weight,
            expiration_date: weight_item.expiration_date,
            main_image: parent_weight_item.main_image.clone()
        };

        response_body.weight_items.push(weight_item_body);
    }
    println!("{:?}", response_body);
    response_body
}


pub async fn find_no_code_product(
    search: String,
    business_id: i32,
    database: &DatabaseConnection
) -> NoCodeProductsSchema{
    let weight_items = NoCodeProduct::find()
        .find_with_related(ParentNoCodeProduct)
        .filter(

            Condition::all()
                .add(no_code_product::Column::BusinessId.eq(business_id))
                .add(
                    Condition::any()
                        // .add(Expr::expr(Func::lower(Expr::col(parent_weight_item::Column::Title))).like(&like))
                        .add(starts_with(&search, parent_no_code_product::Column::Title, false))
                )
        )
        .all(database)

        .await.unwrap();
    let mut response_body = NoCodeProductsSchema{
        result: vec![]
    };

    for (product, vec_parent) in weight_items {
        let parent = vec_parent.first().unwrap();
        let no_code_product = NoCodeProductSchema {
            id: product.id,
            title: parent.title.clone(),
            price: product.price,
            max_quantity: product.quantity,
            expiration_date: product.expiration_date,
            main_image: parent.main_image.clone()
        };

        response_body.result.push(no_code_product);
    }
    println!("{:?}", response_body);
    response_body
}

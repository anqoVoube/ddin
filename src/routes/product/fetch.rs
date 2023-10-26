use axum::{debug_handler, Extension, Json};
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use chrono::NaiveDate;
use http::StatusCode;


use sea_orm::{Condition, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use crate::database::product::Entity as Product;

use sea_orm::ColumnTrait;
use crate::database::product;
use crate::database::parent_product;

use crate::database::parent_product::Entity as ParentProduct;
use crate::routes::utils::{default_ok, not_found};


#[derive(Deserialize, Serialize, Debug)]
pub struct Body {
    code: String
}


#[derive(Serialize, Debug)]
pub struct ProductSchema {
    id: i32,
    title: String,
    price: i32,
    main_image: Option<String>,
    max_quantity: i32,
    expiration_date: NaiveDate
}


#[derive(Serialize, Debug)]
pub struct ProductsSchema{
    products: Vec<ProductSchema>
}



#[debug_handler] 
pub async fn fetch_products(
    Extension(database): Extension<DatabaseConnection>, Path(code): Path<String>
) -> Response {
    println!("{}", code);
    let products = Product::find()
        .find_with_related(ParentProduct)

        .filter(
            Condition::all()
                .add(product::Column::BusinessId.eq(1))
                .add(parent_product::Column::Code.eq(code))
        )

        .all(&database)

        .await.unwrap();

    let mut response_body = ProductsSchema{
        products: vec![]
    };

    if products.len() == 0{
        return not_found();
    }

    for (product, vec_parent_product) in products{
        let parent_product = vec_parent_product.first().unwrap();
        let product_body = ProductSchema{
            id: product.id,
            title: parent_product.title.clone(),
            price: product.price,
            max_quantity: product.quantity,
            expiration_date: product.expiration_date,
            main_image: parent_product.main_image.clone()
        };

        response_body.products.push(product_body);

    }
    println!("{:#?}", response_body);
    (
        StatusCode::OK,
        Json(response_body)
    ).into_response()
}

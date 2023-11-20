use axum::{debug_handler, Extension, Json};
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use chrono::NaiveDate;
use http::{header, StatusCode};


use sea_orm::{Condition, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use crate::database::product::Entity as Product;

use sea_orm::ColumnTrait;
use crate::core::auth::middleware::Auth;
use crate::database::product;
use crate::database::parent_product;

use crate::database::parent_product::Entity as ParentProduct;
use crate::routes::utils::not_found;
use serde_json::json;


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
    expiration_date: Option<NaiveDate>
}


#[derive(Serialize, Debug)]
pub struct ProductsSchema{
    products: Vec<ProductSchema>
}


#[debug_handler] 
pub async fn fetch_products(
    Extension(auth): Extension<Auth>,
    Extension(database): Extension<DatabaseConnection>,
    Path(code): Path<String>
) -> Result<Response, Response> {
    let api_key = "12345";  // Replace with your actual API key
    let centrifuge_url = "http://127.0.0.1:8000/api";  // Replace with your Centrifugo server URL

    // Headers
    let mut headers = header::HeaderMap::new();
    headers.insert("Authorization", header::HeaderValue::from_str(&format!("apikey {}", api_key)).unwrap());
    headers.insert("Content-Type", header::HeaderValue::from_static("application/json"));

    // Client

    // Example payload



    let products = Product::find()
        .find_with_related(ParentProduct)

        .filter(
            Condition::all()
                .add(product::Column::BusinessId.eq(auth.business_id))
                .add(parent_product::Column::Code.eq(code))
        )

        .all(&database)

        .await.unwrap();

    let mut response_body = ProductsSchema{
        products: vec![]
    };

    if products.len() == 0{
        return Err(not_found());
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

    // {"id": 1, "image_url": "http://127.0.0.1:3000/media/milk.jpg", "title": "Milk", "price": 12000, "expiration_date": "2023-05-13", "max_quantity": 2}

    let payload = json!({
        "method": "publish",
        "params": {
            "channel": "channel",
            "data": {
                "id": response_body.products[0].id,
                "main_image": response_body.products[0].main_image,
                "title": response_body.products[0].title,
                "price": response_body.products[0].price,
                "expiration_date": response_body.products[0].expiration_date,
                "max_quantity": response_body.products[0].max_quantity
            }
        }
    });
    // Send request
    let response = reqwest::Client::new()
    .post(centrifuge_url)
    .headers(headers)
    .json(&payload)
    .send().await.unwrap();

    println!("TELL");
    println!("{:#?}", response_body);
    Ok(
        (
        StatusCode::OK,
        Json(response_body)
        ).into_response()
    )
}

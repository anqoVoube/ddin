use axum::{debug_handler, Extension, Json};
use axum::extract::{Path, Query};
use axum::response::{IntoResponse, Response};
use chrono::NaiveDate;
use http::{header, StatusCode};


use sea_orm::{Condition, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use crate::database::product::Entity as Product;

use sea_orm::ColumnTrait;
use crate::core::auth::middleware::{Auth, CustomHeader};
use crate::database::product;
use crate::database::parent_product;

use crate::database::parent_product::Entity as ParentProduct;
use crate::routes::utils::not_found;
use serde_json::json;
use crate::routes::find::code::ResponseBody;
use crate::routes::parent_product::fetch::{get_object, ParentProductSchema};

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


#[derive(Serialize, Deserialize, Debug)]
pub struct QueryBody{
    fast: Option<bool>
}

#[debug_handler] 
pub async fn fetch_products(
    Extension(auth): Extension<Auth>,
    Extension(custom_headers): Extension<CustomHeader>,
    Extension(database): Extension<DatabaseConnection>,
    Path(code): Path<String>,
    Query(QueryBody{fast}): Query<QueryBody>

) -> Result<Response, Response> {
    let api_key = "api-key";  // Replace with your actual API key
    let centrifuge_url = "https://centrifuge.ddin.uz/api";  // Replace with your Centrifugo server URL

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
                .add(product::Column::BusinessId.eq(custom_headers.business_id))
                .add(parent_product::Column::Code.eq(code.clone()))
        )

        .all(&database)

        .await.unwrap();

    let mut products_view: Vec<ProductSchema> = vec![];

    if products.len() == 0{
        println!("0 products find, trying to find parent product {}", code);
        return match get_object(&database, code, custom_headers.business_id).await {
            Ok(parent_product) => {
                let parent_product_schema: ParentProductSchema = parent_product.into();
                return Ok(
                    (
                        StatusCode::BAD_REQUEST,
                        Json(parent_product_schema)
                    ).into_response()
                );
            },
            Err(error_status_code) => Err(not_found())
        };
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

        products_view.push(product_body);

    }

    // {"id": 1, "image_url": "http://127.0.0.1:3000/media/milk.jpg", "title": "Milk", "price": 12000, "expiration_date": "2023-05-13", "max_quantity": 2}
    if let Some(_) = fast{
        let payload = json!({
            "method": "publish",
            "params": {
                "channel": "channel",
                "data": products_view
            }
        });
        // Send request
        let response = reqwest::Client::new()
            .post(centrifuge_url)
            .headers(headers)
            .json(&payload)
            .send().await.unwrap();
    }
    println!("{:#?}", products_view);
    Ok(
        (
        StatusCode::OK,
        Json(ProductsSchema{
            products: products_view
        })
        ).into_response()
    )
}

use axum::{Extension, Json};
use axum::extract::Query;
use crate::core::auth::middleware::Auth;
use axum::response::{Response, IntoResponse};
use http::StatusCode;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use crate::routes::find::{find_product, find_no_code_product, find_weight_item, NoCodeProductSchema, ProductSchema, WeightItemSchema};


#[derive(Deserialize, Serialize)]
pub struct Search {
    search: String,
}


#[derive(Deserialize, Serialize)]
pub struct SearchResult {
    products: Vec<ProductSchema>,
    weight_items: Vec<WeightItemSchema>,
    no_code_products: Vec<NoCodeProductSchema>
}



pub async fn search(
    Extension(auth): Extension<Auth>,
    Extension(database): Extension<DatabaseConnection>,
    Query(Search {search}): Query<Search>
) -> Response {
    let products = find_product(search.clone(), auth.business_id, &database).await;
    let weight_items = find_weight_item(search.clone(), auth.business_id, &database).await;
    let no_code_products = find_no_code_product(search, auth.business_id, &database).await;
    (
        StatusCode::OK,
        Json(SearchResult{
                products: products.products,
                weight_items: weight_items.weight_items,
                no_code_products: no_code_products.result
        }),
    ).into_response()
}
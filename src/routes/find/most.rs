use sea_orm::Condition;
use crate::database::prelude::{NoCodeProduct, NoCodeProductSearch, ParentNoCodeProduct, ParentWeightItem, WeightItem, WeightItemSearch};
use axum::{Extension, Json};
use axum::extract::Query;
use crate::core::auth::middleware::{Auth, CustomHeader};
use axum::response::{Response, IntoResponse};
use http::StatusCode;
use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use crate::database::{no_code_product, no_code_product_search, parent_no_code_product, parent_weight_item, weight_item, weight_item_search};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;

const DEFAULT_PAGE_SIZE: i32 = 10;
const DEFAULT_PAGE: i32 = 1;


#[derive(Deserialize, Serialize)]
pub struct Search {
    page_size: Option<i32>,
    page: Option<i32>,
    item_type: u8
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PopularProductSearch {
    id: i32,
    title: String,
    main_image: Option<String>,
    max_quantity: i32,
    hits: i32,
    price: f64
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WeightItemSearchResult {
    id: i32,
    title: String,
    main_image: Option<String>,
    max_kg_weight: f64,
    hits: i32,
    price: f64,
}


pub async fn most_searched(
    Extension(auth): Extension<Auth>,
    Extension(headers): Extension<CustomHeader>,
    Extension(database): Extension<DatabaseConnection>,
    Query(Search {page_size, page, item_type}): Query<Search>
) -> Response {
    match item_type {
        1 => {
            let weight_item_search: Vec<(weight_item_search::Model, Vec<parent_weight_item::Model>)> = WeightItemSearch::find()
                .find_with_related(ParentWeightItem)
                .order_by_desc(weight_item_search::Column::Hits)
                .offset(((page.unwrap_or(DEFAULT_PAGE) - 1) * page_size.unwrap_or(DEFAULT_PAGE_SIZE)) as u64)
                .limit(page_size.unwrap_or(DEFAULT_PAGE_SIZE) as u64)

                .all(&database)
                .await.unwrap();
            let mut result = vec![];
            for (weight_item_search, vec_weight_item) in weight_item_search {
                let parent_weight_item = vec_weight_item.first().unwrap();
                let condition = Condition::all()
                    .add(weight_item::Column::ParentId.eq(parent_weight_item.id))
                    .add(weight_item::Column::BusinessId.eq(headers.business_id));
                match WeightItem::find().filter(condition).one(&database).await{
                    Ok(Some(weight_item_instance)) => {
                        result.push(WeightItemSearchResult{
                            id: weight_item_instance.id,
                            title: parent_weight_item.title.clone(),
                            main_image: parent_weight_item.main_image.clone(),
                            max_kg_weight: weight_item_instance.kg_weight,
                            hits: weight_item_search.hits,
                            price: weight_item_instance.price
                        });
                    },
                    _ => {
                        println!("xz")
                    }
                }

            }
            println!("{:?}", result);
            (
                StatusCode::OK,
                Json(result)
            ).into_response()
        },
        3 => {
            let no_code_product_search: Vec<(no_code_product_search::Model, Vec<parent_no_code_product::Model>)> = NoCodeProductSearch::find()
                .find_with_related(ParentNoCodeProduct)
                .order_by_desc(no_code_product_search::Column::Hits)
                .offset(((page.unwrap_or(DEFAULT_PAGE) - 1) * page_size.unwrap_or(DEFAULT_PAGE_SIZE)) as u64)
                .limit(page_size.unwrap_or(DEFAULT_PAGE_SIZE) as u64)

                .all(&database)
                .await.unwrap();

            let mut result = vec![];
            for (no_code_product_search, vec_no_code_product) in no_code_product_search{
                let parent_no_code_product = vec_no_code_product.first().unwrap();
                let condition = Condition::all()
                    .add(no_code_product::Column::ParentId.eq(parent_no_code_product.id))
                    .add(no_code_product::Column::BusinessId.eq(headers.business_id));
                match NoCodeProduct::find().filter(condition).one(&database).await {
                    Ok(Some(no_code_product_instance)) => {
                        result.push(PopularProductSearch {
                            id: no_code_product_instance.id,
                            title: parent_no_code_product.title.clone(),
                            main_image: parent_no_code_product.main_image.clone(),
                            max_quantity: no_code_product_instance.quantity,
                            hits: no_code_product_search.hits,
                            price: no_code_product_instance.price
                        });
                    },
                    _ => {
                        println!("xz")
                    }
                }


            }
            println!("{:?}", result);
            (
                StatusCode::OK,
                Json(result)
            ).into_response()

        },

        _ => return (StatusCode::BAD_REQUEST, Json("Invalid item type")).into_response()
    }
}


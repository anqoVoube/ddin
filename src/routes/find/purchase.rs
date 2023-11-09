use axum::{Extension, Json};
use axum::extract::Query;
use axum::response::{Response, IntoResponse};
use chrono::NaiveDate;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_repr::{Serialize_repr, Deserialize_repr};
use http::StatusCode;
use crate::core::auth::middleware::Auth;
use crate::database::prelude::Product;
use crate::database::prelude::ParentProduct;
use crate::database::{parent_no_code_product, parent_product, parent_weight_item, product, weight_item};
use crate::database::prelude::WeightItem;
use sea_orm::entity::*;
use sea_orm::query::*;
use crate::database::prelude::ParentNoCodeProduct;
use crate::database::prelude::ParentWeightItem;
use crate::database::weight_item::Column::ParentId;
use crate::routes::AppConnections;
use crate::routes::find::{Search, Types};
use crate::routes::utils::condition::starts_with;


#[derive(Serialize, Debug)]
pub struct ParentProductSchema{
    id: i32,
    title: String,
    main_image: Option<String>
}

#[derive(Serialize, Debug)]
pub struct ParentProductsSchema{
    result: Vec<ParentProductSchema>
}

#[derive(Serialize, Debug)]
pub struct ParentWeightItemSchema{
    id: i32,
    title: String,
    main_image: Option<String>
}

#[derive(Serialize, Debug)]
pub struct ParentWeightItemsSchema{
    result: Vec<ParentWeightItemSchema>
}

#[derive(Serialize, Debug)]
pub struct ParentNoCodeProductSchema{
    id: i32,
    title: String,
    main_image: Option<String>
}

#[derive(Serialize, Debug)]
pub struct ParentNoCodeProductsSchema{
    result: Vec<ParentNoCodeProductSchema>
}


pub async fn search(
    Extension(auth): Extension<Auth>,
    Extension(connections): Extension<AppConnections>,
    Query(query): Query<Search>
) -> Response{
    println!("{} {:?}", query.search, query.r#type);
    match query.r#type{
        Types::Product => {
            let data = find_product(query.search, auth.business_id, &connections.database).await;
            ().into_response()
        },
        Types::WeightItem => {
            let data = find_parent_weight_item(
                query.search,
                auth.business_id,
                &connections.database
            ).await;
            (
                StatusCode::OK,
                Json(data)
            ).into_response()
        },
        Types::NoCodeProduct => {
            let data = find_no_code_product(
                query.search,
                auth.business_id,
                &connections.database
            ).await;
            (
                StatusCode::OK,
                Json(data)
            ).into_response()
        }
    }
}


pub async fn find_product(search: String, business_id: i32, database: &DatabaseConnection) -> i32{
    let products = Product::find()
        .find_with_related(ParentProduct)

        .filter(
            Condition::all()
                .add(product::Column::BusinessId.eq(business_id))
                .add(parent_product::Column::Title.starts_with(search))
        )

        .all(database)

        .await.unwrap();
    1
}

pub async fn find_parent_weight_item(
    search: String,
    business_id: i32,
    database: &DatabaseConnection
) -> ParentWeightItemsSchema{
    let like = format!("{}%", search.to_lowercase());
    let parent_weight_items = ParentWeightItem::find()
        .filter(
            Condition::all()
                .add(
                    Condition::all()
                        .add(starts_with(&search, parent_weight_item::Column::Title, false))
                )
                .add(
                    Condition::any()
                        .add(parent_weight_item::Column::BusinessId.eq(business_id))
                        .add(parent_weight_item::Column::BusinessId.is_null())
                )
        )
        .all(database)

        .await.unwrap();

    let mut response_body = ParentWeightItemsSchema{
        result: vec![]
    };

    for instance in parent_weight_items {
        let weight_item = ParentWeightItemSchema {
            id: instance.id,
            title: instance.title.clone(),
            main_image: instance.main_image.clone()
        };

        response_body.result.push(weight_item);
    }
    println!("{:?}", response_body);
    response_body
}


pub async fn find_no_code_product(
    search: String,
    business_id: i32,
    database: &DatabaseConnection
) -> ParentNoCodeProductsSchema{
    let like = format!("{}%", search.to_lowercase());
    let parent_no_code_products = ParentNoCodeProduct::find()
        .filter(
            Condition::all()
                .add(
                    Condition::all()
                        .add(starts_with(&search, parent_no_code_product::Column::Title, false))

                )
                .add(
                    Condition::any()
                        .add(parent_no_code_product::Column::BusinessId.eq(business_id))
                        .add(parent_no_code_product::Column::BusinessId.is_null())

                )
        )
        .all(database)

        .await.unwrap();

    let mut response_body = ParentNoCodeProductsSchema{
        result: vec![]
    };

    for instance in parent_no_code_products {
        let product = ParentNoCodeProductSchema {
            id: instance.id,
            title: instance.title.clone(),
            main_image: instance.main_image.clone()
        };

        response_body.result.push(product);
    }
    println!("{:?}", response_body);
    response_body
}

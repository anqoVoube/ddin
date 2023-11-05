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
use crate::database::{parent_product, parent_weight_item, product, weight_item};
use crate::database::prelude::WeightItem;
use sea_orm::entity::*;
use sea_orm::query::*;
use crate::database::prelude::ParentWeightItem;
use crate::database::weight_item::Column::ParentWeightItemId;
use crate::routes::AppConnections;
use crate::routes::utils::condition::starts_with;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum Types{
    Product = 1,
    WeightItem = 2,
}
#[derive(Deserialize, Serialize)]
pub struct Search {
    search: String,
    r#type: Types,
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
            Condition::any()
                .add(
                    Condition::all()
                        .add(starts_with(&search, parent_weight_item::Column::Title, false))
                )
                .add(
                    Condition::all()
                        .add(parent_weight_item::Column::BusinessId.eq(business_id))
                )
        )
        .all(database)

        .await.unwrap();

    let mut response_body = ParentWeightItemsSchema{
        result: vec![]
    };

    for parent_weight_item in parent_weight_items {
        let weight_item_body = ParentWeightItemSchema {
            id: parent_weight_item.id,
            title: parent_weight_item.title.clone(),
            main_image: parent_weight_item.main_image.clone()
        };

        response_body.result.push(weight_item_body);
    }
    println!("{:?}", response_body);
    response_body
}



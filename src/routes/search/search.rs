use std::string::ToString;
use axum::{Extension, Json};
use axum::extract::Query;
use axum::response::{Response, IntoResponse};
use chrono::NaiveDate;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_repr::{Serialize_repr, Deserialize_repr};
use tower_http::classify::ServerErrorsFailureClass::StatusCode;
use crate::core::auth::middleware::Auth;
use crate::database::prelude::Product;
use crate::database::prelude::ParentProduct;
use crate::database::{parent_product, parent_weight_item, product, weight_item};
use crate::database::prelude::WeightItem;
use crate::database::weight_item::Relation::ParentWeightItem;
use crate::routes::AppConnections;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum Types{
    Product = 1,
    WeightItem = 2,
}
#[derive(Deserialize, Serialize)]
struct Search {
    search: String,
    r#type: Types,
}


#[derive(Serialize, Debug)]
pub struct WeightItemSchema{
    id: i32,
    title: String,
    price: i32,
    main_image: Option<String>,
    max_kg_weight: f32,
    expiration_date: Option<NaiveDate>
}

#[derive(Serialize, Debug)]
pub struct WeightItemsSchema{
    weight_items: Vec<WeightItemSchema>
}

pub async fn find_by_name(
    Extension(auth): Extension<Auth>,
    Extension(connections): Extension<AppConnections>,
    Query(query): Query<Search>
) -> Response{
        match query.r#type{
            Types::Product => {
                let data = find_product(query.search, auth.business_id, &connections.database);
                ().into_response()
            },
            Types::WeightItem => {
                let data = find_weight_item(
                    query.search,
                    auth.business_id,
                    &connections.database
                );
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

pub async fn find_weight_item(
    search: String,
    business_id: i32,
    database: &DatabaseConnection
) -> WeightItemsSchema{
    let weight_items = WeightItem::find()
        .find_with_related(ParentProduct)

        .filter(

            Condition::all()
                .add(weight_item::Column::BusinessId.eq(business_id))
                .add(
                    Condition::any()
                        .add(parent_weight_item::Column::Title.starts_with(search.clone()))
                        .add(parent_weight_item::Column::TitleUz.starts_with(search.clone()))
                        .add(parent_weight_item::Column::TitleRu.starts_with(search.clone()))
                )
        )
        .distinct()
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
            main_image: weight_item.main_image.clone()
        };

        response_body.weight_items.push(weight_item_body);
    }

    response_body
}



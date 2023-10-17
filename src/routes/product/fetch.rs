use axum::{debug_handler, Extension, Json};
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use log::warn;
use sea_orm::{Condition, DatabaseConnection, EntityOrSelect, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait};
use serde::{Deserialize, Serialize};
use crate::database::product::Entity as Product;
use crate::database::product::Model as ProductModel;
use sea_orm::ColumnTrait;
use crate::database::product;
use crate::database::parent_product;
use crate::database::product::Column::ParentProductId;
use crate::database::parent_product::Entity as ParentProduct;

use crate::routes::parent_product::fetch::get_object as get_parent_product_by_code;


#[derive(Deserialize, Serialize, Debug)]
pub struct Body {
    code: String
}


#[derive(Serialize, Debug)]
pub struct ProductSchema {
    id: i64,
    price: i64,
    main_image: Option<String>
}


#[debug_handler] 
pub async fn fetch_products(
    Extension(database): Extension<DatabaseConnection>, Path(code): Path<String>
) -> Response {

    let products = Product::find()

        .find_with_related(ParentProduct)

        .filter(
            Condition::all()
                .add(product::Column::BusinessId.eq(2))
                .add(parent_product::Column::Code.eq(code))
        )

        .all(&database)

        .await.unwrap();
    for (product, vec_parent_product) in products{
        let parent_product = vec_parent_product.first().unwrap();
        println!("{:?}", parent_product.main_image);
    }
    ().into_response()
}

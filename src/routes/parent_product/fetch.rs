use axum::{Extension, Json};
use axum::extract::Path;
use http::StatusCode;
use sea_orm::{Condition, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;
use crate::database::parent_product::Entity as ParentProduct;
use crate::database::parent_product::Model as ParentProductModel;
use log::{warn};
use crate::database::parent_product;
use sea_orm::ColumnTrait;

#[derive(Serialize, Debug)]
pub struct ParentProductSchema {
    code: String,
    title: String,
    description: String,
}

impl From<ParentProductModel> for ParentProductSchema {
    fn from(parent_product: ParentProductModel) -> Self {
        ParentProductSchema {
            code: parent_product.code,
            title: parent_product.title,
            description: parent_product.description
        }
    }
}

pub async fn get_object_by_code(
    Extension(database): Extension<DatabaseConnection>, Path(code): Path<String>
) -> Result<Json<ParentProductSchema>, StatusCode> {

    let mut condition = Condition::all();
    condition = condition.add(parent_product::Column::Code.eq(code.clone()));
    let parent_product = ParentProduct::find().filter(condition).one(&database).await
        .map_err(|_error| {warn!("Couldn't fetch parent_product with code: {}", &code); StatusCode::INTERNAL_SERVER_ERROR})?;

    if let Some(value) = parent_product{
        Ok(Json(value.into()))
    }
    else{
        warn!("Couldn't fetch parent_product with code: {}", &code);
        Err(StatusCode::NOT_FOUND)
    }
}
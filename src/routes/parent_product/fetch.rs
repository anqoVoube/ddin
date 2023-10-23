use axum::{debug_handler, Extension, Json};
use axum::extract::Path;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;

use http::StatusCode;
use sea_orm::{Condition, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;
use crate::database::parent_product::Entity as ParentProduct;
use crate::database::parent_product::Model as ParentProductModel;
use log::{error, info, warn};
use crate::database::parent_product;
use sea_orm::ColumnTrait;

#[derive(Serialize, Debug)]
pub struct ParentProductSchema {
    id: i32,
    code: String,
    title: String,
    main_image: Option<String>
}

impl From<ParentProductModel> for ParentProductSchema {
    fn from(parent_product: ParentProductModel) -> Self {
        ParentProductSchema {
            id: parent_product.id,
            code: parent_product.code,
            title: parent_product.title,
            main_image: parent_product.main_image
        }
    }
}

#[debug_handler]
pub async fn get_object_by_code(
    Extension(database): Extension<DatabaseConnection>, Path(code): Path<String>, jar: CookieJar
) -> Result<Json<ParentProductSchema>, StatusCode> {
    println!("REQUEST!");
    match get_object(&database, code).await{
        Ok(instance) => Ok(Json(instance.into())),
        Err(error_status_code) => Err(error_status_code)
    }
}

pub async fn get_object(database: &DatabaseConnection, code: String) -> Result<ParentProductModel, StatusCode> {
    let mut condition = Condition::all();
    condition = condition.add(parent_product::Column::Code.eq(code.clone()));
    let parent_product = ParentProduct::find().filter(condition).one(database).await
        .map_err(|_error| {warn!("Couldn't fetch parent_product with code: {}", &code); StatusCode::INTERNAL_SERVER_ERROR})?;

    if let Some(value) = parent_product{
        Ok(value)
    }
    else{
        warn!("Couldn't fetch parent_product with code: {}", &code);
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn get_object_by_id(database: &DatabaseConnection, id: i32) -> Result<ParentProductModel, StatusCode> {
    let parent_product = ParentProduct::find_by_id(id).one(database).await
        .map_err(|_error| {error!("Couldn't fetch parent_product with id: {}", id); StatusCode::INTERNAL_SERVER_ERROR})?;

    if let Some(value) = parent_product{
        Ok(value)
    }
    else{
        info!("Not found parent_product with id: {}", &id);
        Err(StatusCode::NOT_FOUND)
    }
}
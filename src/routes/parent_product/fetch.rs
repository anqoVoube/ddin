use std::sync::Arc;
use axum::{debug_handler, Extension, Json};
use axum::extract::Path;
use axum::response::IntoResponse;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;

use http::StatusCode;
use sea_orm::{ActiveModelTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;
use crate::database::parent_product::{Entity as ParentProduct, Model};
use crate::database::parent_product::Model as ParentProductModel;
use log::{error, info, warn};
use sea_orm::ActiveValue::Set;
use crate::database::parent_product;
use sea_orm::ColumnTrait;
use tokio::sync::Mutex;
use crate::core::auth::middleware::{Auth, CustomHeader};
use crate::routes::find::code::{Data, ResponseBody};
use crate::routes::SqliteDBConnection;
use crate::routes::utils::{default_created, internal_server_error};

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

#[derive(Debug, Clone)]
pub struct SqliteData{
    name: String
}

#[debug_handler]
pub async fn get_object_by_code(
    Extension(database): Extension<DatabaseConnection>,
    Extension(SqliteDBConnection { sqlite }): Extension<SqliteDBConnection>,
    Extension(Auth{user_id}): Extension<Auth>,
    Extension(CustomHeader{business_id}): Extension<CustomHeader>,
    Path(code): Path<String>
) -> Result<Json<ParentProductSchema>, StatusCode> {
    println!("REQUEST!");
    match get_object(&database, Some(sqlite), code, business_id).await{
        Ok(instance) => Ok(Json(instance.into())),
        Err(error_status_code) => Err(error_status_code)
    }
}

pub async fn get_object(database: &DatabaseConnection, sqlite: Option<Arc<Mutex<rusqlite::Connection>>>, code: String, business_id: i32) -> Result<ParentProductModel, StatusCode> {
    let condition = Condition::all()
        .add(
            Condition::all()
                .add(parent_product::Column::Code.eq(code.clone()))
        )
        .add(
            Condition::any()
                .add(parent_product::Column::BusinessId.eq(business_id))
                .add(parent_product::Column::BusinessId.is_null())

        );

    let parent_product = ParentProduct::find().filter(condition).one(database).await
        .map_err(|_error| {warn!("Couldn't fetch parent_product with code: {}", &code); StatusCode::INTERNAL_SERVER_ERROR})?;

    if let Some(value) = parent_product{

        Ok(value)
    }
    else{
        if let Some(sqlite) = sqlite{
            let sqlite_products: Vec<String> = match sqlite.lock().await.prepare("SELECT name FROM Barcodes WHERE barcode=:code;") {
                Ok(mut value) => match value.query_map(&[(":code", &code.clone())], |row| {
                    Ok(SqliteData {
                        name: row.get(0).unwrap_or("ou".to_owned()),
                    })
                }) {
                    Ok(titles) => {
                        let mut title_names: Vec<String> = vec!();
                        for title in titles {
                            title_names.push(title.unwrap().name);
                        }
                        title_names
                    },
                    Err(err) => {
                        println!("{}", err);
                        return Err(
                            StatusCode::INTERNAL_SERVER_ERROR,
                        );
                    }
                },
                Err(err) => {
                    println!("{}", err);
                    return Err(
                        StatusCode::INTERNAL_SERVER_ERROR
                    )
                }
            };
            if sqlite_products.len() == 0{
                warn!("Couldn't fetch parent_product with code: {}", &code);
                return Err(StatusCode::NOT_FOUND);
            }
            let new_parent_product = parent_product::ActiveModel {
                code: Set(code),
                title: Set(sqlite_products[0].clone()),
                description: Set("Interesting".to_owned()),
                ..Default::default()
            };
            match new_parent_product.save(database).await {
                Ok(instance) => {
                    return Ok(Model::try_from(instance).unwrap());
                },
                Err(error) => {
                    error!("Unable to create {:?}. Original error was {}", 1, error);
                    return Err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                    );
                }
            }
        }
        else{
            warn!("Not found parent_product with code: {}", &code);
            return Err(StatusCode::NOT_FOUND);
        }

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
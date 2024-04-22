use axum::{Extension, Json, debug_handler};
use axum::response::Response;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use serde::{Deserialize, Serialize};
use sea_orm::ActiveValue::Set;
use crate::database::parent_product;


use log::{error, info};
use crate::core::auth::middleware::{Auth, CustomHeader};
use crate::routes::utils::{default_created, internal_server_error};


#[derive(Clone, Serialize, Deserialize)]
pub struct Body {
    code: String,
    title: String,
    description: String,
}


#[debug_handler]
pub async fn create(
    Extension(database): Extension<DatabaseConnection>,
    Extension(auth): Extension<Auth>,
    Extension(headers): Extension<CustomHeader>,
    Json(Body { code, title, description }): Json<Body>
) -> Response {
    let new_parent_product = parent_product::ActiveModel {
        code: Set(code),
        title: Set(title),
        description: Set(description),
        business_id: Set(Some(headers.business_id)),
        ..Default::default()
    };

    match new_parent_product.save(&database).await {
        Ok(instance) => {
            info!("{:?}", instance);
            default_created()
        },
        Err(error) => {
            error!("Unable to create {:?}. Original error was {}", 1, error);
            internal_server_error()
        }
    }
}

use sea_orm::{Condition, QueryFilter};
use axum::{ debug_handler, Extension};

use axum_extra::extract::Multipart;

use std::{str, fs::File, io::Write, path::Path, fs};
use axum::response::Response;
use log::error;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use sea_orm::ActiveValue::Set;
use crate::database::{parent_no_code_product, parent_weight_item};
use crate::database::prelude::ParentNoCodeProduct;
use crate::routes::utils::{default_created, internal_server_error, hash_helper::generate_uuid4, bad_request};
use sea_orm::ColumnTrait;
use crate::core::auth::middleware::{Auth, CustomHeader};

const DEFAULT_EXPIRATION_IN_DAYS: i32 = 365;

pub struct RequestBody{
    main_image: Option<String>,
    title: Option<String>,
    description: Option<String>,
    images: Vec<String>,
}

#[debug_handler]
pub async fn upload(
    Extension(Auth{user_id}): Extension<Auth>,
    Extension(CustomHeader{business_id}): Extension<CustomHeader>,
    Extension(database): Extension<DatabaseConnection>,
    mut multipart: Multipart
) -> Response {
    let mut request_body = RequestBody{
        main_image: None,
        title: None,
        description: None,
        images: vec!(),
    };
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();

        if name.ends_with("images") {
            let file_data: Vec<u8> = field.bytes().await.unwrap().to_vec();
            let file_name = format!("{}.jpg", generate_uuid4());


        } else if name.ends_with("main_image") {
            let file_data: Vec<u8> = field.bytes().await.unwrap().to_vec();
            let file_name = format!("{}.jpg", generate_uuid4());
            request_body.main_image = Some(file_name.clone());
            let filepath = Path::new("/var/www/html/media/").join(file_name);
            let mut file = File::create(filepath).unwrap();
            file.write_all(&file_data).unwrap();

        } else {
            let bytes = field.bytes().await.unwrap();
            let text_data: String = str::from_utf8(&bytes).unwrap().to_string();
            // checking for title existence
            if name.ends_with("title"){
                match ParentNoCodeProduct::find()
                    .filter(
                        Condition::all()
                            .add(
                                Condition::all()
                                    .add(parent_no_code_product::Column::Title.eq(text_data.clone()))
                            )
                            .add(
                                Condition::any()
                                    .add(parent_no_code_product::Column::BusinessId.eq(business_id))
                                    .add(parent_no_code_product::Column::BusinessId.is_null())
                            )
                    )
                    .one(&database).await{
                    Ok(Some(_)) => {
                        return bad_request("Title already exists");
                    },
                    Ok(None) => {
                        request_body.title = Some(text_data);
                    },
                    Err(err) => {
                        error!("Error: {:?}", err);
                        return internal_server_error();
                    }
                }
            } else if name.ends_with("description"){
                request_body.description = Some(text_data);
            } else {
                return bad_request("Invalid field name");
            }
        }
    }

    let new_parent = parent_no_code_product::ActiveModel{
        title: Set(request_body.title.clone().unwrap()),
        description: Set(request_body.description.clone().unwrap_or("hello".to_owned())),
        main_image: Set(request_body.main_image.clone()),
        images: Set(request_body.images.clone()),
        business_id: Set(Some(business_id)),

        ..Default::default()
    };

    match new_parent.save(&database).await{
        Ok(_) => default_created(),
        Err(err) => {
            error!("Error: {:?}", err);
            internal_server_error()
        }
    }
}

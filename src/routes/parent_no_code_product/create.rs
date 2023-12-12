use axum::{ debug_handler, Extension};

use axum_extra::extract::Multipart;

use std::{str, fs::File, io::Write, path::Path, fs};
use axum::response::Response;
use log::error;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use sea_orm::ActiveValue::Set;
use crate::database::parent_no_code_product;
use crate::database::prelude::ParentNoCodeProduct;
use crate::routes::utils::{default_created, internal_server_error, hash_helper::generate_uuid4, space_upload::upload_to_space, bad_request};

pub struct RequestBody{
    main_image: Option<String>,
    title: Option<String>,
    description: Option<String>,
    images: Vec<String>
}

#[debug_handler]
pub async fn upload(
    Extension(database): Extension<DatabaseConnection>,
    mut multipart: Multipart
) -> Response {
    let mut request_body = RequestBody{
        main_image: None,
        title: None,
        description: None,
        images: vec!()
    };
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();

        if name.ends_with("images") {
            let file_data: Vec<u8> = field.bytes().await.unwrap().to_vec();
            let unique_hash = generate_uuid4();
            match upload_to_space(file_data, unique_hash.clone()).await{
                Ok(_) => {
                    request_body.images.push(unique_hash);
                },
                Err(err) => {
                    error!("Error: {:?}", err);
                    return internal_server_error();
                }
            }

        } else if name.ends_with("title") {
            let bytes = field.bytes().await.unwrap();
            let text_data: String = str::from_utf8(&bytes).unwrap().to_string();
            // checking for title existence
            match ParentNoCodeProduct::find()
                .filter(parent_no_code_product::Column::Title.Eq(text_data.clone()))
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
        } else {
            let file_data: Vec<u8> = field.bytes().await.unwrap().to_vec();
            let unique_hash = generate_uuid4();
            match upload_to_space(file_data, unique_hash.clone()).await{
                Ok(_) => {
                    request_body.main_image = Some(unique_hash);
                },
                Err(err) => {
                    error!("Error: {:?}", err);
                    return internal_server_error();
                }
            }
        }
    }

    let new_parent = parent_no_code_product::ActiveModel{
        title: Set(request_body.title.clone().unwrap()),
        description: Set(request_body.description.clone().unwrap()),
        main_image: Set(request_body.main_image.clone()),
        images: Set(request_body.images.clone()),
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




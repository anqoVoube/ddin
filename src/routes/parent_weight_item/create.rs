use axum::{ debug_handler, Extension};

use axum_extra::extract::Multipart;

use std::{str, fs::File, io::Write, path::Path, fs};
use axum::response::Response;
use log::error;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use sea_orm::ActiveValue::Set;
use crate::database::parent_weight_item;
use crate::database::prelude::ParentWeightItem;
use crate::routes::utils::{default_created, internal_server_error, hash_helper::generate_uuid4, bad_request};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use crate::core::auth::middleware::{Auth, CustomHeader};

const DEFAULT_EXPIRATION_IN_DAYS: i32 = 365;

pub struct RequestBody{
    main_image: Option<String>,
    title: Option<String>,
    description: Option<String>,
    images: Vec<String>,
    expiration_in_days: Option<i32>,
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
        expiration_in_days: None
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
            let filepath = Path::new("media/images").join(file_name);
            let mut file = File::create(filepath).unwrap();
            file.write_all(&file_data).unwrap();


        } else {
            let bytes = field.bytes().await.unwrap();
            let text_data: String = str::from_utf8(&bytes).unwrap().to_string();
            // checking for title existence
            if name.ends_with("title"){
                match ParentWeightItem::find()
                    .filter(parent_weight_item::Column::Title.eq(text_data.clone()))
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
            } else if name.ends_with("expiration"){
                request_body.expiration_in_days = Some(text_data.parse::<i32>().unwrap());
            } else {
                return bad_request("Invalid field name");
            }
        }
    }

    let new_parent = parent_weight_item::ActiveModel{
        title: Set(request_body.title.clone().unwrap()),
        description: Set(request_body.description.clone().unwrap_or("hello".to_owned())),
        main_image: Set(request_body.main_image.clone()),
        images: Set(request_body.images.clone()),
        expiration_in_days: Set(request_body.expiration_in_days.unwrap_or(DEFAULT_EXPIRATION_IN_DAYS)),
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




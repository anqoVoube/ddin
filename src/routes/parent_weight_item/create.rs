use axum::{ debug_handler, Extension};

use axum_extra::extract::Multipart;

use std::{str, fs::File, io::Write, path::Path, fs};
use axum::response::Response;
use log::error;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use sea_orm::ActiveValue::Set;
use crate::database::parent_weight_item;
use crate::routes::utils::{default_created, internal_server_error};

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

        if name.starts_with("image") {
            let count = name.split("_").collect::<Vec<&str>>().first().unwrap().parse::<usize>().unwrap();
            // Generate a unique filename for the image
            // title won't be null as it will be send before the photo
            let filename = format!("{}", process_title(&request_body.title.clone().unwrap()));
            let filename_with_format = format!("additional-{}.jpg", filename);
            let dir_path = format!("media/images/");
            if let Ok(_) = fs::create_dir_all(&dir_path){
                println!("Created directory")
            }
            // Specify the directory where the file will be saved
            let filepath = Path::new(&dir_path).join(filename.clone());

            // Get the image data
            let file_data = field.bytes().await.unwrap();

            // Save the file
            let mut file = File::create(filepath).unwrap();
            file.write_all(&file_data).unwrap();
            request_body.images.push(filename_with_format);

        } else if name.ends_with("title") {
            let bytes = field.bytes().await.unwrap();
            let text_data: String = str::from_utf8(&bytes).unwrap().to_string();
            request_body.title = Some(text_data);
        }
    }

    let new_parent = parent_weight_item::ActiveModel{
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

pub fn process_title(title: &str) -> String{
    let split_vec = title.split(" (").take(2).collect::<Vec<&str>>();
    let [name, weight] = <[&str; 2]>::try_from(split_vec).ok().unwrap();
    let replaced = name.replace(" ", "-").to_lowercase();
    let replaced_weight = weight.replace(")", "");
    format!("{}-{}.jpg", replaced, replaced_weight)
}
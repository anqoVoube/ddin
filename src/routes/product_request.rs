use axum::{http::StatusCode, response::IntoResponse, debug_handler, Extension};

use axum_extra::extract::Multipart;

use std::{collections::HashMap, str, fs::File, io::Write, path::Path, fs};
use axum::response::Response;
use log::{error, info};
use uuid::Uuid;
use once_cell::sync::Lazy;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use sea_orm::prelude::DateTimeWithTimeZone;
use tokio::sync::Mutex;
use crate::database::parent_product;
use crate::routes::utils::{default_created, default_ok, internal_server_error};
use sea_orm::ActiveValue::Set;
static GLOBAL_DATA: Lazy<Mutex<i32>> = Lazy::new(|| {
    let mut global_count = 0;
    Mutex::new(global_count)
});


#[derive(Debug, Default)]
pub struct ObjectBody{
    title: Option<String>,
    code: Option<String>,
    file: Option<String>
}

#[debug_handler]
pub async fn upload(
    Extension(database): Extension<DatabaseConnection>,
    mut multipart: Multipart
) -> Response {
    let mut global_count = GLOBAL_DATA.lock().await;
    println!("{}", global_count);
    // let mut data = HashMap::new();
    let mut objects: HashMap<usize, ObjectBody> = HashMap::new();
    // let mut file_names: HashMap<String, String> = HashMap::new();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let count = name.split("_").collect::<Vec<&str>>().first().unwrap().parse::<usize>().unwrap();
        let mut object = objects.entry(count).or_insert(ObjectBody{..Default::default()});

        if name.ends_with("photo") {
            // Generate a unique filename for the image
            // title won't be null as it will be send before the photo
            let filename = format!("{}", process_title(&object.title.clone().unwrap()));
            let filename_with_format = format!("{}.jpg", filename);
            let dir_path = format!("media/images/{}", *global_count / 50);
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
            object.file = Some(filename_with_format);

        } else  {
            let bytes = field.bytes().await.unwrap();
            let text_data: String = str::from_utf8(&bytes).unwrap().to_string();
            if name.ends_with("title"){
                object.title = Some(text_data);
            } else {
                object.code = Some(text_data);
            }
        }
    }
    *global_count += 1;

    for object in objects.values(){
        let new_parent_product = parent_product::ActiveModel {
            title: Set(object.title.clone().unwrap()),
            code: Set(object.code.clone().unwrap()),
            description: Set("hello".to_string()),
            main_image: Set(object.file.clone()),
            images: Set(vec!()),
            ..Default::default()
        };

        match new_parent_product.save(&database).await{
            Ok(instance) => {
                info!("{:?}", instance);
            },
            Err(error) => {
                error!("Unable to create {:?}. Original error was {}", 1, error);
                internal_server_error()
            }
        }
    }

    default_ok()
}

pub fn process_title(title: &str) -> String{
    let split_vec = title.split(" (").take(2).collect::<Vec<&str>>();
    let [name, weight] = <[&str; 2]>::try_from(split_vec).ok().unwrap();
    let replaced = name.replace(" ", "-").to_lowercase();
    let replaced_weight = weight.replace(")", "");
    format!("{}-{}.jpg", replaced, replaced_weight)
}
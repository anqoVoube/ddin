use axum::{ debug_handler, Extension};

use axum_extra::extract::Multipart;

use std::{collections::HashMap, str, fs::File, io::Write, path::Path, fs};
use axum::response::Response;
use log::{error, info};
use once_cell::sync::Lazy;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use tokio::sync::Mutex;
use crate::database::parent_product;
use crate::routes::utils::{default_ok, internal_server_error};
use sea_orm::ActiveValue::Set;
static GLOBAL_DATA: Lazy<Mutex<i32>> = Lazy::new(|| {
    let global_count = 0;
    Mutex::new(global_count)
});


#[derive(Debug, Default)]
pub struct ObjectBody{
    title: Option<String>,
    expiration_in_days: Option<i32>,
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
            object.file = Some(filename);

        } else  {
            let bytes = field.bytes().await.unwrap();
            let text_data: String = str::from_utf8(&bytes).unwrap().to_string();
            if name.ends_with("title"){
                object.title = Some(text_data);
            } else if name.ends_with("expiration"){
                println!("{}", text_data);
                object.expiration_in_days = Some(text_data.parse::<i32>().unwrap());
            } else {
                object.code = Some(text_data)
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
            expiration_in_days: Set(object.expiration_in_days.unwrap()),
            ..Default::default()
        };

        match new_parent_product.save(&database).await{
            Ok(instance) => {
                info!("{:?}", instance);
            },
            Err(error) => {
                error!("Unable to create {:?}. Original error was {}", 1, error);
                return internal_server_error();
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
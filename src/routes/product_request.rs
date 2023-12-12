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
use crate::routes::utils::hash_helper::generate_uuid4;
use crate::routes::utils::title_processor::process_title;


static GLOBAL_DATA: Lazy<Mutex<i32>> = Lazy::new(|| {
    let global_count = 0;
    Mutex::new(global_count)
});


const FILE_COUNT: i32 = 50;

#[derive(Debug, Default)]
pub struct ObjectBody{
    title: Option<String>,
    expiration_in_days: Option<i32>,
    code: Option<String>,
    main_image: Option<String>,
    images: Vec<String>
}

#[debug_handler]
pub async fn upload(
    Extension(database): Extension<DatabaseConnection>,
    mut multipart: Multipart
) -> Response {
    let mut global_count = GLOBAL_DATA.lock().await;
    println!("{}", global_count);
    let mut objects: HashMap<usize, ObjectBody> = HashMap::new();
    let mut orig_names: HashMap<usize, String> = HashMap::new();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let count = name.split("_").collect::<Vec<&str>>().first().unwrap().parse::<usize>().unwrap();
        if objects.get(&count).is_none(){
            *global_count += 1;
        }
        let mut object = objects.entry(count).or_insert(ObjectBody{..Default::default()});

        if name == "main_image" {
            let filename = format!("{}.jpg", generate_uuid4());
            let dir_path = format!("media/images/{}", (*global_count - 1) / FILE_COUNT);
            if let Ok(_) = fs::create_dir_all(&dir_path){
                println!("Created directory")
            }

            // Specify the directory where the file will be saved
            let filepath = Path::new(&dir_path).join(filename.clone());
            object.main_image = Some(filename.clone());
            // Get the image data
            let file_data = field.bytes().await.unwrap();

            // Save the file
            let mut file = File::create(filepath).unwrap();
            file.write_all(&file_data).unwrap();

        } else if name.ends_with("images"){
            // todo! handle images
            println!("Received additional images... Skipping");
        } else {
            let bytes = field.bytes().await.unwrap();
            let text_data: String = str::from_utf8(&bytes).unwrap().to_string();
            if name.ends_with("title"){
                orig_names.insert(count, text_data.clone());
                object.title = Some(text_data);

            } else if name.ends_with("expiration"){
                println!("{}", text_data);
                object.expiration_in_days = Some(text_data.parse::<i32>().unwrap());
            } else {
                object.code = Some(text_data)
            }
        }
    }

    println!("Global: {}", global_count);

    // for (obj_count, gen_name) in generated_names.iter(){
    //     println!("{} {}", obj_count, gen_name);
    //     let folder_name = (*global_count - max_count as i32 + *obj_count as i32 - 1) / FILE_COUNT;
    //     fs::rename(
    //         format!(
    //             "media/images/{}/{}.jpg",
    //             folder_name,
    //             gen_name
    //         ),
    //         format!(
    //             "media/images/{}/{}",
    //             folder_name,
    //             process_title(orig_names.get(obj_count).unwrap())
    //         )
    //     ).unwrap();
    // }

    println!("{:?}", objects);
    for object in objects.values(){
        println!("creating");
        let new_parent_product = parent_product::ActiveModel {
            title: Set(object.title.clone().unwrap()),
            code: Set(object.code.clone().unwrap()),
            description: Set("hello".to_string()),
            main_image: Set(object.main_image.clone()),
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

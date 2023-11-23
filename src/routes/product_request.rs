use axum::{http::StatusCode, response::IntoResponse, debug_handler};

use axum_extra::extract::Multipart;

use std::{collections::HashMap, str, fs::File, io::Write, path::Path, fs};
use std::sync::Mutex;
use axum::response::Response;
use uuid::Uuid;
use once_cell::sync::Lazy;

static GLOBAL_DATA: Lazy<Mutex<HashMap<i32, String>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(13, "Spica".to_string());
    m.insert(74, "Hoyten".to_string());
    Mutex::new(m)
});


#[derive(Debug, Default)]
pub struct ObjectBody{
    title: Option<String>,
    code: Option<String>,
    file: Option<String>
}

#[debug_handler]
pub async fn upload(
    mut multipart: Multipart
) -> Response {
    let mut data = HashMap::new();
    let mut objects: HashMap<usize, ObjectBody> = HashMap::new();
    let mut file_names: HashMap<String, String> = HashMap::new();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let count = name.split("_").collect::<Vec<&str>>().first().unwrap().parse::<usize>().unwrap();
        let mut object = objects.entry(count).or_insert(ObjectBody{..Default::default()});

        if name.ends_with("photo") {
            // Generate a unique filename for the image
            let filename = format!("{}.jpg", object.title.unwrap());

            if let Ok(_) = fs::create_dir_all("/some/dir"){
                println!("Created directory")
            }
            // Specify the directory where the file will be saved
            let filepath = Path::new("").join(filename.clone());

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
            } else {
                object.code = Some(text_data);
            }
        }
    }

    (StatusCode::OK).into_response()
}

pub fn process_title(title: &str) -> String{
    let split_vec = title.split(" (").take(2).collect::<Vec<&str>>();
    let [name, weight] = <[&str; 2]>::try_from(split_vec).ok().unwrap();
    let replaced = name.replace(" ", "-").to_lowercase();
    let replaced_weight = weight.replace(")", "");
    format!("{}-{}.jpg", replaced, replaced_weight)
}
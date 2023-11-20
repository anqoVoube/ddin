use axum::{http::StatusCode, response::IntoResponse, routing::post, Router, Extension, debug_handler};

use axum_extra::extract::Multipart;
use http::header;
use axum_extra::extract::multipart::Field;

use std::{collections::HashMap, str, fs::File, io::Write, path::Path, fs};
use std::sync::Arc;
use axum::response::Response;
use sea_orm::DatabaseConnection;
use reqwest::multipart;
use bytes::Bytes;

// (title, code, file as field)
pub struct ObjectBody{
    title: Option<String>,
    code: Option<String>,
    file: Option<Vec<u8>>
}

#[debug_handler]
pub async fn upload(
    Extension(database): Extension<DatabaseConnection>,
    mut inc_multipart: Multipart
) -> Response {
    let mut data: HashMap<usize, ObjectBody> = HashMap::new();
    let mut headers = header::HeaderMap::new();
    headers.insert("x-api-key", header::HeaderValue::from_static("jamoliddin"));
    let mut fields: Vec<Field> = vec!();
    while let Some(field) = inc_multipart.next_field().await.unwrap(){
        fields.push(field)
    }
    for field in fields{
        let name = field.name().unwrap().to_string();
        let count = name.split("_").collect::<Vec<&str>>().first().unwrap().parse::<usize>().unwrap();
        data.entry(count).or_insert(
            ObjectBody{
                title: None,
                code: None,
                file: None
            }
        );

        if name.ends_with("photo") {
            // Generate a unique filename for the image
            let object = data.get_mut(&count).unwrap();
            object.file = Some(field.bytes().await.unwrap().to_vec());
        }
        else if name.ends_with("title"){
            let object = data.get_mut(&count).unwrap();
            object.title = Some(field.text().await.unwrap());
        }

        else {
            // Handle text data
            let object = data.get_mut(&count).unwrap();
            object.code = Some(field.text().await.unwrap());
        }
    }
    let keys: Vec<_> = data.keys().cloned().collect();
    for key in keys{
        let object: ObjectBody = data.remove(&key).unwrap();
        let filename = format!("{}.jpg", to_filename(&object.title.clone().unwrap()));
        let title: &str = &object.title.clone().unwrap();
        let code: &str = &object.code.clone().unwrap();
        // Get the image data

        // // Save the file
        // let mut file = File::create(filepath).unwrap();
        // file.write_all(&file_data).unwrap();

        //
        // let file_part = reqwest::multipart::Part::bytes(file)
        //     .file_name(filepath.as_path())
        //     .mime_str("image/jpg")
        //     .unwrap();

        let part = multipart::Part::bytes(object.file.unwrap()).file_name(filename);
        let form = multipart::Form::new()
            .text("templateId", "value1")
            .part("imageFile", part);


        // "https://beta-sdk.photoroom.com/v1/render"
        // Send the request
        let response = reqwest::Client::new().post("http://127.0.0.1:8000/")
            .multipart(form)
            .send()
            .await.unwrap();
    }



    // let filename = format!("{}-{}.jpg", Uuid::new_v4(), name);

    // // Specify the directory where the file will be saved
    // let filepath = Path::new("").join(filename);

    // // Get the image data
    // let file_data = field.bytes().await.unwrap();

    // // Save the file
    // let mut file = File::create(filepath).unwrap();
    // file.write_all(&file_data).unwrap();

    (StatusCode::OK).into_response()
}


fn to_filename(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_digit = false;

    for c in s.to_lowercase().chars() {
        match c {
            'g' if prev_was_digit => {}, // Skip 'g' if it follows a digit
            c if c.is_alphanumeric() || c == ' ' || c == '-' => {
                result.push(c);
                prev_was_digit = c.is_digit(10);
            },
            _ => prev_was_digit = false,
        }
    }

    result.replace(" ", "-")             // Replace spaces with dashes
        .split('(').next().unwrap()    // Split at '(' and take the first part
        .trim_end_matches('-')         // Trim trailing dashes if any
        .to_string()
}

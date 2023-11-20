use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};

use axum_extra::extract::Multipart;
use http::header;
use axum_extra::extract::multipart::Field;

use std::{
    collections::HashMap,
    str,
    fs::File,
    io::Write,
    path::Path,
};

// (title, code, file as field)
pub struct ObjectBody{
    title: String,
    code: String,
    file: Field
}

async fn upload(
    mut multipart: Multipart
) -> impl IntoResponse {
    
    let mut data: HashMap<usize, ObjectBody> = HashMap::new();
    let mut headers = header::HeaderMap::new();
    headers.insert("x-api-key", header::HeaderValue::from_static("jamol")).unwrap();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let count = name.split("_").collect::<Vec<&str>>().first().unwrap().parse::<usize>().unwrap();

        if name.ends_with("_photo") {
            // Generate a unique filename for the image
            let object = data.get_mut(&count).unwrap();
            object.file = field;

            
            // let form = multipart::Form::new()
            // .text("key1", "value1")
            // .text("key2", "value2")
            // // Add a file, specify the MIME type
            // .file("file_field", "path/to/your/file")?;
            
        // Send the request
        // let response = client.post("https://beta-sdk.photoroom.com/v1/render")
        //     .multipart(form)
        //     .send()
        //     .await.unwrap();
        }
        else if name.ends_with("_title"){
            let object = data.get_mut(&count).unwrap();
            object.title = field.text().await.unwrap();
        }

        else {
            // Handle text data
            let object = data.get_mut(&count).unwrap();
            object.title = field.text().await.unwrap();
        }
    }

    // let filename = format!("{}-{}.jpg", Uuid::new_v4(), name);

    // // Specify the directory where the file will be saved
    // let filepath = Path::new("").join(filename);

    // // Get the image data
    // let file_data = field.bytes().await.unwrap();

    // // Save the file
    // let mut file = File::create(filepath).unwrap();
    // file.write_all(&file_data).unwrap();

    StatusCode::OK
}
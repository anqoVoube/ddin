// use std::env;
// use rusoto_core::{Region, HttpClient, RusotoError};
// use rusoto_credential::{StaticProvider, CredentialsError};
// use rusoto_s3::{S3Client, S3, PutObjectRequest};
// use dotenvy_macro::dotenv;
//
// pub async fn upload_to_space(file_content: Vec<u8>, file_name: String) -> Result<(), Box<dyn std::error::Error>> {
//     // Your DigitalOcean Spaces credentials
//     let (access_key, secret_key, bucket_name) = fetch_space_secrets().await;
//
//     // Set up the S3 client configuration
//     let region = Region::Custom {
//         name: "fra1".to_owned(),
//         endpoint: "https://fra1.digitaloceanspaces.com".to_owned(),
//     };
//
//     let client = S3Client::new_with(
//         HttpClient::new()?,
//         StaticProvider::new_minimal(access_key.to_owned(), secret_key.to_owned()),
//         region,
//     );
//
//     // Create the PUT request
//     let put_request = PutObjectRequest {
//         bucket: bucket_name.to_owned(),
//         key: file_name,
//         body: Some(file_content.into()),
//         acl: Some("public-read".to_owned()),
//         ..Default::default()
//     };
//
//     // Send the request
//     match client.put_object(put_request).await {
//         Ok(_) => println!("File uploaded successfully."),
//         Err(e) => return Err(Box::new(e)),
//     }
//
//     Ok(())
// }
//
//
// async fn fetch_space_secrets<'a>() -> (&'a str, &'a str, &'a str){
//     (dotenv!("SPACE_ACCESS_KEY"), dotenv!("SPACE_SECRET_KEY"), dotenv!("SPACE_BUCKET_NAME"))
// }
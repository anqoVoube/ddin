use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use axum::response::{Response, IntoResponse};
use axum::extract::Path;
use axum::Json;
use http::StatusCode;


#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseBody{
    result: Vec<String>
}

pub async fn google_search_title_by_code(
    Path(code): Path<String>
) -> Response{
    let url = "https://google.serper.dev/search";
    let payload = json!({
        "q": code
    });

    let client = reqwest::Client::new();
    let res = client.post(url)
        .header("X-API-KEY", "3b89e716e6543f4972299525884d29aad40f8da9")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await.expect("Failed to send request");

    let response_text = res.text().await.expect("Failed to get response text");

    let v: Value = serde_json::from_str(&response_text).unwrap();

    // Extract titles from the JSON
    let mut titles = v["organic"]
        .as_array()
        .ok_or("Expected 'organic' to be an array").expect("Failed to get organic array")
        .iter()
        .filter_map(|entry| entry["title"].as_str())
        .map(String::from)
        .collect::<Vec<String>>();

    titles.append(&mut barcode_site_inner(code).await);

    (
        StatusCode::OK,
        Json(
            ResponseBody{
                result: titles
            }
        )
    ).into_response()
}


pub async fn barcode_site_inner(code: String) -> Vec<String>{
    let url = format!("https://barcode-list.ru/barcode/RU/barcode-{}/%D0%9F%D0%BE%D0%B8%D1%81%D0%BA.htm", code);

    let client = reqwest::Client::new();
    let res = client.get(url)
        .send()
        .await.expect("Failed to send request");
    let response_text = res.text().await.expect("Failed to get response text");
    let table = table_extract::Table::find_first(&response_text).unwrap();
    let mut titles: Vec<String> = vec![];
    for row in &table {
        if let Some(value) = row.get("Наименование"){
            titles.push(value.to_owned());
        }
    }
    titles
}
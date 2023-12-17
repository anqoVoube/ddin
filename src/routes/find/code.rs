use std::cmp::min;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use axum::response::{Response, IntoResponse};
use axum::extract::Path;
use axum::{debug_handler, Extension, Json};
use chrono::format::parse;
use http::StatusCode;
use rusqlite::{params, Connection, Result, ToSql, Statement};
use crate::routes::SqliteDBConnection;

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseBody{
    pub result: Vec<String>
}


#[derive(Debug, Clone)]
pub struct Data{
    name: String
}

#[debug_handler]
pub async fn google_search_title_by_code(
    Extension(SqliteDBConnection { sqlite }): Extension<SqliteDBConnection>,
    Path(code): Path<String>
) -> Response {
    let brah: Vec<String> = match sqlite.lock().await.prepare("SELECT name FROM barcodes WHERE barcode=:code;") {
        Ok(mut value) => match value.query_map(&[(":code", &code.clone())], |row| {
            Ok(Data {
                name: row.get(0).unwrap_or("ou".to_owned()),
            })
        }) {
            Ok(titles) => {
                let mut title_names: Vec<String> = vec!();
                for title in titles {
                    title_names.push(title.unwrap().name);
                }
                title_names
            },
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(
                        ResponseBody {
                            result: vec!()
                        }
                    )
                ).into_response();
            }
        },
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    ResponseBody {
                        result: vec!()
                    }
                )
            ).into_response();
        }
    };

    if brah.len() > 0 {
        return (
            StatusCode::OK,
            Json(
                ResponseBody {
                    result: brah
                }
            )
        ).into_response();
    }
    if let Some(value) = uz_catalog_site_inner(code.clone()).await{
        return (
            StatusCode::OK,
            Json(
                ResponseBody {
                    result: vec!(value)
                }
            )
        ).into_response();
    }

    let url = "https://google.serper.dev/search";
    let payload = json!({
        "q": code.clone()
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

    let mut final_result = vec!();
    for i in 0..min(2, titles.len()) {
        final_result.push(titles[i].clone());
    }
    final_result.append(&mut barcode_site_inner(code.clone()).await);
    final_result.extend_from_slice(&titles[2..]);
    let transformed_vec: Vec<String> = final_result
        .into_iter()
        .filter(|s| !s.contains("Barcode-list.ru"))
        .map(|s| s.replace((&format!("({})", code)), "").replace((&format!("{}", code)), ""))
        .collect();
    return (
        StatusCode::OK,
        Json(
            ResponseBody {
                result: titles
            }
        )
    ).into_response();
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

#[derive(Serialize, Deserialize, Debug)]
struct UzCatalogResponse {
    suggestions: Vec<Suggestion>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Suggestion {
    value: String,
    data: SuggestionData,
}

#[derive(Serialize, Deserialize, Debug)]
struct SuggestionData {
    data: String,
    value: String,
    #[serde(rename = "type")]
    type_field: String,
}

pub async fn uz_catalog_site_inner(code: String) -> Option<String>{
    let url = format!("https://catalog.milliykatalogi.uz/search/getSearchResult?type=goods&query={}", code);
    let response = reqwest::get(url).await.unwrap().text().await.unwrap();
    let parsed: UzCatalogResponse = serde_json::from_str(&response).unwrap();
    if parsed.suggestions.len() > 0{
        Some(parsed.suggestions[0].value.clone())
    } else {
        None
    }
}
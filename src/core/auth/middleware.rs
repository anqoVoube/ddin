use std::sync::Arc;
use axum::{debug_handler, Extension};
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use redis::aio::Connection;
use redis::AsyncCommands;
use sea_orm::{DatabaseConnection, EntityTrait};
use tokio::sync::Mutex;
use tower_cookies::{Cookie, Cookies};
use crate::database::prelude::User;
use crate::database::user::Model as UserModel;
use crate::database::prelude::Business;
use crate::routes::AppState;

const SESSION_KEY: &str = "session-key";

#[derive(Copy, Clone)]
pub struct Auth{
    pub user_id: i32,
    pub business_id: Option<i32>
}


// Response::builder()
//     .status(StatusCode::BAD_REQUEST)
//     .body(Body::from("Missing x-business-id header"))
//     .unwrap()

pub async fn auth_getter<B>(
    State(state): State<AppState>,
    cookies: Cookies,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, Response<Body>>{
    let headers = request.headers();
    let business_id: Option<i32> = match headers.get("x-business-id"){
        Some(business_header_value) => Ok(Some(business_header_value.to_str().map_err(|_error|
            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Invalid x-business-id header"))
                .unwrap()
        )?.to_owned().parse::<i32>().map_err(|_error| Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Invalid x-business-id header"))
            .unwrap())?)),
        None => Ok(None)
    }?;


    let extensions = request.extensions_mut();
    let con: Arc<Mutex<Connection>> = state.redis;
    if let Some(session_id) = cookies.get(SESSION_KEY){
        let mut locked_con = con.lock().await; // Lock the Mutex
        let user_id: i32 = locked_con.get(session_id.value()).await.unwrap();
        extensions.insert(
            Auth{
                user_id,
                business_id
            }
        );
    } else {
        return Err(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from("You have no session"))
            .unwrap());
    };
    Ok(next.run(request).await)
}

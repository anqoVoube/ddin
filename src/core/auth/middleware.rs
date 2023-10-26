use std::sync::Arc;
use axum::{debug_handler, Extension};
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use redis::aio::Connection;
use redis::AsyncCommands;
use sea_orm::{DatabaseConnection, EntityTrait};
use tokio::sync::Mutex;
use crate::database::prelude::User;
use crate::database::user::Model as UserModel;
use crate::database::prelude::Business;
use crate::routes::AppState;


struct Auth{
    user_id: i32,
    business_id: i32,
}

pub async fn auth_getter<B>(
    State(_state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode>{
    // let mut con: Arc<Mutex<Connection>> = state.redis;
    // let mut locked_con = con.lock().await; // Lock the Mutex
    // let _: () = locked_con.set("my_key", 42i32).await.unwrap();
    // let answer: i32 = locked_con.get("my_key").await.unwrap();
    // assert_eq!(locked_con.get("my_key").await, Ok(42i32));
    // println!("{:?}", answer);
    // dbg!(cookie);
    // let headers = request.headers();
    // let message = headers.get("X-Business-Id").ok_or(StatusCode::BAD_REQUEST)?;
    // let message = message.to_str().map_err(|_error| StatusCode::BAD_REQUEST)?.to_owned();
    let extensions = request.extensions_mut();
    let user_id = 1;
    let business_id = 1;
    extensions.insert(
        Auth{
            user_id,
            business_id
        }
    );
    Ok(next.run(request).await)
}

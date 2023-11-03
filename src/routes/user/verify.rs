use std::sync::Arc;
use axum::{debug_handler, Extension, Json};
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use log::error;
use redis::aio::Connection;
use redis::AsyncCommands;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};
use crate::database::prelude::{User, Verification};
use crate::database::user;
use crate::routes::utils::{bad_request, default_ok, internal_server_error};
use sea_orm::ActiveValue::Set;
use tokio::sync::Mutex;
use crate::routes::AppConnections;

const SESSION_KEY: &str = "session-key";

#[derive(Deserialize)]
pub struct Body {
    verification_id: i32,
    verification_code: i32
}

#[debug_handler]
pub async fn verify(
    Extension(AppConnections{redis, database}): Extension<AppConnections>,
    cookies: Cookies,
    Json(Body{verification_id, verification_code}): Json<Body>
) -> Response {
    println!("{}, {}", verification_id, verification_code);
    match Verification::find_by_id(verification_id).one(&database).await {
        Ok(Some(verification)) => {
            if verification.code == verification_code {
                match User::find_by_id(verification.user_id).one(&database).await {
                    Ok(Some(user)) => {
                        let mut user: user::ActiveModel = user.into();
                        user.is_verified = Set(true);
                        match user.update(&database).await {
                            Ok(user) => {
                                let mut redis_conn = redis.get().await.expect("Failed to get Redis connection");
                                let _: () = redis_conn.set(user.id, user.id).await.unwrap();
                                let mut cookie = Cookie::new(SESSION_KEY, user.id.to_string());
                                cookie.set_secure(true);
                                cookie.set_http_only(true);
                                cookies.add(cookie);
                                return default_ok();
                            },
                            Err(error) => {
                                error!("Couldn't update user with id: {}. Original error is: {}", verification.user_id, error);
                                return internal_server_error();
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        error!("Couldn't fetch user with id: {}. Original error is: {}", verification.user_id, error);
                        return internal_server_error();
                    }
                }
                // todo! create session


                default_ok()

            }
            else {
                bad_request("Invalid verification code")
            }
        },
        Ok(None) => {
            bad_request("Invalid verification id")
        },
        Err(error) => {
            error!("Couldn't fetch verification with id: {}. Original error is: {}", verification_id, error);
            internal_server_error()
        }
    }
}
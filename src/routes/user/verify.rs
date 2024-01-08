use strum_macros::Display;
use strum_macros::EnumString;
use std::sync::Arc;
use axum::{debug_handler, Extension, Json};
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use log::error;
use redis::aio::Connection;
use redis::AsyncCommands;
use sea_orm::{ActiveModelTrait, Condition, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};
use crate::database::prelude::{User, Verification};
use crate::database::user;
use crate::routes::utils::{bad_request, cookie, default_ok, internal_server_error, internal_server_error_with_log, not_found};
use sea_orm::ActiveValue::Set;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};
use std::str::FromStr;
use tower_cookies::cookie::SameSite;
use crate::{create_model, define_model, RedisPool};
use crate::database::user::ActiveModel;
use crate::routes::user::{AuthType, CODE, FIRST_NAME, LAST_NAME, PHONE_NUMBER, TYPE};
use sea_orm::{ColumnTrait, QueryFilter};


#[derive(Deserialize)]
pub struct Body {
    verification_id: String,
    verification_code: String
}

// #[debug_handler]
// pub async fn verify(
//     Extension(redis): Extension<RedisPool>,
//     Extension(database): Extension<DatabaseConnection>,
//     cookies: Cookies,
//     Json(Body{verification_id, verification_code}): Json<Body>
// ) -> Response {
//     println!("{}, {}", verification_id, verification_code);
//     let mut redis_conn = redis.get().await.expect("Failed to get Redis connection.");
//     let user_id = redis_conn.hgetall(verification_id).await.unwrap();
//     match Verification::find_by_id(verification_id).one(&database).await {
//         Ok(Some(verification)) => {
//             if 123456 == verification_code {
//                 match User::find_by_id(verification.user_id).one(&database).await {
//                     Ok(Some(user)) => {
//                         let mut user: user::ActiveModel = user.into();
//                         user.is_verified = Set(true);
//                         match user.update(&database).await {
//                             Ok(user) => {
//                                 let mut redis_conn = redis.get().await.expect("Failed to get Redis connection");
//                                 let _: () = redis_conn.set(user.id, user.id).await.unwrap();
//                                 let mut cookie = Cookie::new(SESSION_KEY, user.id.to_string());
//                                 cookie.set_secure(true);
//                                 cookie.set_http_only(true);
//                                 cookie.set_same_site(SameSite::Strict);
//                                 cookie.set_domain("ddin.uz");
//                                 cookie.set_path("/");
//                                 cookies.add(cookie);
//                                 return default_ok();
//                             },
//                             Err(error) => {
//                                 error!("Couldn't update user with id: {}. Original error is: {}", verification.user_id, error);
//                                 return internal_server_error();
//                             }
//                         }
//                     }
//                     Ok(None) => {}
//                     Err(error) => {
//                         error!("Couldn't fetch user with id: {}. Original error is: {}", verification.user_id, error);
//                         return internal_server_error();
//                     }
//                 }
//                 // todo! create session
//
//
//                 default_ok()
//
//             }
//             else {
//                 bad_request("Invalid verification code")
//             }
//         },
//         Ok(None) => {
//             bad_request("Invalid verification id")
//         },
//         Err(error) => {
//             error!("Couldn't fetch verification with id: {}. Original error is: {}", verification_id, error);
//             internal_server_error()
//         }
//     }
// }


#[debug_handler]
pub async fn verify(
    Extension(redis): Extension<RedisPool>,
    Extension(database): Extension<DatabaseConnection>,
    cookies: Cookies,
    Json(Body{verification_id, verification_code}): Json<Body>
) -> Response {
    println!("{}, {}", verification_id, verification_code);
    let mut redis_conn = redis.get().await.expect("Failed to get Redis connection.");

    let mut data: HashMap<String, String> = redis_conn.hgetall(verification_id).await.unwrap();

    let auth_type = data.remove(TYPE).unwrap();
    let code = data.remove(CODE).unwrap();
    // TODO: use real code
    if verification_code == "123456" {
        match AuthType::from_str(&auth_type).unwrap() {
            AuthType::Register => {
                let (first_name, last_name, phone_number, is_verified) = (
                    data.remove(FIRST_NAME).unwrap(),
                    data.remove(LAST_NAME).unwrap(),
                    data.remove(PHONE_NUMBER).unwrap(),
                    // TODO: remove is_verified
                    true
                );

                match create_model!(ActiveModel, &database, first_name, last_name, phone_number, is_verified) {
                    Ok(user) => {
                        let user_id: i32 = user.id.unwrap();
                        let _: () = redis_conn.set(user_id, user_id).await.unwrap();
                        let set_cookie = cookie::create(user_id.to_string());
                        cookies.add(set_cookie);
                        default_ok()
                    },
                    Err(e) => {
                        error!("{}", e);
                        internal_server_error()
                    }
                }
            },
            AuthType::Login => {
                let mut condition = Condition::all();
                let phone_number = data.remove(PHONE_NUMBER).unwrap();
                condition = condition.add(user::Column::PhoneNumber.eq(phone_number));
                match User::find().filter(condition).one(&database).await{
                    Ok(Some(user)) => {
                        let user_id: i32 = user.id.into();
                        let _: () = redis_conn.set(user_id, user_id).await.unwrap();
                        let set_cookie = cookie::create(user_id.to_string());
                        cookies.add(set_cookie);
                        default_ok()
                    },
                    Ok(None) => not_found(),
                    Err(e) => internal_server_error_with_log(e)
                }
            }
        }
    } else {
        bad_request("Incorrect code, try again.")
    }
}
use axum::{debug_handler, Extension, Json};
use axum::response::Response;
use log::error;
use redis::AsyncCommands;
use sea_orm::{ActiveModelTrait, Condition, DatabaseConnection, EntityTrait};
use serde::Deserialize;
use tower_cookies::Cookies;
use crate::database::prelude::User;
use crate::database::user;
use crate::routes::utils::{bad_request, cookie, default_ok, generate, internal_server_error, internal_server_error_with_log, not_found};
use sea_orm::ActiveValue::Set;
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;
use crate::{create_model, RedisPool};
use crate::database::user::ActiveModel;
use crate::routes::user::{AuthType, CODE, FIRST_NAME, LAST_NAME, PHONE_NUMBER, TYPE};
use sea_orm::{ColumnTrait, QueryFilter};


#[derive(Deserialize)]
pub struct Body {
    verification_id: String,
    verification_code: String
}

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
    if verification_code == code {
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
                        let user_id = user.id.unwrap().to_string();
                        let session_key = generate::uuid4();
                        let _: () = redis_conn.set(&session_key, &user_id).await.unwrap();
                        cookies.add(cookie::create(session_key));
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
                        let user_id = user.id.to_string();
                        let session_key = generate::uuid4();
                        let _: () = redis_conn.set(&session_key, &user_id).await.unwrap();
                        cookies.add(cookie::create(session_key));
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
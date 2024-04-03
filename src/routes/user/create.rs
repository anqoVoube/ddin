use axum::{debug_handler, Extension, Json};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sea_orm::{Condition, DatabaseConnection, EntityTrait, QueryFilter, ActiveModelTrait};
use sea_orm::ActiveValue::Set;
use crate::database::user::Entity as User;
use crate::database::{user, verification};
use crate::routes::utils::{bad_request, created, default_created, internal_server_error};

use sea_orm::ColumnTrait;
use crate::database::verification::Entity as Verification;

use rand::{Rng, thread_rng};
use redis::AsyncCommands;
use sea_orm::prelude::DateTimeWithTimeZone;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use crate::RedisPool;
use crate::routes::AppConnections;
use crate::routes::user::{AuthType, CODE, FIRST_NAME, LAST_NAME, PHONE_NUMBER, send_verification_code, TYPE, VerificationData};
use crate::routes::utils::{check::is_valid_phone_number, generate::six_digit_number, hash_helper::generate_uuid4};


#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct Body {
    first_name: String,
    last_name: String,
    phone_number: String
}



// #[debug_handler]
// pub async fn create(
//     Extension(database): Extension<DatabaseConnection>,
//     cookies: Cookies,
//     Json(Body{ first_name, last_name, phone_number}): Json<Body>,
// ) -> Response {
//     if !is_valid_phone_number(&phone_number) {
//         return bad_request("Invalid phone number");
//     }
//     println!("{:?}", cookies);
//     let mut condition = Condition::all();
//     condition = condition.add(user::Column::PhoneNumber.eq(phone_number.clone()));
//     match User::find().filter(condition).one(&database).await{
//         Ok(Some(user)) => {
//             if user.is_verified {
//                 return bad_request("Phone number is already registered");
//             }
//
//             let mut condition = Condition::all();
//             condition = condition.add(verification::Column::UserId.eq(user.id));
//             match Verification::find().filter(condition).one(&database).await{
//                 Ok(Some(instance)) => {
//                     let mut instance: verification::ActiveModel = instance.into();
//                     instance.code = Set(generate_six_digit_number());
//
//                     instance.expiration = Set(DateTimeWithTimeZone::from(chrono::Utc::now() + chrono::Duration::minutes(5)));
//
//                     match instance.update(&database).await{
//                         Ok(_) => {
//                             // todo! create session
//
//                             println!("verification created");
//                             default_created()
//                         },
//                         Err(err) => {
//                             println!("{}", err);
//                             internal_server_error()
//                         }
//                     }
//                 },
//                 Err(err) => {
//                     println!("{}", err);
//                     return internal_server_error();
//                 },
//                 _ => {
//                     println!("Verification instance not found for user_id: {}", user.id);
//                     return internal_server_error();
//                 }
//             }
//         },
//         Err(err) => {
//             println!("{}", err);
//             return internal_server_error();
//         },
//         _ => {
//             let new_user = user::ActiveModel {
//                 first_name: Set(first_name),
//                 last_name: Set(last_name),
//                 phone_number: Set(phone_number),
//                 is_verified: Set(false),
//                 ..Default::default()
//             };
//             match new_user.save(&database).await{
//                 Ok(user) => {
//                     let user_id = user.id.unwrap();
//                     println!("User created");
//                     let new_verification = verification::ActiveModel {
//                         user_id: Set(user_id),
//                         code: Set(generate_six_digit_number()),
//                         expiration: Set(DateTimeWithTimeZone::from(chrono::Utc::now() + chrono::Duration::minutes(5))),
//                         ..Default::default()
//                     };
//
//                     match new_verification.save(&database).await {
//                         Ok(verification) => {
//                             let verification_id = verification.id.unwrap();
//                             println!("Verification created");
//                             (
//                                 StatusCode::CREATED,
//                                 Json(VerificationData{verification_id})
//                             ).into_response()
//                         },
//                         Err(err) => {
//                             println!("{}", err);
//                             internal_server_error()
//                         }
//                     }
//                 },
//                 Err(err) => {
//                     println!("{}", err);
//                     internal_server_error()
//                 }
//             }
//         }
//     }
// }


#[debug_handler]
pub async fn create(
    Extension(database): Extension<DatabaseConnection>,
    Extension(redis): Extension<RedisPool>,
    cookies: Cookies,
    Json(Body{ first_name, last_name, phone_number}): Json<Body>,
) -> Response {
    println!("{} {} {}", first_name, last_name, phone_number);
    let mut condition = Condition::all();
    condition = condition.add(user::Column::PhoneNumber.eq(phone_number.clone()));
    match User::find().filter(condition).one(&database).await{
        Ok(Some(user)) => {
            return bad_request("Phone number already exists");
        },
        Err(err) => {
            println!("{}", err);
            return internal_server_error();
        },
        _ => {
            let verification_id = generate_uuid4();
            let mut redis_conn = redis.get().await.expect("Failed to get Redis connection");
            let _: () = redis_conn.hset_multiple(
                &verification_id,
                &*vec![
                    (TYPE, AuthType::Register.to_string()),
                    (FIRST_NAME, first_name),
                    (LAST_NAME, last_name),
                    (PHONE_NUMBER, phone_number),
                    (CODE, six_digit_number())
                ]).await.unwrap();
            let _: () = redis_conn.expire(&verification_id, 300).await.unwrap();
            send_verification_code(&verification_id, &phone_number);
            return (
                StatusCode::OK,
                Json(VerificationData{verification_id})
            ).into_response()
        }
    }
}

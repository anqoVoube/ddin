use teloxide::types::ChatId;
use axum::{debug_handler, Extension, Json};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use sea_orm::{Condition, DatabaseConnection, EntityTrait, QueryFilter, ActiveModelTrait};

use crate::database::{telegram_user, user};
use crate::routes::utils::{bad_request, created, default_created, internal_server_error};

use sea_orm::ColumnTrait;
use teloxide::Bot;
use teloxide::requests::Requester;
use tower_cookies::Cookies;
use crate::database::prelude::{User, TelegramUser};
use crate::RedisPool;
use crate::routes::user::{AuthType, CODE, PHONE_NUMBER, send_verification_code, TYPE, VerificationData};
use crate::routes::utils::{check::is_valid_phone_number, generate};

const SESSION_KEY: &str = "session-key";

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct Body {
    phone_number: String
}


// #[debug_handler]
// pub async fn login(
//     Extension(database): Extension<DatabaseConnection>,
//     cookies: Cookies,
//     Json(Body{ phone_number}): Json<Body>,
// ) -> Response {
//     println!("LOGIN");
//     if !is_valid_phone_number(&phone_number) {
//         return bad_request("Invalid phone number");
//     }
//     let mut condition = Condition::all();
//     condition = condition.add(user::Column::PhoneNumber.eq(phone_number.clone()));
//     match User::find().filter(condition).one(&database).await{
//         Ok(Some(user)) => {
//             if !user.is_verified {
//                 return bad_request("Phone number is not registered");
//             }
//
//             let mut condition = Condition::all();
//             condition = condition.add(verification::Column::UserId.eq(user.id));
//             match Verification::find().filter(condition).one(&database).await{
//                 Ok(Some(instance)) => {
//                     if instance.expiration < DateTimeWithTimeZone::from(chrono::Utc::now()) {
//                         let mut instance: verification::ActiveModel = instance.into();
//                         instance.code = Set(generate_six_digit_number());
//                         instance.expiration = Set(DateTimeWithTimeZone::from(chrono::Utc::now() + chrono::Duration::minutes(5)));
//                         match instance.update(&database).await{
//                             Ok(updated_instance) => {
//                                 // todo! create session
//                                 println!("updated");
//                                 println!("verification created");
//                                 (
//                                     StatusCode::CREATED,
//                                     Json(VerificationData{verification_id: updated_instance.id})
//                                 ).into_response()
//                             },
//                             Err(err) => {
//                                 println!("{}", err);
//                                 internal_server_error()
//                             }
//                         }
//                     } else {
//                         (
//                             StatusCode::OK,
//                             Json(VerificationData{verification_id: instance.id})
//                         ).into_response()
//                     }
//
//                 },
//                 Ok(None) => {
//                     let user_id = user.id;
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
//                     return internal_server_error();
//                 },
//
//             }
//         },
//         Err(err) => {
//             println!("{}", err);
//             return internal_server_error();
//         },
//         _ => {
//             bad_request("Phone number is not registered")
//         }
//     }
// }

#[debug_handler]
pub async fn login(
    Extension(database): Extension<DatabaseConnection>,
    Extension(redis): Extension<RedisPool>,
    Extension(bot): Extension<Bot>,
    Json(Body{ phone_number}): Json<Body>,
) -> Response {
    println!("Еперный бабай!");
    if !is_valid_phone_number(&phone_number) {
        return bad_request("Invalid phone number");
    }
    let mut condition = Condition::all();
    condition = condition.add(user::Column::PhoneNumber.eq(&phone_number));
    match User::find().filter(condition).one(&database).await{
        Ok(Some(user)) => {
            let user_id = user.id.to_string();
            let mut redis_conn = redis.get().await.expect("Failed to get Redis connection.");
            let possible_verification_id: RedisResult<String> = redis_conn.get(&user_id).await;
            let verification_code = generate::five_digit_number();
            let verification_id = match possible_verification_id {
                Ok(verification_id) => verification_id.to_string(),
                Err(_) => {
                    let verification_id = generate::uuid4();
                    let _: () = redis_conn.set(&user_id, &verification_id).await.unwrap();
                    let _: () = redis_conn.hset_multiple(
                        &verification_id,
                        &*vec![
                            (TYPE, AuthType::Login.to_string()),
                            (PHONE_NUMBER, phone_number),
                            (CODE, verification_code.clone())
                        ]).await.unwrap();
                    let _: () = redis_conn.expire(user_id, 120).await.unwrap();
                    let _: () = redis_conn.expire(&verification_id, 130).await.unwrap();
                    send_verification_code(&verification_code, &phone_number);
                    // let mut condition = Condition::all();
                    // condition = condition.add(telegram_user::Column::UserId.eq(user.id));
                    // match TelegramUser::find().filter(condition).one(&database).await {
                    //     Ok(Some(tg_user)) => {
                    //         println!("Trying to send message");
                    //         bot.send_message(ChatId(tg_user.telegram_id), verification_code).await.unwrap();
                    //     },
                    //     Ok(None) => {println!("{}", "NOT ENOUGH!")}
                    //     Err(e) => {println!("{}", e)},
                    // };
                    verification_id
                }
            };
            (
                StatusCode::OK,
                Json(
                    VerificationData {
                        verification_id
                    }
                )
            ).into_response()
        }
        Err(err) => {
            println!("{}", err);
            internal_server_error()
        },
        _ => {
            bad_request("Phone number is not registered")
        }
    }
}


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
use sea_orm::prelude::DateTimeWithTimeZone;
use tower_cookies::Cookies;

const SESSION_KEY: &str = "session-key";

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct Body {
    phone_number: String
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct VerificationData {
    verification_id: i32,
}

#[debug_handler]
pub async fn login(
    Extension(database): Extension<DatabaseConnection>,
    cookies: Cookies,
    Json(Body{ phone_number}): Json<Body>,
) -> Response {
    println!("LOGIN");
    if !is_valid_phone_number(&phone_number) {
        return bad_request("Invalid phone number");
    }
    let mut condition = Condition::all();
    condition = condition.add(user::Column::PhoneNumber.eq(phone_number.clone()));
    match User::find().filter(condition).one(&database).await{
        Ok(Some(user)) => {
            if !user.is_verified {
                return bad_request("Phone number is not registered");
            }

            let mut condition = Condition::all();
            condition = condition.add(verification::Column::UserId.eq(user.id));
            match Verification::find().filter(condition).one(&database).await{
                Ok(Some(instance)) => {
                    if instance.expiration < DateTimeWithTimeZone::from(chrono::Utc::now()) {
                        let mut instance: verification::ActiveModel = instance.into();
                        instance.code = Set(generate_six_digit_number());
                        instance.expiration = Set(DateTimeWithTimeZone::from(chrono::Utc::now() + chrono::Duration::minutes(5)));
                        match instance.update(&database).await{
                            Ok(updated_instance) => {
                                // todo! create session
                                println!("updated");
                                println!("verification created");
                                (
                                    StatusCode::CREATED,
                                    Json(VerificationData{verification_id: updated_instance.id})
                                ).into_response()
                            },
                            Err(err) => {
                                println!("{}", err);
                                internal_server_error()
                            }
                        }
                    } else {
                        (
                            StatusCode::OK,
                            Json(VerificationData{verification_id: instance.id})
                        ).into_response()
                    }

                },
                Ok(None) => {
                    let user_id = user.id;
                    println!("User created");
                    let new_verification = verification::ActiveModel {
                        user_id: Set(user_id),
                        code: Set(generate_six_digit_number()),
                        expiration: Set(DateTimeWithTimeZone::from(chrono::Utc::now() + chrono::Duration::minutes(5))),
                        ..Default::default()
                    };

                    match new_verification.save(&database).await {
                        Ok(verification) => {
                            let verification_id = verification.id.unwrap();
                            println!("Verification created");
                            (
                                StatusCode::CREATED,
                                Json(VerificationData{verification_id})
                            ).into_response()
                        },
                        Err(err) => {
                            println!("{}", err);
                            internal_server_error()
                        }
                    }
                },
                Err(err) => {
                    println!("{}", err);
                    return internal_server_error();
                },

            }
        },
        Err(err) => {
            println!("{}", err);
            return internal_server_error();
        },
        _ => {
            bad_request("Phone number is not registered")
        }
    }
}


pub fn is_valid_phone_number(phone_number: &str) -> bool{
    if phone_number.starts_with("+998") && phone_number.len() == 13 {
        return true;
    }
    false
}


pub fn generate_six_digit_number() -> i32 {
    let mut rng = thread_rng();
    rng.gen_range(100_000..1_000_000)
}

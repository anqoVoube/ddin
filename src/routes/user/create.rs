use axum::{debug_handler, Extension, Json};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use sea_orm::{Condition, DatabaseConnection, EntityTrait, QueryFilter, ActiveModelTrait};
use sea_orm::ActiveValue::Set;
use crate::database::user::Entity as User;
use crate::database::{user, verification};
use crate::routes::utils::{bad_request, default_created, internal_server_error};

use sea_orm::ColumnTrait;
use crate::database::verification::Entity as Verification;

use rand::{Rng, thread_rng};
use sea_orm::prelude::DateTimeWithTimeZone;

fn default_as_false() -> bool {
    false
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct Body {
    first_name: String,
    last_name: String,
    phone_number: String
}


#[debug_handler]
pub async fn create(
    Extension(database): Extension<DatabaseConnection>,
    Json(Body{ first_name, last_name, phone_number}): Json<Body>
) -> Response {
    if !is_valid_phone_number(&phone_number) {
        return bad_request("Invalid phone number");
    }
    let mut condition = Condition::all();
    condition = condition.add(user::Column::PhoneNumber.eq(phone_number.clone()));
    match User::find().filter(condition).one(&database).await{
        Ok(Some(user)) => {
            if user.is_verified {
                return bad_request("Phone number is already registered");
            }

            let mut condition = Condition::all();
            condition = condition.add(verification::Column::UserId.eq(user.id));
            match Verification::find().filter(condition).one(&database).await{
                Ok(Some(instance)) => {
                    let mut instance: verification::ActiveModel = instance.into();
                    instance.code = Set(generate_six_digit_number());

                    instance.expiration = Set(DateTimeWithTimeZone::from(chrono::Utc::now() + chrono::Duration::minutes(5)));
                    match instance.update(&database).await{
                        Ok(_) => {
                            println!("verification created");
                            default_created()
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
                _ => {
                    println!("Verification instance not found for user_id: {}", user.id);
                    return internal_server_error();
                }
            }
        },
        Err(err) => {
            println!("{}", err);
            return internal_server_error();
        },
        _ => {
            let new_user = user::ActiveModel {
                first_name: Set(first_name),
                last_name: Set(last_name),
                phone_number: Set(phone_number),
                is_verified: Set(false),
                ..Default::default()
            };
            match new_user.save(&database).await{
                Ok(_) => {
                    println!("User created");
                    let new_verification = verification::ActiveModel {
                        user_id: Set(1),
                        code: Set(generate_six_digit_number()),
                        expiration: Set(DateTimeWithTimeZone::from(chrono::Utc::now() + chrono::Duration::minutes(5))),
                        ..Default::default()
                    };

                    match new_verification.save(&database).await {
                        Ok(_) => {
                            println!("Verification created");
                            default_created()
                        },
                        Err(err) => {
                            println!("{}", err);
                            internal_server_error()
                        }
                    }
                },
                Err(err) => {
                    println!("{}", err);
                    internal_server_error()
                }
            }
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

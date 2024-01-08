// use std::sync::Arc;
// use axum::{debug_handler, Extension};
// use axum::body::Body;
// use axum::extract::State;
// use axum::http::{Request, StatusCode};
// use axum::middleware::Next;
// use axum::response::{IntoResponse, Response};
// use redis::aio::Connection;
// use redis::AsyncCommands;
// use sea_orm::{DatabaseConnection, EntityTrait};
// use tokio::sync::Mutex;
// use tower_cookies::{Cookie, Cookies};
// use crate::database::business;
// use crate::database::prelude::User;
// use crate::database::user::Model as UserModel;
// use crate::database::prelude::Business;
// use crate::routes::{AppConnections};
// use crate::routes::utils::{bad_request, default_missing_header, response_builder};
//
// const SESSION_KEY: &str = "session-key";
//
// #[derive(Copy, Clone)]
// pub struct Auth{
//     pub user_id: i32,
//     pub business_id: i32
// }
//
//
// impl Auth{
//     pub async fn validate_business_id(&self, database: &DatabaseConnection) -> Result<i32, Response>{
//         match self.business_id {
//             0 => Err(
//                 (
//                     StatusCode::NOT_ACCEPTABLE,
//                     "Business with this id doesn't exist"
//                 ).into_response()
//             ),
//             value => {
//                 match Business::find_by_id(value).one(database).await {
//                     Ok(Some(_)) => {
//                         Ok(value)
//                     },
//                     _ => Err(
//                         (
//                             StatusCode::NOT_ACCEPTABLE,
//                             "Business with this id doesn't exist"
//                         ).into_response()
//                     )
//                 }
//             },
//         }
//     }
// }
//
// // Combining every possible error into one type for safety reasons
// fn create_invalid_header_response() -> Response<Body> {
//     response_builder(StatusCode::BAD_REQUEST, "Invalid x-business-id header")
// }
//
// pub async fn auth_getter<B>(
//     State(connections): State<AppConnections>,
//     cookies: Cookies,
//     mut request: Request<B>,
//     next: Next<B>,
// ) -> Result<Response, Response<Body>>{
//     let headers = request.headers();
//     println!("{:?}", headers);
//     let business_id: i32 = match headers.get("x-business-id"){
//         Some(business_header_value) => Ok(business_header_value.to_str().map_err(
//             |_error|
//             create_invalid_header_response()
//         )?.to_owned().parse::<i32>().map_err(|_error| create_invalid_header_response())?),
//         None => Err(create_invalid_header_response())
//     }?;
//
//
//     let extensions = request.extensions_mut();
//     if let Some(session_id) = cookies.get(SESSION_KEY){
//         let mut redis_conn = connections.redis.get().await.expect("Failed to get Redis connection.");
//         let user_id: i32 = redis_conn.get(session_id.value()).await.unwrap();
//         extensions.insert(
//             Auth{
//                 user_id,
//                 business_id
//             }
//         );
//     } else {
//         // todo: remove
//         extensions.insert(
//             Auth{
//                 user_id: 4,
//                 business_id: 1
//             }
//         );
//         // return Err(Response::builder()
//         //     .status(StatusCode::UNAUTHORIZED)
//         //     .body(Body::from("You have no session"))
//         //     .unwrap());
//     };
//     Ok(next.run(request).await)
// }
//
//
//
//
//


use std::sync::Arc;
use axum::{debug_handler, Extension};
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use redis::aio::Connection;
use redis::AsyncCommands;
use sea_orm::{DatabaseConnection, EntityTrait};
use tokio::sync::Mutex;
use tower_cookies::{Cookie, Cookies};
use crate::database::business;
use crate::database::prelude::User;
use crate::database::user::Model as UserModel;
use crate::database::prelude::Business;
use crate::routes::{AppConnections};
use crate::routes::utils::{bad_request, default_missing_header, response_builder};

const SESSION_KEY: &str = "session-key";

#[derive(Debug, Copy, Clone)]
pub struct Auth{
    pub user_id: i32,
}


#[derive(Debug, Copy, Clone)]
pub struct CustomHeader{
    pub business_id: i32,
}

// impl Auth{
//     pub async fn validate_business_id(&self, database: &DatabaseConnection) -> Result<i32, Response>{
//         match self.business_id {
//             0 => Err(
//                 (
//                     StatusCode::NOT_ACCEPTABLE,
//                     "Business with this id doesn't exist"
//                 ).into_response()
//             ),
//             value => {
//                 match Business::find_by_id(value).one(database).await {
//                     Ok(Some(_)) => {
//                         Ok(value)
//                     },
//                     _ => Err(
//                         (
//                             StatusCode::NOT_ACCEPTABLE,
//                             "Business with this id doesn't exist"
//                         ).into_response()
//                     )
//                 }
//             },
//         }
//     }
// }

fn create_response(status_code: StatusCode, body_text: &str) -> Response<Body> {
    response_builder(status_code, body_text)
}

pub async fn validate_business_id<B>(
    State(connections): State<AppConnections>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, Response<Body>>{
    let auth = request.extensions().get::<Auth>();
    let custom_headers = request.extensions().get::<CustomHeader>();
    match auth {
        Some(Auth{user_id}) => {
            match custom_headers {
                Some(CustomHeader{business_id}) => {
                    match Business::find_by_id(*business_id).one(&connections.database).await {
                        Ok(Some(business_instance)) => {
                            println!("OwnerID: {}, UserID: {}", business_instance.owner_id, *user_id);
                            if business_instance.owner_id == *user_id{
                                Ok(next.run(request).await)
                            } else {
                                Err(create_response(StatusCode::UNAUTHORIZED, "Authorization failed. You are not owner of requesting business"))
                            }
                        },
                        _ => Err(create_response(StatusCode::NOT_ACCEPTABLE, "Not acceptable request"))
                    }
                },
                None => {
                    return Err(create_response(StatusCode::NOT_ACCEPTABLE, "Not acceptable request"));
                }
            }
        },
        None => {
            return Err(create_response(StatusCode::UNAUTHORIZED, "Authorization failed"));
        }
    }
}


// Combining every possible error into one type for safety reasons


pub async fn auth_getter<B>(
    State(connections): State<AppConnections>,
    cookies: Cookies,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, Response<Body>>{
    let extensions = request.extensions_mut();
    if let Some(session_id) = cookies.get(SESSION_KEY){
        let mut redis_conn = connections.redis.get().await.expect("Failed to get Redis connection.");
        let user_id: i32 = redis_conn.get(session_id.value()).await.unwrap();
        extensions.insert(
            Auth{
                user_id,
            }
        );
    } else {
        // TODO: remove
        extensions.insert(
            Auth{
                user_id: 94,
            }
        );
        // return Err(Response::builder()
        //     .status(StatusCode::UNAUTHORIZED)
        //     .body(Body::from("You have no session"))
        //     .unwrap());
    };
    Ok(next.run(request).await)
}

pub async fn business_getter<B>(
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, Response<Body>>{
    let headers = request.headers();
    println!("{:?}", headers);
    let business_id: i32 = match headers.get("x-business-id"){
        Some(business_header_value) => Ok(business_header_value.to_str().map_err(
            |_error|
            create_response(StatusCode::NOT_ACCEPTABLE, "Not acceptable request")
        )?.to_owned().parse::<i32>().map_err(|_error| create_response(StatusCode::NOT_ACCEPTABLE, "Not acceptable request"))?),
        None => Err(create_response(StatusCode::NOT_ACCEPTABLE, "Not acceptable request"))
    }?;
    let extensions = request.extensions_mut();
    extensions.insert(
        CustomHeader{
            business_id,
        }
    );
    Ok(next.run(request).await)
}

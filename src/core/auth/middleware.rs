use axum::Extension;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use sea_orm::{DatabaseConnection, EntityTrait};
use crate::database::prelude::User;
use crate::database::user::Model as UserModel;
use crate::database::prelude::Business;


struct Auth{
    user_id: i32,
    business_id: i32,
}
pub async fn auth_getter<B>(
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode>{
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

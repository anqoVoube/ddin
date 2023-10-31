use axum::Extension;
use axum::response::Response;
use crate::core::auth::middleware::Auth;
use crate::routes::utils::{bad_request, default_ok};

pub async fn ping(
    Extension(auth): Extension<Auth>
) -> Response{
    match auth.business_id{
        Some(_) => default_ok(),
        None => bad_request("No business_id")
    }
}
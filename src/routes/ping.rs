use axum::Extension;
use axum::response::Response;
use crate::core::auth::middleware::Auth;
use crate::routes::AppConnections;
use crate::routes::product::create::Body;
use crate::routes::utils::{bad_request, default_missing_header, default_ok};

pub async fn ping(
    Extension(auth): Extension<Auth>,
    Extension(AppConnections{redis, database, scylla}): Extension<AppConnections>
) -> Result<Response, Response>{
    auth.validate_business_id(&database).await?;
    Ok(default_ok())
}
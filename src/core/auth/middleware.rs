use axum::{Extension, TypedHeader};
use axum::headers::Cookie;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use sea_orm::DatabaseConnection;


pub async fn get_user_by_session<B>(
    request: Request<B>,
    next: Next<B>,
    TypedHeader(cookie): TypedHeader<Cookie>,
    Extension(_): Extension<DatabaseConnection>
) -> Result<Response, StatusCode>{
    dbg!(cookie);
    // let headers = request.headers();
    // let message = headers.get("message").ok_or(StatusCode::BAD_REQUEST)?;
    // let message = message.to_str().map_err(|_error| StatusCode::BAD_REQUEST)?.to_owned();
    // let extensions = request.extensions_mut();
    //
    // extensions.insert(Auth{user_id: 2});
    Ok(next.run(request).await)
}

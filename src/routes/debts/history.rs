use axum::{debug_handler, Extension, Json};
use axum::extract::Path;
use http::StatusCode;
use sea_orm::{DatabaseConnection, EntityTrait};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::auth::middleware::Auth;
use crate::database::prelude::RentHistory;
use crate::database::rent_history;
use axum::response::{IntoResponse, Response};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;

#[derive(Serialize, Deserialize, Debug)]
struct History{
    id: i32,
    products: Value,
    buy_date: DateTimeWithTimeZone
}

#[derive(Serialize, Deserialize, Debug)]
struct Histories{
    histories: Vec<History>
}

#[debug_handler]
pub async fn get_history(
    Extension(Auth{user_id, business_id}): Extension<Auth>,
    Extension(database): Extension<DatabaseConnection>,
    Path(debt_user_id): Path<i32>
) -> Response{
    // ToDo: Check RentUser for business_id for security purposes
    let histories = RentHistory::find()
        .filter(
            rent_history::Column::RentUserId.eq(debt_user_id)
        )
        .all(&database)
        .await
        .unwrap();

    let mut response_body = Histories{histories: vec![]};

    for history in histories{
        response_body.histories.push(History{
            id: history.id,
            products: history.products,
            buy_date: history.buy_date
        })
    }

    (
        StatusCode::OK,
        Json(response_body)
    ).into_response()
}
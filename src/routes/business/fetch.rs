use axum::{Extension, Json};
use axum::extract::Path;
use http::StatusCode;
use sea_orm::{Condition, DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use serde::Serialize;
use chrono::NaiveTime;
use crate::database::business::Entity as Business;
use crate::database::business::Model as BusinessModel;
use rust_decimal::Decimal;
use log::{warn};
use crate::core::auth::middleware::{Auth, CustomHeader};
use crate::database::business;

#[derive(Serialize, Debug)]
pub struct BusinessSchema {
    id: i32,
    title: String,
    // location: Vec<Decimal>,
    // works_from: NaiveTime,
    // works_until: NaiveTime,
    // #[serde(default="default_as_false")]
    // is_closed: bool,
    // owner_id: i32
}

#[derive(Serialize, Debug)]
pub struct BusinessesSchema {
    pub businesses: Vec<BusinessSchema>
}


impl From<BusinessModel> for BusinessSchema {
    fn from(business: BusinessModel) -> Self {
        BusinessSchema {
            id: business.id,
            title: business.title,
            // location: business.location,
            // works_from: business.works_from,
            // works_until: business.works_until,
            // is_closed: business.is_closed,
            // owner_id: business.owner_id
        }
    }
}

pub async fn list(
    // Extension(Auth{user_id}): Extension<Auth>,
    Extension(database): Extension<DatabaseConnection>,
) -> Result<Json<BusinessesSchema>, StatusCode> {
    let user_id = 94;
    let businesses = Business::find()
        .filter(
            Condition::all()
                .add(business::Column::OwnerId.eq(user_id))
        )
        .all(&database)
        .await
        .map_err(|_error| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|business| business.into())
        .collect::<Vec<BusinessSchema>>();

    let business_schema: BusinessesSchema = BusinessesSchema{
        businesses
    };

    Ok(Json(business_schema))
}

pub async fn get_object(
    Extension(database): Extension<DatabaseConnection>, Path(business_id): Path<i32>
) -> Result<Json<BusinessSchema>, StatusCode> {

    let business = Business::find_by_id(business_id).one(&database).await
        .map_err(|_error| {warn!("Couldn't fetch business with id: {}", business_id); StatusCode::INTERNAL_SERVER_ERROR})?;

    if let Some(value) = business{
        Ok(Json(value.into()))
    }
    else{
        warn!("Couldn't fetch business with id: {}", business_id);
        Err(StatusCode::NOT_FOUND)
    }
}

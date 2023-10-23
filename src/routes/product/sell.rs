use axum::{Extension, Json, debug_handler};
use axum::response::{IntoResponse, Response};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use sea_orm::ActiveValue::Set;
use crate::database::product::Entity as Product;
use crate::database::product;
use crate::routes::utils::{not_found, bad_request, internal_server_error, default_created, default_ok};

fn default_as_false() -> bool {
    false
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductBody {
    id: i32,
    quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductsListBody {
    products: Vec<ProductBody>
}


#[debug_handler]
pub async fn sell(
    Extension(database): Extension<DatabaseConnection>,
    Json(products): Json<ProductsListBody>
) -> Response {
    println!("{:#?}", products);
    for product_instance in products.products{
        match Product::find_by_id(product_instance.id).one(&database).await{
            Ok(Some(pear)) => {
                let mut pear: product::ActiveModel = pear.into();

                let total = pear.quantity.unwrap();
                if product_instance.quantity > total{
                    return bad_request("Not enough products in stock");
                }

                pear.quantity = Set(total - product_instance.quantity);

                match pear.update(&database).await{
                    Ok(_) => default_created(),
                    Err(err) => {
                        println!("{:?}", err);
                        return internal_server_error();
                    }
                };
            },
            Ok(None) => {
                return not_found();
            },
            Err(err) => {
                println!("{:?}", err);
                return internal_server_error();
            }
        };
    }

    default_ok()
}

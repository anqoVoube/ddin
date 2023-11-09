use axum::{Extension, Json, debug_handler};
use axum::response::{IntoResponse, Response};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use sea_orm::ActiveValue::Set;
use crate::database::prelude::{NoCodeProduct, WeightItem};
use crate::database::product::Entity as Product;
use crate::database::{no_code_product, product, weight_item};
use crate::routes::{AppConnections};
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
pub struct NoCodeProductBody {
    id: i32,
    quantity: i32
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightItemBody {
    id: i32,
    kg_weight: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellBody {
    weight_items: Vec<WeightItemBody>,
    products: Vec<ProductBody>,
    no_code_products: Vec<NoCodeProductBody>
}


#[debug_handler]
pub async fn sell(
    Extension(database): Extension<DatabaseConnection>,
    Json(sell): Json<SellBody>
) -> Response {
    println!("{:?}", sell);
    for product_instance in sell.products{
        match Product::find_by_id(product_instance.id).one(&database).await{
            Ok(Some(pear)) => {
                let mut pear: product::ActiveModel = pear.into();

                let total = pear.quantity.clone().unwrap();
                if product_instance.quantity > total{
                    return bad_request("Not enough products in stock");
                }

                if product_instance.quantity == total {
                    if let Err(err) = pear.delete(&database).await{
                        println!("{:?}", err);
                        return internal_server_error();
                    }
                } else {
                    pear.quantity = Set(total - product_instance.quantity);

                    if let Err(err) = pear.update(&database).await {
                           println!("{:?}", err);
                           return internal_server_error();
                    }
                }
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

    for weight_item_instance in sell.weight_items{
        match WeightItem::find_by_id(weight_item_instance.id).one(&database).await{
            Ok(Some(pear)) => {
                let mut pear: weight_item::ActiveModel = pear.into();

                let total = pear.kg_weight.clone().unwrap();
                if weight_item_instance.kg_weight > total{
                    return bad_request("Not enough kg in stock");
                }
                if weight_item_instance.kg_weight == total {
                    if let Err(err) = pear.delete(&database).await {
                        println!("{:?}", err);
                        return internal_server_error();
                    }
                } else {
                    pear.kg_weight = Set(total - weight_item_instance.kg_weight);

                    if let Err(err) = pear.update(&database).await {
                        println!("{:?}", err);
                        return internal_server_error();
                    }
                }
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

    for no_code_product_instance in sell.no_code_products{
        match NoCodeProduct::find_by_id(no_code_product_instance.id).one(&database).await{
            Ok(Some(pear)) => {
                let mut pear: no_code_product::ActiveModel = pear.into();

                let total = pear.quantity.clone().unwrap();
                if no_code_product_instance.quantity > total {
                    return bad_request("Not enough no code products in stock");
                }

                if no_code_product_instance.quantity == total {
                    if let Err(err) = pear.delete(&database).await {
                        println!("{:?}", err);
                        return internal_server_error();
                    }
                } else {
                    pear.quantity = Set(total - no_code_product_instance.quantity);

                    if let Err(err) = pear.update(&database).await {
                        println!("{:?}", err);
                        return internal_server_error();
                    }
                }
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

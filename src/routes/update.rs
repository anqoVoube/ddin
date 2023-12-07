use axum::{debug_handler, Extension, Json};
use serde::{Serialize, Deserialize};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveValue::Set, ActiveModelTrait};
use axum::response::{IntoResponse, Response};
use log::error;
use crate::database::prelude::{NoCodeProduct, Product, WeightItem};
use crate::database::{no_code_product, product, weight_item};
use crate::routes::utils::{bad_request, default_ok, internal_server_error};


#[derive(Serialize, Deserialize, Debug)]
pub struct RequestBody{
    id: i32,
    item_type: i8,
    price: Option<i32>,
    quantity: Option<i32>,
    kg_weight: Option<f64>
}


#[debug_handler]
pub async fn update_product(
    Extension(database): Extension<DatabaseConnection>,
    Json(RequestBody{id, item_type, price, quantity, kg_weight}): Json<RequestBody>
) -> Response{
    match item_type {
        1 => {
            match Product::find_by_id(id).one(&database).await{
                Ok(Some(product)) => {
                    let mut product: product::ActiveModel = product.into();
                    if price.is_some(){
                        product.price = Set(price.unwrap());
                    }
                    if quantity.is_some(){
                        product.quantity = Set(quantity.unwrap());
                    }

                    match product.update(&database).await{
                        Ok(_) => {
                            // todo! create session
                            println!("verification created");
                            return default_ok();
                        },
                        Err(err) => {
                            println!("{}", err);
                            return internal_server_error();
                        }
                    }
                },

                Ok(None) => {
                    return bad_request("Product not found")
                },
                Err(err) => {
                    error!("Couldn't fetch product");
                    return internal_server_error();
                }
            }
        }
        2 => {
            match WeightItem::find_by_id(id).one(&database).await{
                Ok(Some(weight_item)) => {
                    let mut weight_item: weight_item::ActiveModel = weight_item.into();
                    if price.is_some(){
                        weight_item.price = Set(price.unwrap());
                    }
                    if kg_weight.is_some(){
                        weight_item.kg_weight = Set(kg_weight.unwrap());
                    }
                    match weight_item.update(&database).await{
                        Ok(_) => {
                            // todo! create session
                            println!("verification created");
                            return default_ok();
                        },
                        Err(err) => {
                            println!("{}", err);
                            return internal_server_error();
                        }
                    }
                },

                Ok(None) => {
                    return bad_request("Weight item not found")
                },
                Err(err) => {
                    error!("Couldn't fetch product");
                    return internal_server_error();
                }
            }
        }
        3 => {
            match NoCodeProduct::find_by_id(id).one(&database).await{
                Ok(Some(no_code_product)) => {
                    let mut no_code_product: no_code_product::ActiveModel = no_code_product.into();
                    if price.is_some(){
                        no_code_product.price = Set(price.unwrap());
                    }
                    if quantity.is_some(){
                        no_code_product.quantity = Set(quantity.unwrap());
                    }

                    match no_code_product.update(&database).await{
                        Ok(_) => {
                            // todo! create session
                            println!("verification created");
                            return default_ok();
                        },
                        Err(err) => {
                            println!("{}", err);
                            return internal_server_error();
                        }
                    }
                },

                Ok(None) => {
                    return bad_request("Product not found")
                },
                Err(err) => {
                    error!("Couldn't fetch product");
                    return internal_server_error();
                }
            }
        }
        _ => {
            return internal_server_error();
        }
    }
}

use std::sync::Arc;
use axum::{Extension, Json, debug_handler};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use scylla::{IntoTypedRows, Session};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use sea_orm::ActiveValue::Set;
use crate::core::auth::middleware::Auth;
use crate::database::prelude::{NoCodeProduct, WeightItem};
use crate::database::product::Entity as Product;
use crate::database::{no_code_product, product, weight_item};
use crate::routes::ScyllaDBConnection;
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

pub enum ItemType{
    Product,
    WeightItem,
    NoCodeProduct
}

trait EnumValue{
    fn get_value(&self) -> i8;
}

impl EnumValue for ItemType{
    fn get_value(&self) -> i8 {
        match self {
            ItemType::Product => 1,
            ItemType::WeightItem => 2,
            ItemType::NoCodeProduct => 3
        }
    }
}
#[debug_handler]
pub async fn sell(
    Extension(database): Extension<DatabaseConnection>,
    Extension(ScyllaDBConnection {scylla}): Extension<ScyllaDBConnection>,
    Extension(Auth{user_id, business_id}): Extension<Auth>,
    Json(sell): Json<SellBody>
) -> Response {
    println!("{:?}", sell);
    for product_instance in sell.products{
        match Product::find_by_id(product_instance.id).one(&database).await{
            Ok(Some(pear)) => {
                let mut pear: product::ActiveModel = pear.into();
                let profit = pear.profit.clone().unwrap();
                let total = pear.quantity.clone().unwrap();
                let parent_id = pear.parent_id.clone().unwrap();
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
                    } else {
                        let current_date = Utc::now().naive_utc().date();

                        let select_query = "SELECT quantity, profit FROM statistics.products WHERE parent_id = ? AND business_id = ? AND item_type = ? AND date = ?";
                        let result = scylla
                            .query(select_query, (parent_id, business_id, ItemType::Product.get_value(), current_date))
                            .await.unwrap();

                        match result.rows.expect("failed to get rows").into_typed::<(i32, i32)>().next() {
                            Some(row) => {
                                let (current_quantity, current_profit) = row.expect("couldn't parse");
                                let new_quantity = current_quantity + product_instance.quantity;
                                let new_profit = current_profit + product_instance.quantity * profit;
                                let update_query = "UPDATE statistics.products SET quantity = ?, profit = ? WHERE parent_id = ? AND business_id = ? AND item_type = ? AND date = ?";
                                scylla
                                    .query(update_query, (new_quantity, new_profit, parent_id, business_id, ItemType::Product.get_value(), current_date))
                                    .await.unwrap();
                            },
                            None => {
                                let insert = "INSERT INTO statistics.products (parent_id, quantity, profit, business_id, date, item_type) VALUES (?, ?, ?, ?, ?, ?);";

                                scylla.query(
                                    insert,
                                    (parent_id, product_instance.quantity, profit * product_instance.quantity, business_id, current_date, ItemType::Product.get_value())
                                ).await.expect("Tired");
                            }
                        };


                        let select_query = "SELECT profit FROM statistics.profits WHERE business_id = ? AND date = ?";
                        let result = scylla
                            .query(select_query, (business_id, current_date))
                            .await.unwrap();

                        match result.rows.expect("failed to get rows").into_typed::<(i32, )>().next() {
                            Some(row) => {
                                let (current_profit, ) = row.expect("couldn't parse");
                                let new_profit = current_profit + product_instance.quantity * profit;
                                let update_query = "UPDATE statistics.profits SET profit = ? WHERE business_id = ? AND date = ?";
                                scylla
                                    .query(update_query, (new_profit, business_id, current_date))
                                    .await.unwrap();
                            },
                            None => {
                                let insert = "INSERT INTO statistics.profits (business_id, profit, date) VALUES (?, ?, ?);";

                                scylla.query(
                                    insert,
                                    (business_id, profit * product_instance.quantity, current_date)
                                ).await.expect("Tired");
                            }
                        };
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
                let profit = pear.profit.clone().unwrap();
                let parent_id = pear.parent_id.clone().unwrap();
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
                    } else {
                        let current_date = Utc::now().naive_utc().date();

                        let select_query = "SELECT quantity, profit FROM statistics.products WHERE parent_id = ? AND business_id = ? AND item_type = ? AND date = ?";
                        let result = scylla
                            .query(select_query, (parent_id, business_id, ItemType::Product.get_value(), current_date))
                            .await.unwrap();

                        match result.rows.expect("failed to get rows").into_typed::<(i32, i32)>().next() {
                            Some(row) => {
                                let (current_quantity, current_profit) = row.expect("couldn't parse");
                                let new_quantity = current_quantity +  (weight_item_instance.kg_weight * 1000.0) as i32;
                                let new_profit = current_profit + (weight_item_instance.kg_weight * profit as f64) as i32;
                                let update_query = "UPDATE statistics.products SET quantity = ?, profit = ? WHERE parent_id = ? AND business_id = ? AND item_type = ? AND date = ?";
                                scylla
                                    .query(update_query, (new_quantity, new_profit, parent_id, business_id, ItemType::WeightItem.get_value(), current_date))
                                    .await.unwrap();
                            },
                            None => {
                                let insert = "INSERT INTO statistics.products (parent_id, quantity, profit, business_id, date, item_type) VALUES (?, ?, ?, ?, ?, ?);";

                                scylla.query(
                                    insert,
                                    (parent_id,  (weight_item_instance.kg_weight * 1000.0) as i32, (weight_item_instance.kg_weight * profit as f64) as i32, business_id, current_date, ItemType::WeightItem.get_value())
                                ).await.expect("Tired");
                            }
                        };
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
                let profit = pear.profit.clone().unwrap();
                let parent_id = pear.parent_id.clone().unwrap();

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
                    } else {
                        let current_date = Utc::now().naive_utc().date();


                        let select_query = "SELECT quantity, profit FROM statistics.products WHERE parent_id = ? AND business_id = ? AND item_type = ? AND date = ?";
                        let result = scylla
                            .query(select_query, (parent_id, business_id, ItemType::Product.get_value(), current_date))
                            .await.unwrap();

                        match result.rows.expect("failed to get rows").into_typed::<(i32, i32)>().next() {
                            Some(row) => {
                                let (current_quantity, current_profit) = row.expect("couldn't parse");
                                let new_quantity = current_quantity + no_code_product_instance.quantity;
                                let new_profit = current_profit + no_code_product_instance.quantity * profit;
                                let update_query = "UPDATE statistics.products SET quantity = ? WHERE parent_id = ? AND business_id = ? AND item_type = ? AND date = ?";
                                scylla
                                    .query(update_query, (new_quantity, parent_id, business_id, ItemType::WeightItem.get_value(), current_date))
                                    .await.unwrap();
                            },
                            None => {
                                let insert = "INSERT INTO statistics.products (parent_id, quantity, profit, business_id, date, item_type) VALUES (?, ?, ?, ?, ?, ?);";

                                scylla.query(
                                    insert,
                                    (parent_id, no_code_product_instance.quantity, no_code_product_instance.quantity * profit, business_id, current_date, ItemType::WeightItem.get_value())
                                ).await.expect("Tired");
                            }
                        };

                        let select_query = "SELECT profit FROM statistics.profits WHERE business_id = ? AND date = ?";
                        let result = scylla
                            .query(select_query, (business_id, current_date))
                            .await.unwrap();

                        match result.rows.expect("failed to get rows").into_typed::<(i32, )>().next() {
                            Some(row) => {
                                let (current_profit, ) = row.expect("couldn't parse");
                                let new_profit = current_profit + no_code_product_instance.quantity * profit;
                                let update_query = "UPDATE statistics.profits SET profit = ? WHERE business_id = ? AND date = ?";
                                scylla
                                    .query(update_query, (new_profit, business_id, current_date))
                                    .await.unwrap();
                            },
                            None => {
                                let insert = "INSERT INTO statistics.profits (business_id, profit, date) VALUES (?, ?, ?);";

                                scylla.query(
                                    insert,
                                    (business_id, profit * no_code_product_instance.quantity, current_date)
                                ).await.expect("Tired");
                            }
                        };
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

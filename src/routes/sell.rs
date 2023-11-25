use std::sync::Arc;
use axum::{Extension, Json, debug_handler};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use log::{error, info};
use scylla::{IntoTypedRows, Session};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::{DateTimeUtc, DateTimeWithTimeZone};
use serde_json::json;
use crate::core::auth::middleware::Auth;
use crate::database::prelude::{NoCodeProduct, Rent, WeightItem};
use crate::database::product::Entity as Product;
use crate::database::{no_code_product, product, rent, rent_history, weight_item};
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
pub struct ParentProductBody {
    parent_id: i32,
    quantity: i32,
    sell_price: i32
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoCodeProductBody {
    id: i32,
    quantity: i32,
    sell_price: i32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentNoCodeProductBody {
    parent_id: i32,
    quantity: i32,
    sell_price: i32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightItemBody {
    id: i32,
    kg_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentWeightItemBody {
    parent_id: i32,
    kg_weight: f64,
    sell_price: i32
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebtUserBody{
    id: i32,
    paid_price: i32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellBody {
    weight_items: Vec<WeightItemBody>,
    products: Vec<ProductBody>,
    no_code_products: Vec<NoCodeProductBody>,
    debt_user: Option<DebtUserBody>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RentHistoryProducts{
    weight_items: Vec<ParentWeightItemBody>,
    products: Vec<ParentProductBody>,
    no_code_products: Vec<ParentNoCodeProductBody>,
}

pub enum ItemType{
    Product,
    WeightItem,
    NoCodeProduct
}

pub trait EnumValue{
    fn get_value(&self) -> i8;
    fn from_value(value: i8) -> Self;
}

impl EnumValue for ItemType{
    fn get_value(&self) -> i8 {
        match self {
            ItemType::Product => 1,
            ItemType::WeightItem => 2,
            ItemType::NoCodeProduct => 3
        }
    }

    fn from_value(value: i8) -> Self {
        match value {
            1 => ItemType::Product,
            2 => ItemType::WeightItem,
            3 => ItemType::NoCodeProduct,
            _ => panic!("Invalid item type")
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
    let mut history_products: Vec<ParentProductBody> = vec!();
    let mut history_weight_items: Vec<ParentWeightItemBody> = vec!();
    let mut history_no_code_products: Vec<ParentNoCodeProductBody> = vec!();
    let mut grant_total = 0;
    for product_instance in &sell.products{
        match Product::find_by_id(product_instance.id)
            .one(&database).await{
            Ok(Some(pear)) => {
                let mut pear: product::ActiveModel = pear.into();
                let profit = pear.profit.clone().unwrap();
                let total = pear.quantity.clone().unwrap();
                let parent_id = pear.parent_id.clone().unwrap();
                let price = pear.price.clone().unwrap();
                grant_total += product_instance.quantity * price;

                history_products.push(ParentProductBody{
                    parent_id: parent_id,
                    quantity: product_instance.quantity,
                    sell_price: price
                });

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

    for weight_item_instance in &sell.weight_items{
        match WeightItem::find_by_id(weight_item_instance.id).one(&database).await{
            Ok(Some(pear)) => {
                let mut pear: weight_item::ActiveModel = pear.into();
                let profit = pear.profit.clone().unwrap();
                let parent_id = pear.parent_id.clone().unwrap();
                let total = pear.kg_weight.clone().unwrap();
                let price = pear.price.clone().unwrap();
                grant_total += (weight_item_instance.kg_weight * price as f64) as i32;

                history_weight_items.push(ParentWeightItemBody{
                    parent_id: parent_id,
                    kg_weight: weight_item_instance.kg_weight,
                    sell_price: price
                });

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
                            .query(select_query, (parent_id, business_id, ItemType::WeightItem.get_value(), current_date))
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

                        let select_query = "SELECT profit FROM statistics.profits WHERE business_id = ? AND date = ?";
                        let result = scylla
                            .query(select_query, (business_id, current_date))
                            .await.unwrap();

                        match result.rows.expect("failed to get rows").into_typed::<(i32, )>().next() {
                            Some(row) => {
                                let (current_profit, ) = row.expect("couldn't parse");
                                let new_profit = current_profit + (weight_item_instance.kg_weight * profit as f64) as i32;
                                let update_query = "UPDATE statistics.profits SET profit = ? WHERE business_id = ? AND date = ?";
                                scylla
                                    .query(update_query, (new_profit, business_id, current_date))
                                    .await.unwrap();
                            },
                            None => {
                                let insert = "INSERT INTO statistics.profits (business_id, profit, date) VALUES (?, ?, ?);";

                                scylla.query(
                                    insert,
                                    (business_id, (weight_item_instance.kg_weight * profit as f64) as i32, current_date)
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

    for no_code_product_instance in &sell.no_code_products{
        match NoCodeProduct::find_by_id(no_code_product_instance.id).one(&database).await{
            Ok(Some(pear)) => {
                let mut pear: no_code_product::ActiveModel = pear.into();

                let total = pear.quantity.clone().unwrap();
                let profit = pear.profit.clone().unwrap();
                let parent_id = pear.parent_id.clone().unwrap();
                let price = pear.price.clone().unwrap();
                grant_total += no_code_product_instance.quantity * price;

                history_no_code_products.push(ParentNoCodeProductBody{
                    parent_id: parent_id,
                    quantity: no_code_product_instance.quantity,
                    sell_price: price
                });

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
                        let select_query = "SELECT quantity FROM statistics.products WHERE parent_id = ? AND business_id = ? AND item_type = ? AND date = ?";
                        let result = scylla
                            .query(select_query, (parent_id, business_id, ItemType::NoCodeProduct.get_value(), current_date))
                            .await.unwrap();

                        match result.rows.expect("failed to get rows").into_typed::<(i32, )>().next() {
                            Some(row) => {
                                let (current_quantity, ) = row.expect("couldn't parse");
                                let new_quantity = current_quantity + no_code_product_instance.quantity;
                                let update_query = "UPDATE statistics.products SET quantity = ? WHERE parent_id = ? AND business_id = ? AND item_type = ? AND date = ?";
                                scylla
                                    .query(update_query, (new_quantity, parent_id, business_id, ItemType::NoCodeProduct.get_value(), current_date))
                                    .await.unwrap();
                            },
                            None => {
                                let insert = "INSERT INTO statistics.products (parent_id, quantity, profit, business_id, date, item_type) VALUES (?, ?, ?, ?, ?, ?);";

                                scylla.query(
                                    insert,
                                    (parent_id, no_code_product_instance.quantity, no_code_product_instance.quantity * profit, business_id, current_date, ItemType::NoCodeProduct.get_value())
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

    if let Some(user_data) = sell.debt_user {
        match Rent::find_by_id(user_data.id).one(&database).await{
            Ok(Some(pear)) => {
                let mut pear: rent::ActiveModel = pear.into();
                let total = pear.price.clone().unwrap();
                let new_debt = total - user_data.paid_price + grant_total;
                pear.price = Set(new_debt);
                if let Err(err) = pear.update(&database).await {
                    println!("{:?}", err);
                    return internal_server_error();
                }

                let new_rent_history = rent_history::ActiveModel {
                    grand_total: Set(grant_total),
                    paid_amount: Set(user_data.paid_price),
                    products: Set(json!(RentHistoryProducts{
                        products: history_products,
                        weight_items: history_weight_items,
                        no_code_products: history_no_code_products,

                    })),
                    buy_date: Set(DateTimeWithTimeZone::from(chrono::Utc::now())),
                    ..Default::default()
                };

                match new_rent_history.save(&database).await {
                    Ok(instance) => {
                        info!("{:?}", instance);
                    },
                    Err(error) => {
                        error!("Unable to create {:?}. Original error was {}", 1, error);
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
        }
    }

    default_ok()
}

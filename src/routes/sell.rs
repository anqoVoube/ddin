use std::sync::Arc;
use axum::{Extension, Json, debug_handler};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use log::{error, info};
use mongodb::Database;

use rust_decimal_macros::dec;
use scylla::{IntoTypedRows, Session};
use scylla::_macro_internal::Value;
use sea_orm::{ActiveModelTrait, Condition, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::{DateTimeUtc, DateTimeWithTimeZone};
use serde_json::json;
use crate::core::auth::middleware::{Auth, CustomHeader};
use crate::database::prelude::{NoCodeProduct, NoCodeProductSearch, ProductStatistics, Rent, WeightItem, WeightItemSearch};
use crate::database::product::Entity as Product;
use crate::database::{no_code_product, no_code_product_search, product, product_statistics, profit_statistics, rent, rent_history, weight_item, weight_item_search};
use crate::database::prelude::ProfitStatistics;
use crate::routes::utils::{not_found, bad_request, internal_server_error, default_created, default_ok};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
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
    pub parent_id: i32,
    pub quantity: i32,
    pub sell_price: f64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoCodeProductBody {
    id: i32,
    quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentNoCodeProductBody {
    pub parent_id: i32,
    pub quantity: i32,
    pub sell_price: f64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightItemBody {
    id: i32,
    kg_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentWeightItemBody {
    pub parent_id: i32,
    pub kg_weight: f64,
    pub sell_price: f64
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebtUserBody{
    id: i32,
    paid_price: f64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellBody {
    weight_items: Vec<WeightItemBody>,
    products: Vec<ProductBody>,
    no_code_products: Vec<NoCodeProductBody>,
    debt_user: Option<DebtUserBody>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History{
    pub weight_items: Vec<ParentWeightItemBody>,
    pub products: Vec<ParentProductBody>,
    pub no_code_products: Vec<ParentNoCodeProductBody>,
}

pub enum ItemType{
    Product,
    WeightItem,
    NoCodeProduct
}

pub trait EnumValue{
    fn get_value(&self) -> i16;
    fn from_value(value: i16) -> Self;
}

impl EnumValue for ItemType{
    fn get_value(&self) -> i16 {
        match self {
            ItemType::Product => 1,
            ItemType::WeightItem => 2,
            ItemType::NoCodeProduct => 3
        }
    }

    fn from_value(value: i16) -> Self {
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
    Extension(mongo): Extension<Database>,
    Extension(Auth{user_id}): Extension<Auth>,
    Extension(CustomHeader{business_id}): Extension<CustomHeader>,
    Json(sell): Json<SellBody>
) -> Response {
    println!("{:?}", sell);
    let history_collection = mongo.collection::<History>("sell");
    let mut history_products: Vec<ParentProductBody> = vec!();
    let mut history_weight_items: Vec<ParentWeightItemBody> = vec!();
    let mut history_no_code_products: Vec<ParentNoCodeProductBody> = vec!();
    let mut grant_total = 0f64;
    for product_instance in &sell.products{
        match Product::find_by_id(product_instance.id)
            .one(&database).await{
            Ok(Some(pear)) => {
                let mut pear: product::ActiveModel = pear.into();
                let profit = pear.profit.clone().unwrap();
                let total = pear.quantity.clone().unwrap();
                let parent_id = pear.parent_id.clone().unwrap();
                let price = pear.price.clone().unwrap();
                grant_total += product_instance.quantity as f64 * price;

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
                    // match ProductSearch::find().filter(product_search::Column::ParentId.eq(pear.parent_id)).one(&database).await{
                    //     Ok(Some(pear_search)) => {
                    //         let mut pear_search: product_search::ActiveModel = pear_search.into();
                    //         pear_search.hits = Set(pear_search.hits.unwrap() + 1);
                    //     },
                    //     Ok(None) => {
                    //         return not_found();
                    //     },
                    //     Err(err) => {
                    //         println!("{:?}", err);
                    //         return internal_server_error();
                    //     }
                    // }
                    if let Err(err) = pear.update(&database).await {
                        println!("{:?}", err);
                        return internal_server_error();
                    }
                }

                let current_date = Utc::now().naive_utc().date();
                match ProductStatistics::find().filter(
                    Condition::all()
                        .add(product_statistics::Column::ParentId.eq(parent_id))
                        .add(product_statistics::Column::BusinessId.eq(business_id))
                        .add(product_statistics::Column::ItemType.eq(ItemType::Product.get_value()))
                        .add(product_statistics::Column::Date.eq(current_date))

                ).one(&database).await {
                    Ok(Some(product_stats_instance)) => {
                        let mut product_stats_instance: product_statistics::ActiveModel = product_stats_instance.into();
                        product_stats_instance.quantity = Set(product_stats_instance.quantity.unwrap() + product_instance.quantity);
                        product_stats_instance.profit = Set(product_stats_instance.profit.unwrap() + product_instance.quantity as f64 * profit);
                        product_stats_instance.update(&database).await.unwrap();
                    },
                    Ok(None) => {
                        let product_stats_instance = product_statistics::ActiveModel {
                            parent_id: Set(parent_id),
                            quantity: Set(product_instance.quantity),
                            profit: Set(product_instance.quantity as f64 * profit),
                            business_id: Set(business_id),
                            date: Set(current_date),
                            item_type: Set(ItemType::Product.get_value()),
                            ..Default::default()
                        };

                        product_stats_instance.save(&database).await.unwrap();
                    },
                    Err(e) => {
                        println!("{:?}", e);
                    }
                };

                match ProfitStatistics::find().filter(
                    Condition::all()
                        .add(profit_statistics::Column::BusinessId.eq(business_id))
                        .add(profit_statistics::Column::Date.eq(current_date))
                ).one(&database).await {
                    Ok(Some(profit_stats_instance)) => {
                        let mut profit_stats_instance: profit_statistics::ActiveModel = profit_stats_instance.into();
                        profit_stats_instance.profit = Set(profit_stats_instance.profit.unwrap() + product_instance.quantity as f64 * profit);
                        profit_stats_instance.update(&database).await.unwrap();
                    },
                    Ok(None) => {
                        let profit_stats_instance = profit_statistics::ActiveModel {
                            business_id: Set(business_id),
                            profit: Set(product_instance.quantity as f64 * profit),
                            date: Set(current_date),
                            ..Default::default()
                        };
                        profit_stats_instance.save(&database).await.unwrap();
                    },
                    Err(e) => {
                        println!("{:?}", e);
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

    for weight_item_instance in &sell.weight_items{
        match WeightItem::find_by_id(weight_item_instance.id).one(&database).await{
            Ok(Some(pear)) => {
                let mut pear: weight_item::ActiveModel = pear.into();
                let profit = pear.profit.clone().unwrap();
                let parent_id = pear.parent_id.clone().unwrap();
                let total = pear.kg_weight.clone().unwrap();
                let price = pear.price.clone().unwrap();
                grant_total += weight_item_instance.kg_weight * price;

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
                    match WeightItemSearch::find().filter(weight_item_search::Column::ParentId.eq(pear.parent_id.clone().unwrap())).one(&database).await{
                        Ok(Some(pear_search)) => {
                            let mut pear_search: weight_item_search::ActiveModel = pear_search.into();
                            pear_search.hits = Set(pear_search.hits.unwrap() + 1);
                        },
                        Ok(None) => {
                            return not_found();
                        },
                        Err(err) => {
                            println!("{:?}", err);
                            return internal_server_error();
                        }
                    }
                    if let Err(err) = pear.update(&database).await {
                        println!("{:?}", err);
                        return internal_server_error();
                    }
                }
                let current_date = Utc::now().naive_utc().date();
                match ProductStatistics::find().filter(
                    Condition::all()
                        .add(product_statistics::Column::ParentId.eq(parent_id))
                        .add(product_statistics::Column::BusinessId.eq(business_id))
                        .add(product_statistics::Column::ItemType.eq(ItemType::WeightItem.get_value()))
                        .add(product_statistics::Column::Date.eq(current_date))
                ).one(&database).await {
                    Ok(Some(product_stats_instance)) => {
                        let mut product_stats_instance: product_statistics::ActiveModel = product_stats_instance.into();
                        product_stats_instance.quantity = Set(product_stats_instance.quantity.unwrap()+ (weight_item_instance.kg_weight * 1000f64) as i32);
                        product_stats_instance.profit = Set(product_stats_instance.profit.unwrap() + weight_item_instance.kg_weight * profit);
                        product_stats_instance.update(&database).await.unwrap();
                    },
                    Ok(None) => {
                        let product_stats_instance = product_statistics::ActiveModel {
                            parent_id: Set(parent_id),
                            quantity: Set((weight_item_instance.kg_weight * 1000f64) as i32),
                            profit: Set(weight_item_instance.kg_weight * profit),
                            business_id: Set(business_id),
                            date: Set(current_date),
                            item_type: Set(ItemType::WeightItem.get_value()),
                            ..Default::default()
                        };

                        match product_stats_instance.save(&database).await {
                            Ok(instance) => {
                                println!("Successfully created instance");
                                println!("{:?}", instance);
                            },
                            Err(error) => {
                                error!("Unable to create {:?}. Original error was {}", 1, error);
                                println!("Failed to create instance. Error is {:?}", error)
                            }
                        }
                    },
                    Err(e) => {
                        println!("{:?}", e);
                    }
                };

                match ProfitStatistics::find().filter(
                    Condition::all()
                        .add(profit_statistics::Column::BusinessId.eq(business_id))
                        .add(profit_statistics::Column::Date.eq(current_date))
                ).one(&database).await {
                    Ok(Some(profit_stats_instance)) => {
                        let mut profit_stats_instance: profit_statistics::ActiveModel = profit_stats_instance.into();
                        profit_stats_instance.profit = Set(profit_stats_instance.profit.unwrap() + weight_item_instance.kg_weight * profit);
                        profit_stats_instance.update(&database).await.unwrap();
                    },
                    Ok(None) => {
                        let profit_stats_instance = profit_statistics::ActiveModel {
                            business_id: Set(business_id),
                            profit: Set(weight_item_instance.kg_weight * profit),
                            date: Set(current_date),
                            ..Default::default()
                        };
                        profit_stats_instance.save(&database).await.unwrap();
                    },
                    Err(e) => {
                        println!("{:?}", e);
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

    for no_code_product_instance in &sell.no_code_products{
        match NoCodeProduct::find_by_id(no_code_product_instance.id).one(&database).await{
            Ok(Some(pear)) => {
                let mut pear: no_code_product::ActiveModel = pear.into();

                let total = pear.quantity.clone().unwrap();
                let profit = pear.profit.clone().unwrap();
                let parent_id = pear.parent_id.clone().unwrap();
                let price = pear.price.clone().unwrap();
                grant_total += no_code_product_instance.quantity as f64 * price;

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

                    match NoCodeProductSearch::find().filter(no_code_product_search::Column::ParentId.eq(pear.parent_id.clone().unwrap())).one(&database).await{
                        Ok(Some(pear_search)) => {
                            let mut pear_search: no_code_product_search::ActiveModel = pear_search.into();
                            pear_search.hits = Set(pear_search.hits.unwrap() + 1);
                        },
                        Ok(None) => {
                            return not_found();
                        },
                        Err(err) => {
                            println!("{:?}", err);
                            return internal_server_error();
                        }
                    }
                    if let Err(err) = pear.update(&database).await {
                        println!("{:?}", err);
                        return internal_server_error();
                    }
                }
                let current_date = Utc::now().naive_utc().date();
                match ProductStatistics::find().filter(
                    Condition::all()
                        .add(product_statistics::Column::ParentId.eq(parent_id))
                        .add(product_statistics::Column::BusinessId.eq(business_id))
                        .add(product_statistics::Column::ItemType.eq(ItemType::NoCodeProduct.get_value()))
                        .add(product_statistics::Column::Date.eq(current_date))

                ).one(&database).await {
                    Ok(Some(product_stats_instance)) => {
                        let mut product_stats_instance: product_statistics::ActiveModel = product_stats_instance.into();
                        product_stats_instance.quantity = Set(product_stats_instance.quantity.unwrap() + no_code_product_instance.quantity);
                        product_stats_instance.profit = Set(product_stats_instance.profit.unwrap() + no_code_product_instance.quantity as f64 * profit);
                        product_stats_instance.update(&database).await.unwrap();
                    },
                    Ok(None) => {
                        let product_stats_instance = product_statistics::ActiveModel {
                            parent_id: Set(parent_id),
                            quantity: Set(no_code_product_instance.quantity),
                            profit: Set(no_code_product_instance.quantity as f64 * profit),
                            business_id: Set(business_id),
                            date: Set(current_date),
                            item_type: Set(ItemType::NoCodeProduct.get_value()),
                            ..Default::default()
                        };


                        product_stats_instance.save(&database).await.unwrap();
                    },
                    Err(e) => {
                        println!("{:?}", e);
                    }
                };

                match ProfitStatistics::find().filter(
                    Condition::all()
                        .add(profit_statistics::Column::BusinessId.eq(business_id))
                        .add(profit_statistics::Column::Date.eq(current_date))
                ).one(&database).await {
                    Ok(Some(profit_stats_instance)) => {
                        let mut profit_stats_instance: profit_statistics::ActiveModel = profit_stats_instance.into();
                        profit_stats_instance.profit = Set(profit_stats_instance.profit.unwrap() + no_code_product_instance.quantity as f64 * profit);
                        profit_stats_instance.update(&database).await.unwrap();
                    },
                    Ok(None) => {
                        let profit_stats_instance = profit_statistics::ActiveModel {
                            business_id: Set(business_id),
                            profit: Set(no_code_product_instance.quantity as f64 * profit),
                            date: Set(current_date),
                            ..Default::default()
                        };
                        profit_stats_instance.save(&database).await.unwrap();
                    },
                    Err(e) => {
                        println!("{:?}", e);
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

    let history = History{
        products: history_products,
        weight_items: history_weight_items,
        no_code_products: history_no_code_products,
    };

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
                    products: Set(
                        json!(
                            history
                        )
                    ),
                    buy_date: Set(DateTimeWithTimeZone::from(chrono::Utc::now())),
                    rent_user_id: Set(user_data.id),
                    ..Default::default()
                };

                match new_rent_history.save(&database).await {
                    Ok(instance) => {
                        println!("Successfully created instance");
                        info!("{:?}", instance);
                    },
                    Err(error) => {
                        error!("Unable to create {:?}. Original error was {}", 1, error);
                        println!("Failed to create instance. Error is {:?}", error)
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

    history_collection.insert_one(
        history,
        None
    ).await.expect("Failed to insert history");

    default_ok()
}

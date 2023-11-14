use http::StatusCode;
use log::{error, info};
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::Serialize;
use crate::database::parent_product::Model as ParentProductModel;
use crate::database::parent_no_code_product::Model as ParentNoCodeProductModel;
use crate::database::parent_weight_item::Model as ParentWeightItemModel;
use crate::database::prelude::ParentProduct;
use crate::database::prelude::ParentWeightItem;
use crate::database::prelude::ParentNoCodeProduct;
use crate::routes::sell::ItemType;
use crate::routes::utils::get_parent::Stats::Quantity;


pub enum Parent{
    Product(ParentProductModel),
    WeightItem(ParentWeightItemModel),
    NoCodeProduct(ParentNoCodeProductModel)
}

#[derive(Serialize, Debug)]
pub struct BestQuantity{
    pub title: String,
    pub main_image: Option<String>,
    pub overall_quantity: i32
}

#[derive(Serialize, Debug)]
pub struct BestProfit{
    pub title: String,
    pub main_image: Option<String>,
    pub overall_profit: i32
}

pub enum Stats{
    Quantity(BestQuantity),
    Profit(BestProfit)
}

pub enum StatsType{
    Quantity,
    Profit
}

pub trait ParentGetter{
    fn fetch_data(&self, stats_type: StatsType, overall: i32) -> Stats;
}

impl ParentGetter for Parent{
    fn fetch_data(&self, stats_type: StatsType, overall: i32) -> Stats {
        match self{
            Parent::Product(model) => {
                match stats_type{
                    StatsType::Quantity =>
                        Stats::Quantity(BestQuantity{
                        title: model.title.clone(),
                        main_image: model.main_image.clone(),
                        overall_quantity: overall
                    }),
                    StatsType::Profit =>
                        Stats::Profit(BestProfit{
                        title: model.title.clone(),
                        main_image: model.main_image.clone(),
                        overall_profit: overall
                    })
                }
            },
            Parent::WeightItem(model) => {
                match stats_type{
                    StatsType::Quantity => Stats::Quantity(BestQuantity{
                        title: model.title.clone(),
                        main_image: model.main_image.clone(),
                        overall_quantity: overall
                    }),
                    StatsType::Profit => Stats::Profit(BestProfit{
                        title: model.title.clone(),
                        main_image: model.main_image.clone(),
                        overall_profit: overall
                    })
                }
            },
            Parent::NoCodeProduct(model) => {
                match stats_type{
                    StatsType::Quantity => Stats::Quantity(BestQuantity{
                        title: model.title.clone(),
                        main_image: model.main_image.clone(),
                        overall_quantity: overall
                    }),
                    StatsType::Profit => Stats::Profit(BestProfit{
                        title: model.title.clone(),
                        main_image: model.main_image.clone(),
                        overall_profit: overall
                    })
                }
            }
        }
    }
}

pub async fn get_parent_by_id(database: &DatabaseConnection, id: i32, item_type: ItemType) -> Result<Parent, StatusCode> {
    match item_type{
        ItemType::Product => {
            let parent = ParentProduct::find_by_id(id).one(database).await
                .map_err(|_error| {error!("Couldn't fetch parent_product with id: {}", id); StatusCode::INTERNAL_SERVER_ERROR})?;

            if let Some(value) = parent{
                Ok(Parent::Product(value))
            }
            else{
                info!("Not found parent_product with id: {}", &id);
                Err(StatusCode::NOT_FOUND)
            }
        },
        ItemType::WeightItem => {
            let parent = ParentWeightItem::find_by_id(id).one(database).await
                .map_err(|_error| {error!("Couldn't fetch parent_product with id: {}", id); StatusCode::INTERNAL_SERVER_ERROR})?;

            if let Some(value) = parent{
                Ok(Parent::WeightItem(value))
            }
            else{
                info!("Not found parent_product with id: {}", &id);
                Err(StatusCode::NOT_FOUND)
            }
        },
        ItemType::NoCodeProduct => {
            let parent = ParentNoCodeProduct::find_by_id(id).one(database).await
                .map_err(|_error| {error!("Couldn't fetch parent_product with id: {}", id); StatusCode::INTERNAL_SERVER_ERROR})?;

            if let Some(value) = parent{
                Ok(Parent::NoCodeProduct(value))
            }
            else{
                info!("Not found parent_product with id: {}", &id);
                Err(StatusCode::NOT_FOUND)
            }
        }
    }
}
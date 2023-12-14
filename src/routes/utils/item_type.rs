use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ItemType {
    #[serde(rename = "1")]
    ParentProduct,
    #[serde(rename = "2")]
    ParentWeightItem,
    #[serde(rename = "3")]
    ParentNoCodeProduct,
}
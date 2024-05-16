//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "parent_weight_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub code: Option<String>,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub main_image: Option<String>,
    pub images: Vec<String>,
    pub business_id: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::business::Entity",
        from = "Column::BusinessId",
        to = "super::business::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Business,
    #[sea_orm(has_many = "super::weight_item::Entity")]
    WeightItem,
    #[sea_orm(has_many = "super::weight_item_search::Entity")]
    WeightItemSearch,
}

impl Related<super::business::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Business.def()
    }
}

impl Related<super::weight_item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WeightItem.def()
    }
}

impl Related<super::weight_item_search::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WeightItemSearch.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

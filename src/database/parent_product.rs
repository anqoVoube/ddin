//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "parent_product")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub code: String,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub main_image: Option<String>,
    pub images: Vec<String>,
    pub business_id: Option<i32>,
    pub is_verified: bool,
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
    #[sea_orm(has_many = "super::product::Entity")]
    Product,
}

impl Related<super::business::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Business.def()
    }
}

impl Related<super::product::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Product.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

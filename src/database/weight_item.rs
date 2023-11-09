//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "weight_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub price: i32,
    #[sea_orm(column_type = "Double")]
    pub kg_weight: f64,
    pub parent_id: i32,
    pub expiration_date: Option<Date>,
    pub business_id: i32,
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
    #[sea_orm(
        belongs_to = "super::parent_weight_item::Entity",
        from = "Column::ParentId",
        to = "super::parent_weight_item::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    ParentWeightItem,
}

impl Related<super::business::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Business.def()
    }
}

impl Related<super::parent_weight_item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ParentWeightItem.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

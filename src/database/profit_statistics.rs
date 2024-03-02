//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "profit_statistics")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub date: Date,
    pub business_id: i32,
    #[sea_orm(column_type = "Double")]
    pub profit: f64,
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
}

impl Related<super::business::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Business.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
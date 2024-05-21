//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "no_code_product_search")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub hits: i32,
    pub parent_id: i32,
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
        belongs_to = "super::parent_no_code_product::Entity",
        from = "Column::ParentId",
        to = "super::parent_no_code_product::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    ParentNoCodeProduct,
}

impl Related<super::business::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Business.def()
    }
}

impl Related<super::parent_no_code_product::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ParentNoCodeProduct.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

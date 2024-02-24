//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "services_service_categories")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub service_id: i64,
    pub category_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::categories_category::Entity",
        from = "Column::CategoryId",
        to = "super::categories_category::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    CategoriesCategory,
    #[sea_orm(
        belongs_to = "super::services_service::Entity",
        from = "Column::ServiceId",
        to = "super::services_service::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    ServicesService,
}

impl Related<super::categories_category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CategoriesCategory.def()
    }
}

impl Related<super::services_service::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ServicesService.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

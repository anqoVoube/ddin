//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "business")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub location: Vec<Decimal>,
    pub works_from: Time,
    pub works_until: Time,
    pub is_closed: bool,
    pub owner_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::product::Entity")]
    Product,
    #[sea_orm(has_many = "super::rent::Entity")]
    Rent,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::OwnerId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    User,
    #[sea_orm(has_many = "super::weight_item::Entity")]
    WeightItem,
}

impl Related<super::product::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Product.def()
    }
}

impl Related<super::rent::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rent.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::weight_item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WeightItem.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

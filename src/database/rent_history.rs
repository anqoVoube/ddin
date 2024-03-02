//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rent_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "JsonBinary")]
    pub products: Json,
    pub buy_date: DateTimeWithTimeZone,
    pub rent_user_id: i32,
    #[sea_orm(column_type = "Double")]
    pub grand_total: f64,
    #[sea_orm(column_type = "Double")]
    pub paid_amount: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::rent::Entity",
        from = "Column::RentUserId",
        to = "super::rent::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Rent,
}

impl Related<super::rent::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rent.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

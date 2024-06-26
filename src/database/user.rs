//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    #[sea_orm(unique)]
    pub phone_number: String,
    pub is_verified: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::business::Entity")]
    Business,
    #[sea_orm(has_one = "super::telegram_user::Entity")]
    TelegramUser,
    #[sea_orm(has_one = "super::verification::Entity")]
    Verification,
}

impl Related<super::business::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Business.def()
    }
}

impl Related<super::telegram_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TelegramUser.def()
    }
}

impl Related<super::verification::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Verification.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

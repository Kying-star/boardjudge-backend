//! SeaORM Entity. Generated by sea-orm-codegen 0.8.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "contest")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "Custom(\"uuid\".to_owned())"
    )]
    pub id: String,
    pub nick: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub start: DateTime,
    pub end: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::privilege::Entity")]
    Privilege,
    #[sea_orm(has_many = "super::problem::Entity")]
    Problem,
}

impl Related<super::privilege::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Privilege.def()
    }
}

impl Related<super::problem::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Problem.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
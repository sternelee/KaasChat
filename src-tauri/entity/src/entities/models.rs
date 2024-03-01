//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.14
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use async_openai::{Client, config::Config};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Deserialize, Serialize)]
#[sea_orm(table_name = "models")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub provider: String,
    pub config: Json,
    #[serde(skip_deserializing)]
    pub created_at: DateTimeLocal,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(skip_deserializing)]
    pub updated_at: Option<DateTimeLocal>,
    #[serde(skip_deserializing)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTimeLocal>
}

impl Model {
    pub fn get_config<C: Config>(&self) -> Option<C> {
        None
    }
    pub fn get_client<C: Config>(&self) -> Option<Client<C>> {
        None
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::conversations::Entity")]
    Conversations
}

impl Related<super::conversations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Conversations.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

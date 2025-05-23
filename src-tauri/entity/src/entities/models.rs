//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.14
use sea_orm::{entity::prelude::*, FromQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Providers {
    Azure,
    OpenAI,
    Claude,
    Ollama,
    Openrouter,
    Deepseek,
    Xai,
    Google,
    CUSTOM,
    Unknown,
}

impl From<&str> for Providers {
    fn from(value: &str) -> Self {
        match value {
            "Azure" => Providers::Azure,
            "OpenAI" => Providers::OpenAI,
            "Claude" => Providers::Claude,
            "Ollama" => Providers::Ollama,
            "Openrouter" => Providers::Openrouter,
            "Deepseek" => Providers::Deepseek,
            "Xai" => Providers::Xai,
            "Google" => Providers::Google,
            "CUSTOM" => Providers::CUSTOM,
            _ => Providers::Unknown,
        }
    }
}

impl From<String> for Providers {
    fn from(value: String) -> Self {
        Providers::from(value.as_str())
    }
}

impl From<&String> for Providers {
    fn from(value: &String) -> Self {
        Providers::from(value.as_str())
    }
}

impl Into<String> for Providers {
    fn into(self) -> String {
        match self {
            Providers::Azure => "Azure".to_owned(),
            Providers::OpenAI => "OpenAI".to_owned(),
            Providers::Claude => "Claude".to_owned(),
            Providers::Ollama => "Ollama".to_owned(),
            Providers::Openrouter => "Openrouter".to_owned(),
            Providers::Deepseek => "Deepseek".to_owned(),
            Providers::Xai => "Xai".to_owned(),
            Providers::Google => "Google".to_owned(),
            Providers::CUSTOM => "CUSTOM".to_owned(),
            _ => "Unknown".to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Deserialize, Serialize)]
#[sea_orm(table_name = "models")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub alias: String,
    pub provider: String,
    pub config: String,
    #[serde(skip_deserializing)]
    pub created_at: Option<DateTimeLocal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(skip_deserializing)]
    pub updated_at: Option<DateTimeLocal>,
    #[serde(skip_deserializing)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTimeLocal>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::conversations::Entity")]
    Conversations,
}

impl Related<super::conversations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Conversations.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, FromQueryResult, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericConfig {
    pub provider: String,
    pub config: String,
}

#[derive(DeriveIntoActiveModel, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewModel {
    pub alias: String,
    pub provider: String,
    pub config: String,
}

//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.3

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "user_account"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq, Serialize, Deserialize)]
pub struct Model {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub created_at: DateTimeWithTimeZone,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    Username,
    Password,
    CreatedAt,
    DeletedAt,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = Uuid;
    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Project,
    ProjectData,
    ProjectDataImage,
    ProjectParticipant,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Uuid.def(),
            Self::Username => ColumnType::Text.def(),
            Self::Password => ColumnType::Text.def(),
            Self::CreatedAt => ColumnType::TimestampWithTimeZone.def(),
            Self::DeletedAt => ColumnType::TimestampWithTimeZone.def().null(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Project => Entity::has_many(super::project::Entity).into(),
            Self::ProjectData => Entity::has_many(super::project_data::Entity).into(),
            Self::ProjectDataImage => Entity::has_many(super::project_data_image::Entity).into(),
            Self::ProjectParticipant => Entity::has_many(super::project_participant::Entity).into(),
        }
    }
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::project_data::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProjectData.def()
    }
}

impl Related<super::project_data_image::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProjectDataImage.def()
    }
}

impl Related<super::project_participant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProjectParticipant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "competitions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub competition_id: Uuid,
    pub competition_type: i32,
    pub target_fish_id: i32,
    pub start_time: DateTimeWithTimeZone,
    pub end_time: DateTimeWithTimeZone,
    pub reward_currency: String,
    pub prize_pool: Vec<i32>,
    pub created_at: DateTimeWithTimeZone,
    pub status: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::competition_results::Entity")]
    CompetitionResults,
}

impl Related<super::competition_results::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CompetitionResults.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

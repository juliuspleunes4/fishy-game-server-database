use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "competition_results")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub result_id: Uuid,
    pub competition_id: Uuid,
    pub player_id: Uuid,
    pub score: i32,
    pub last_updated: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::competitions::Entity",
        from = "Column::CompetitionId",
        to = "super::competitions::Column::CompetitionId"
    )]
    Competition,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::PlayerId",
        to = "super::users::Column::UserId"
    )]
    User,
}

impl Related<super::competitions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Competition.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

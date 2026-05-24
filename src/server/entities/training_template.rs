use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "training_templates")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub club_id: i32,
    pub group_id: i32,
    pub title: String,
    pub description: String,
    pub number_of_throws: Option<i32>,
    pub target_score: Option<i32>,
    pub standing_pins_mask: Option<i32>,
    pub clear_pins: Option<bool>,
    pub created_by_user_id: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::club::Entity",
        from = "Column::ClubId",
        to = "super::club::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Club,
    #[sea_orm(
        belongs_to = "super::club_group::Entity",
        from = "Column::GroupId",
        to = "super::club_group::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Group,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::CreatedByUserId",
        to = "super::user::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    CreatedByUser,
    #[sea_orm(has_many = "super::training_plan_template::Entity")]
    TrainingPlanTemplate,
}

impl Related<super::club::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Club.def()
    }
}

impl Related<super::club_group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Group.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreatedByUser.def()
    }
}

impl Related<super::training_plan_template::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TrainingPlanTemplate.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

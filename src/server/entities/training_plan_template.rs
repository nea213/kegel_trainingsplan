use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "training_plan_templates")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub training_plan_id: i32,
    pub training_template_id: i32,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::training_plan::Entity",
        from = "Column::TrainingPlanId",
        to = "super::training_plan::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    TrainingPlan,
    #[sea_orm(
        belongs_to = "super::training_template::Entity",
        from = "Column::TrainingTemplateId",
        to = "super::training_template::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    TrainingTemplate,
}

impl Related<super::training_plan::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TrainingPlan.def()
    }
}

impl Related<super::training_template::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TrainingTemplate.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

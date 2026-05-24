use crate::{
    server::{
        db,
        entities::{club_group, group_trainer, training_plan, training_plan_template, training_template, user},
        permissions,
        training_plan_templates::{
            create_training_plan_model, create_training_plan_template_model,
            create_training_template_model, standing_pins_from_mask, update_training_plan_model,
            update_training_template_model, TrainingPlanDraft, TrainingTemplateDraft,
        },
    },
    training_management::{
        CreateTrainingPlanInput, CreateTrainingTemplateInput, LinkedTrainingTemplateSummary,
        TrainingPlanSummary, TrainingTemplateSummary, UpdateTrainingPlanInput,
        UpdateTrainingTemplateInput,
    },
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, TransactionTrait,
};
use std::collections::{BTreeMap, BTreeSet};

pub async fn list_templates(group_id: i32) -> Result<Vec<TrainingTemplateSummary>, String> {
    permissions::require_group_manager(group_id).await?;
    let db = db::connection().await.map_err(db_error)?;

    training_template::Entity::find()
        .filter(training_template::Column::GroupId.eq(group_id))
        .order_by_desc(training_template::Column::UpdatedAt)
        .order_by_asc(training_template::Column::Title)
        .all(db)
        .await
        .map_err(db_error)?
        .into_iter()
        .map(template_summary)
        .collect()
}

pub async fn create_template(
    input: CreateTrainingTemplateInput,
) -> Result<TrainingTemplateSummary, String> {
    let user = permissions::require_group_manager(input.group_id).await?;
    validate_group_belongs_to_club(input.group_id, input.club_id).await?;
    let db = db::connection().await.map_err(db_error)?;

    create_training_template_model(template_draft(&input), user.id)?
        .insert(db)
        .await
        .map_err(db_error)
        .and_then(template_summary)
}

pub async fn update_template(
    input: UpdateTrainingTemplateInput,
) -> Result<TrainingTemplateSummary, String> {
    let _ = permissions::require_group_manager(input.group_id).await?;
    validate_group_belongs_to_club(input.group_id, input.club_id).await?;
    let db = db::connection().await.map_err(db_error)?;

    let existing = training_template::Entity::find_by_id(input.template_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Die Vorlage wurde nicht gefunden.".to_string())?;

    permissions::require_group_manager(existing.group_id).await?;

    let mut model = existing.into_active_model();
    update_training_template_model(&mut model, template_draft_from_update(&input))?;

    model
        .update(db)
        .await
        .map_err(db_error)
        .and_then(template_summary)
}

pub async fn delete_template(template_id: i32) -> Result<(), String> {
    let db = db::connection().await.map_err(db_error)?;
    let existing = training_template::Entity::find_by_id(template_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Die Vorlage wurde nicht gefunden.".to_string())?;

    permissions::require_group_manager(existing.group_id).await?;

    let usage_count = training_plan_template::Entity::find()
        .filter(training_plan_template::Column::TrainingTemplateId.eq(template_id))
        .count(db)
        .await
        .map_err(db_error)?;

    if usage_count > 0 {
        return Err(
            "Die Vorlage wird bereits in Trainingstagen verwendet und kann deshalb nicht gelöscht werden."
                .to_string(),
        );
    }

    training_template::Entity::delete_by_id(template_id)
        .exec(db)
        .await
        .map_err(db_error)?;

    Ok(())
}

pub async fn list_plans(group_id: i32) -> Result<Vec<TrainingPlanSummary>, String> {
    permissions::require_group_manager(group_id).await?;
    let db = db::connection().await.map_err(db_error)?;
    plan_summaries_for_group(db, group_id).await
}

pub async fn create_plan(input: CreateTrainingPlanInput) -> Result<TrainingPlanSummary, String> {
    let user = permissions::require_group_manager(input.group_id).await?;
    validate_group_belongs_to_club(input.group_id, input.club_id).await?;
    let template_ids = normalize_template_ids(&input.template_ids);
    let db = db::connection().await.map_err(db_error)?;

    validate_trainer_assignment(db, input.group_id, input.trainer_user_id).await?;
    validate_templates_for_group(db, input.group_id, &template_ids).await?;

    let transaction = db.begin().await.map_err(db_error)?;
    let plan = create_training_plan_model(plan_draft(&input), user.id)?
        .insert(&transaction)
        .await
        .map_err(db_error)?;
    replace_plan_templates(&transaction, plan.id, &template_ids).await?;
    transaction.commit().await.map_err(db_error)?;

    load_plan_summary(db, plan.id).await
}

pub async fn update_plan(input: UpdateTrainingPlanInput) -> Result<TrainingPlanSummary, String> {
    let _ = permissions::require_group_manager(input.group_id).await?;
    validate_group_belongs_to_club(input.group_id, input.club_id).await?;
    let template_ids = normalize_template_ids(&input.template_ids);
    let db = db::connection().await.map_err(db_error)?;

    let existing = training_plan::Entity::find_by_id(input.plan_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Der Trainingstag wurde nicht gefunden.".to_string())?;

    permissions::require_group_manager(existing.group_id).await?;
    validate_trainer_assignment(db, input.group_id, input.trainer_user_id).await?;
    validate_templates_for_group(db, input.group_id, &template_ids).await?;

    let transaction = db.begin().await.map_err(db_error)?;
    let mut model = existing.into_active_model();
    update_training_plan_model(&mut model, plan_draft_from_update(&input))?;
    let updated = model.update(&transaction).await.map_err(db_error)?;
    replace_plan_templates(&transaction, updated.id, &template_ids).await?;
    transaction.commit().await.map_err(db_error)?;

    load_plan_summary(db, updated.id).await
}

pub async fn delete_plan(plan_id: i32) -> Result<(), String> {
    let db = db::connection().await.map_err(db_error)?;
    let existing = training_plan::Entity::find_by_id(plan_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Der Trainingstag wurde nicht gefunden.".to_string())?;

    permissions::require_group_manager(existing.group_id).await?;

    training_plan::Entity::delete_by_id(plan_id)
        .exec(db)
        .await
        .map_err(db_error)?;

    Ok(())
}

async fn validate_group_belongs_to_club(group_id: i32, club_id: i32) -> Result<(), String> {
    let db = db::connection().await.map_err(db_error)?;
    let group = club_group::Entity::find_by_id(group_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Die Gruppe wurde nicht gefunden.".to_string())?;

    if group.club_id != club_id {
        return Err("Die ausgewählte Gruppe gehört nicht zum angegebenen Verein.".to_string());
    }

    Ok(())
}

async fn validate_trainer_assignment(
    db: &DatabaseConnection,
    group_id: i32,
    trainer_user_id: Option<i32>,
) -> Result<(), String> {
    let Some(trainer_user_id) = trainer_user_id else {
        return Ok(());
    };

    let assigned = group_trainer::Entity::find()
        .filter(group_trainer::Column::GroupId.eq(group_id))
        .filter(group_trainer::Column::UserId.eq(trainer_user_id))
        .one(db)
        .await
        .map_err(db_error)?;

    if assigned.is_none() {
        return Err(
            "Der ausgewählte Trainer ist dieser Gruppe nicht als Trainer zugewiesen."
                .to_string(),
        );
    }

    Ok(())
}

async fn validate_templates_for_group(
    db: &DatabaseConnection,
    group_id: i32,
    template_ids: &[i32],
) -> Result<(), String> {
    if template_ids.is_empty() {
        return Ok(());
    }

    let templates = training_template::Entity::find()
        .filter(training_template::Column::Id.is_in(template_ids.iter().copied()))
        .all(db)
        .await
        .map_err(db_error)?;

    if templates.len() != template_ids.len() {
        return Err("Mindestens eine ausgewählte Vorlage wurde nicht gefunden.".to_string());
    }

    if templates.iter().any(|template| template.group_id != group_id) {
        return Err(
            "Alle ausgewählten Vorlagen müssen zur aktuell gewählten Gruppe gehören.".to_string(),
        );
    }

    Ok(())
}

async fn replace_plan_templates<C: ConnectionTrait>(
    db: &C,
    plan_id: i32,
    template_ids: &[i32],
) -> Result<(), String> {
    training_plan_template::Entity::delete_many()
        .filter(training_plan_template::Column::TrainingPlanId.eq(plan_id))
        .exec(db)
        .await
        .map_err(db_error)?;

    for template_id in template_ids {
        create_training_plan_template_model(plan_id, *template_id)
            .insert(db)
            .await
            .map_err(db_error)?;
    }

    Ok(())
}

async fn load_plan_summary(
    db: &DatabaseConnection,
    plan_id: i32,
) -> Result<TrainingPlanSummary, String> {
    let plan = training_plan::Entity::find_by_id(plan_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Der Trainingstag wurde nicht gefunden.".to_string())?;

    let summaries = plan_summaries_for_group(db, plan.group_id).await?;
    summaries
        .into_iter()
        .find(|summary| summary.id == plan_id)
        .ok_or_else(|| "Der Trainingstag konnte nach dem Speichern nicht geladen werden.".to_string())
}

async fn plan_summaries_for_group(
    db: &DatabaseConnection,
    group_id: i32,
) -> Result<Vec<TrainingPlanSummary>, String> {
    let plans = training_plan::Entity::find()
        .filter(training_plan::Column::GroupId.eq(group_id))
        .order_by_desc(training_plan::Column::Day)
        .order_by_asc(training_plan::Column::Title)
        .all(db)
        .await
        .map_err(db_error)?;

    if plans.is_empty() {
        return Ok(Vec::new());
    }

    let plan_ids = plans.iter().map(|plan| plan.id).collect::<Vec<_>>();
    let trainer_ids = plans
        .iter()
        .filter_map(|plan| plan.trainer_user_id)
        .collect::<BTreeSet<_>>();

    let trainers = if trainer_ids.is_empty() {
        BTreeMap::new()
    } else {
        user::Entity::find()
            .filter(user::Column::Id.is_in(trainer_ids.iter().copied()))
            .all(db)
            .await
            .map_err(db_error)?
            .into_iter()
            .map(|trainer| (trainer.id, trainer.username))
            .collect::<BTreeMap<_, _>>()
    };

    let plan_links = training_plan_template::Entity::find()
        .filter(training_plan_template::Column::TrainingPlanId.is_in(plan_ids.clone()))
        .all(db)
        .await
        .map_err(db_error)?;

    let template_ids = plan_links
        .iter()
        .map(|link| link.training_template_id)
        .collect::<BTreeSet<_>>();
    let templates = if template_ids.is_empty() {
        BTreeMap::new()
    } else {
        training_template::Entity::find()
            .filter(training_template::Column::Id.is_in(template_ids.iter().copied()))
            .all(db)
            .await
            .map_err(db_error)?
            .into_iter()
            .map(|template| {
                (
                    template.id,
                    LinkedTrainingTemplateSummary {
                        id: template.id,
                        title: template.title,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>()
    };

    let mut templates_by_plan = BTreeMap::<i32, Vec<LinkedTrainingTemplateSummary>>::new();
    for link in plan_links {
        if let Some(template) = templates.get(&link.training_template_id) {
            templates_by_plan
                .entry(link.training_plan_id)
                .or_default()
                .push(template.clone());
        }
    }

    for linked_templates in templates_by_plan.values_mut() {
        linked_templates.sort_by(|left, right| left.title.cmp(&right.title));
    }

    Ok(plans
        .into_iter()
        .map(|plan| TrainingPlanSummary {
            id: plan.id,
            club_id: plan.club_id,
            group_id: plan.group_id,
            title: plan.title,
            day: plan.day,
            note: plan.note,
            trainer_user_id: plan.trainer_user_id,
            trainer_username: plan
                .trainer_user_id
                .and_then(|trainer_id| trainers.get(&trainer_id).cloned()),
            created_by_user_id: plan.created_by_user_id,
            created_at: plan.created_at,
            updated_at: plan.updated_at,
            templates: templates_by_plan.remove(&plan.id).unwrap_or_default(),
        })
        .collect())
}

fn normalize_template_ids(template_ids: &[i32]) -> Vec<i32> {
    template_ids
        .iter()
        .copied()
        .filter(|template_id| *template_id > 0)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn template_draft(input: &CreateTrainingTemplateInput) -> TrainingTemplateDraft {
    TrainingTemplateDraft {
        club_id: input.club_id,
        group_id: input.group_id,
        title: input.title.clone(),
        description: input.description.clone(),
        number_of_throws: input.number_of_throws,
        target_score: input.target_score,
        standing_pins: input.standing_pins.clone(),
        clear_pins: input.clear_pins,
    }
}

fn template_draft_from_update(input: &UpdateTrainingTemplateInput) -> TrainingTemplateDraft {
    TrainingTemplateDraft {
        club_id: input.club_id,
        group_id: input.group_id,
        title: input.title.clone(),
        description: input.description.clone(),
        number_of_throws: input.number_of_throws,
        target_score: input.target_score,
        standing_pins: input.standing_pins.clone(),
        clear_pins: input.clear_pins,
    }
}

fn plan_draft(input: &CreateTrainingPlanInput) -> TrainingPlanDraft {
    TrainingPlanDraft {
        club_id: input.club_id,
        group_id: input.group_id,
        title: input.title.clone(),
        day: input.day.clone(),
        note: input.note.clone(),
        trainer_user_id: input.trainer_user_id,
    }
}

fn plan_draft_from_update(input: &UpdateTrainingPlanInput) -> TrainingPlanDraft {
    TrainingPlanDraft {
        club_id: input.club_id,
        group_id: input.group_id,
        title: input.title.clone(),
        day: input.day.clone(),
        note: input.note.clone(),
        trainer_user_id: input.trainer_user_id,
    }
}

fn template_summary(model: training_template::Model) -> Result<TrainingTemplateSummary, String> {
    Ok(TrainingTemplateSummary {
        id: model.id,
        club_id: model.club_id,
        group_id: model.group_id,
        title: model.title,
        description: model.description,
        number_of_throws: model.number_of_throws,
        target_score: model.target_score,
        standing_pins: standing_pins_from_mask(model.standing_pins_mask)?,
        clear_pins: model.clear_pins,
        created_by_user_id: model.created_by_user_id,
        created_at: model.created_at,
        updated_at: model.updated_at,
    })
}

fn db_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::normalize_template_ids;

    #[test]
    fn normalize_template_ids_removes_duplicates_and_invalid_values() {
        assert_eq!(normalize_template_ids(&[4, 1, 4, -2, 0, 3]), vec![1, 3, 4]);
    }
}

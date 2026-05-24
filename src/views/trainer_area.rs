use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::input::Input;
use crate::components::ui::label::Label;
use crate::components::ui::sheet::{
    Sheet, SheetContentClose, SheetDescription, SheetHeader, SheetTitle,
};
use crate::components::ui::tabs::{TabContent, TabList, TabTrigger, Tabs, TabsVariant};
use crate::components::ui::textarea::Textarea;
use crate::components::{
    show_error_toast, show_success_toast, ConfirmActionDialog, EmptyStatePanel, LoadingPanel,
    PageHeader, SectionPanel, StatusBanner, StatusBannerTone,
};
use crate::dashboard::get_dashboard_context;
use crate::group_trainers::list_group_trainers;
use crate::training_management::{
    create_training_plan, create_training_template, delete_training_plan,
    delete_training_template, list_training_plans, list_training_templates, update_training_plan,
    update_training_template, CreateTrainingPlanInput, CreateTrainingTemplateInput,
    TrainingPlanSummary, TrainingTemplateSummary, UpdateTrainingPlanInput,
    UpdateTrainingTemplateInput,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::use_toast;
use std::collections::BTreeSet;

#[component]
pub fn TrainerArea() -> Element {
    let mut refresh = use_signal(|| 0_u64);
    let mut active_tab = use_signal(|| Some("templates".to_string()));
    let mut selected_group_id = use_signal(|| None::<i32>);
    let toast = use_toast();

    let context_resource = use_server_future(move || {
        let _ = refresh();
        async move { get_dashboard_context().await }
    })?;
    let context_state = context_resource.read().as_ref().cloned();
    let managed_groups = match &context_state {
        Some(Ok(context)) => context.managed_groups.clone(),
        _ => Vec::new(),
    };
    let selected_group = managed_groups
        .iter()
        .find(|group| Some(group.group_id) == selected_group_id())
        .cloned();
    let selected_group_for_template_submit = selected_group.clone();
    let selected_group_for_plan_submit = selected_group.clone();

    use_effect(move || {
        let has_selected_group = selected_group_id().is_some();
        let valid_selection = managed_groups
            .iter()
            .any(|group| Some(group.group_id) == selected_group_id());

        if !managed_groups.is_empty() && (!has_selected_group || !valid_selection) {
            selected_group_id.set(Some(managed_groups[0].group_id));
        }
    });

    let template_resource = use_server_future(move || {
        let _ = refresh();
        let group_id = selected_group_id();
        async move {
            if let Some(group_id) = group_id {
                list_training_templates(group_id).await
            } else {
                Ok(Vec::new())
            }
        }
    })?;
    let template_state = template_resource.read().as_ref().cloned();

    let plan_resource = use_server_future(move || {
        let _ = refresh();
        let group_id = selected_group_id();
        async move {
            if let Some(group_id) = group_id {
                list_training_plans(group_id).await
            } else {
                Ok(Vec::new())
            }
        }
    })?;
    let plan_state = plan_resource.read().as_ref().cloned();

    let trainer_resource = use_server_future(move || {
        let _ = refresh();
        let group_id = selected_group_id();
        async move {
            if let Some(group_id) = group_id {
                list_group_trainers(group_id).await
            } else {
                Ok(Vec::new())
            }
        }
    })?;
    let trainer_state = trainer_resource.read().as_ref().cloned();

    let mut template_sheet_open = use_signal(|| false);
    let mut editing_template = use_signal(|| None::<TrainingTemplateSummary>);
    let mut template_title = use_signal(String::new);
    let mut template_description = use_signal(String::new);
    let mut template_throws = use_signal(String::new);
    let mut template_target_score = use_signal(String::new);
    let mut template_standing_pins = use_signal(String::new);
    let mut template_clear_pins = use_signal(|| false);
    let mut template_busy = use_signal(|| false);
    let mut delete_template_target = use_signal(|| None::<TrainingTemplateSummary>);
    let mut delete_template_busy = use_signal(|| false);

    let mut plan_sheet_open = use_signal(|| false);
    let mut editing_plan = use_signal(|| None::<TrainingPlanSummary>);
    let mut plan_title = use_signal(String::new);
    let mut plan_day = use_signal(String::new);
    let mut plan_note = use_signal(String::new);
    let mut plan_trainer_user_id = use_signal(|| None::<i32>);
    let mut plan_template_ids = use_signal(BTreeSet::<i32>::new);
    let mut plan_busy = use_signal(|| false);
    let mut delete_plan_target = use_signal(|| None::<TrainingPlanSummary>);
    let mut delete_plan_busy = use_signal(|| false);

    rsx! {
        section { class: "page-section",
            div { class: "page-stack page-stack--spacious",
                PageHeader {
                    title: "Trainerbereich".to_string(),
                    description: "Verwalte Trainingsvorlagen und konkrete Trainingstage je Gruppe in einem zentralen Arbeitsbereich.".to_string(),
                    eyebrow: Some("Trainingsplanung".to_string()),
                }

                match context_state {
                    None => rsx! {
                        LoadingPanel { title: "Trainerbereich".to_string(), lines: 4 }
                    },
                    Some(Err(error)) => rsx! {
                        StatusBanner {
                            tone: StatusBannerTone::Error,
                            title: Some("Trainerbereich konnte nicht geladen werden".to_string()),
                            message: error.to_string(),
                        }
                    },
                    Some(Ok(context)) => {
                        if context.managed_groups.is_empty() {
                            rsx! {
                                SectionPanel {
                                    title: "Keine Gruppen verfügbar".to_string(),
                                    description: "Für diesen Bereich brauchst du mindestens eine betreute Gruppe.".to_string(),
                                    EmptyStatePanel {
                                        title: "Noch keine Trainingsgruppe zugewiesen".to_string(),
                                        message: if context.user.is_system_admin {
                                            "System-Admins sehen diesen Bereich ebenfalls, können aber erst mit betreuten Gruppen arbeiten, sobald Trainerzuweisungen vorhanden sind.".to_string()
                                        } else {
                                            "Sobald dir eine Gruppe als Trainer zugewiesen wurde, kannst du hier Vorlagen und Trainingstage verwalten.".to_string()
                                        },
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                div { class: "workflow-summary-card surface-card trainer-workspace-card",
                                    div { class: "workflow-summary-card__header",
                                        div { class: "workflow-summary-card__copy",
                                            p { class: "metric-card__label", "Aktiver Planungsbereich" }
                                            p { class: "workflow-summary-card__title",
                                                {selected_group.as_ref().map(|group| group.group_name.clone()).unwrap_or_else(|| "Gruppe wird vorbereitet".to_string())}
                                            }
                                            p { class: "workflow-summary-card__text",
                                                {selected_group.as_ref().map(|group| format!("Vorlagen und Trainingstage werden für {} im Verein {} geladen.", group.group_name, group.club_name)).unwrap_or_else(|| "Wähle eine Gruppe aus, um Inhalte zu laden.".to_string())}
                                            }
                                        }
                                        div { class: "detail-badges",
                                            Badge { variant: BadgeVariant::Secondary, "{context.managed_groups.len()} Gruppen" }
                                            if let Some(group) = &selected_group {
                                                Badge { variant: BadgeVariant::Outline, "{group.club_name}" }
                                            }
                                        }
                                    }
                                    div { class: "form-grid trainer-group-picker" ,
                                        Label { html_for: "trainer-group-select", "Gruppe auswählen" }
                                        select {
                                            id: "trainer-group-select",
                                            class: "trainer-select",
                                            value: selected_group_id().map(|value| value.to_string()).unwrap_or_default(),
                                            onchange: move |event| {
                                                let value = event.value();
                                                selected_group_id.set(value.parse::<i32>().ok());
                                            },
                                            for group in context.managed_groups.clone() {
                                                option {
                                                    key: "{group.group_id}",
                                                    value: "{group.group_id}",
                                                    "{group.group_name} · {group.club_name}"
                                                }
                                            }
                                        }
                                    }
                                }

                                Tabs {
                                    class: "club-detail-tabs trainer-tabs".to_string(),
                                    value: ReadSignal::new(active_tab),
                                    default_value: "templates".to_string(),
                                    on_value_change: move |value| active_tab.set(Some(value)),
                                    variant: TabsVariant::Ghost,
                                    TabList {
                                        TabTrigger {
                                            index: 0usize,
                                            value: "templates".to_string(),
                                            "Vorlagen"
                                        }
                                        TabTrigger {
                                            index: 1usize,
                                            value: "plans".to_string(),
                                            "Trainingstage"
                                        }
                                    }

                                    TabContent {
                                        index: 0usize,
                                        value: "templates".to_string(),
                                        div { class: "tab-section",
                                            SectionPanel {
                                                title: "Vorlagen".to_string(),
                                                description: "Erstelle Übungen, pflege Zielwerte und halte wiederverwendbare Trainingsbausteine bereit.".to_string(),
                                                actions: Some(rsx! {
                                                    Button {
                                                        variant: ButtonVariant::Secondary,
                                                        onclick: move |_| {
                                                            editing_template.set(None);
                                                            template_title.set(String::new());
                                                            template_description.set(String::new());
                                                            template_throws.set(String::new());
                                                            template_target_score.set(String::new());
                                                            template_standing_pins.set(String::new());
                                                            template_clear_pins.set(false);
                                                            template_sheet_open.set(true);
                                                        },
                                                        "Neue Vorlage"
                                                    }
                                                }),
                                                match template_state.clone() {
                                                    None => rsx! {
                                                        LoadingPanel { title: "Vorlagen".to_string(), lines: 4 }
                                                    },
                                                    Some(Err(error)) => rsx! {
                                                        StatusBanner {
                                                            tone: StatusBannerTone::Error,
                                                            title: Some("Vorlagen konnten nicht geladen werden".to_string()),
                                                            message: error.to_string(),
                                                        }
                                                    },
                                                    Some(Ok(templates)) if templates.is_empty() => rsx! {
                                                        EmptyStatePanel {
                                                            title: "Noch keine Vorlage angelegt".to_string(),
                                                            message: "Lege die erste Trainingsvorlage an, damit du daraus später konkrete Trainingstage zusammenstellen kannst.".to_string(),
                                                        }
                                                    },
                                                    Some(Ok(templates)) => rsx! {
                                                        div { class: "trainer-card-grid",
                                                            for template in templates {
                                                                div { key: "{template.id}", class: "detail-card trainer-entity-card",
                                                                    div { class: "detail-card-header",
                                                                        div { class: "detail-card-copy",
                                                                            p { class: "section-label", "{template.title}" }
                                                                            p { class: "detail-card-title", "{template.description_or_fallback()}" }
                                                                            p { class: "section-meta", "{template.meta_line()}" }
                                                                        }
                                                                        div { class: "detail-badges",
                                                                            if let Some(throws) = template.number_of_throws {
                                                                                Badge { variant: BadgeVariant::Outline, "{throws} Würfe" }
                                                                            }
                                                                            if let Some(score) = template.target_score {
                                                                                Badge { variant: BadgeVariant::Secondary, "Ziel {score}" }
                                                                            }
                                                                        }
                                                                    }
                                                                    div { class: "section-actions",
                                                                        Button {
                                                                            variant: ButtonVariant::Outline,
                                                                            size: ButtonSize::Sm,
                                                                            onclick: {
                                                                                let template = template.clone();
                                                                                move |_| {
                                                                                    editing_template.set(Some(template.clone()));
                                                                                    template_title.set(template.title.clone());
                                                                                    template_description.set(template.description.clone());
                                                                                    template_throws.set(template.number_of_throws.map(|value| value.to_string()).unwrap_or_default());
                                                                                    template_target_score.set(template.target_score.map(|value| value.to_string()).unwrap_or_default());
                                                                                    template_standing_pins.set(format_standing_pins(template.standing_pins.as_deref()));
                                                                                    template_clear_pins.set(template.clear_pins.unwrap_or(false));
                                                                                    template_sheet_open.set(true);
                                                                                }
                                                                            },
                                                                            "Bearbeiten"
                                                                        }
                                                                        Button {
                                                                            variant: ButtonVariant::Destructive,
                                                                            size: ButtonSize::Sm,
                                                                            onclick: {
                                                                                let template = template.clone();
                                                                                move |_| delete_template_target.set(Some(template.clone()))
                                                                            },
                                                                            "Löschen"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    TabContent {
                                        index: 1usize,
                                        value: "plans".to_string(),
                                        div { class: "tab-section",
                                            SectionPanel {
                                                title: "Trainingstage".to_string(),
                                                description: "Plane konkrete Trainingstage mit Datum, Notiz, verantwortlichem Trainer und den passenden Vorlagen.".to_string(),
                                                actions: Some(rsx! {
                                                    Button {
                                                        variant: ButtonVariant::Secondary,
                                                        onclick: move |_| {
                                                            editing_plan.set(None);
                                                            plan_title.set(String::new());
                                                            plan_day.set(String::new());
                                                            plan_note.set(String::new());
                                                            plan_trainer_user_id.set(None);
                                                            plan_template_ids.set(BTreeSet::new());
                                                            plan_sheet_open.set(true);
                                                        },
                                                        "Neuer Trainingstag"
                                                    }
                                                }),
                                                match plan_state.clone() {
                                                    None => rsx! {
                                                        LoadingPanel { title: "Trainingstage".to_string(), lines: 4 }
                                                    },
                                                    Some(Err(error)) => rsx! {
                                                        StatusBanner {
                                                            tone: StatusBannerTone::Error,
                                                            title: Some("Trainingstage konnten nicht geladen werden".to_string()),
                                                            message: error.to_string(),
                                                        }
                                                    },
                                                    Some(Ok(plans)) if plans.is_empty() => rsx! {
                                                        EmptyStatePanel {
                                                            title: "Noch kein Trainingstag geplant".to_string(),
                                                            message: "Lege einen konkreten Trainingstag an und ordne ihm direkt passende Vorlagen zu.".to_string(),
                                                        }
                                                    },
                                                    Some(Ok(plans)) => rsx! {
                                                        div { class: "trainer-card-grid",
                                                            for plan in plans {
                                                                div { key: "{plan.id}", class: "detail-card trainer-entity-card",
                                                                    div { class: "detail-card-header",
                                                                        div { class: "detail-card-copy",
                                                                            p { class: "section-label", "{plan.day}" }
                                                                            p { class: "detail-card-title", "{plan.title}" }
                                                                            p { class: "section-meta", "{plan.plan_meta_line()}" }
                                                                        }
                                                                        div { class: "detail-badges",
                                                                            Badge { variant: BadgeVariant::Secondary, "{plan.templates.len()} Vorlagen" }
                                                                            if let Some(name) = &plan.trainer_username {
                                                                                Badge { variant: BadgeVariant::Outline, "{name}" }
                                                                            }
                                                                        }
                                                                    }
                                                                    if !plan.note.trim().is_empty() {
                                                                        p { class: "section-note", "{plan.note}" }
                                                                    }
                                                                    if !plan.templates.is_empty() {
                                                                        div { class: "detail-badges detail-badges--wrap",
                                                                            for template in plan.templates.clone() {
                                                                                Badge { key: "{template.id}", variant: BadgeVariant::Outline, "{template.title}" }
                                                                            }
                                                                        }
                                                                    }
                                                                    div { class: "section-actions",
                                                                        Button {
                                                                            variant: ButtonVariant::Outline,
                                                                            size: ButtonSize::Sm,
                                                                            onclick: {
                                                                                let plan = plan.clone();
                                                                                move |_| {
                                                                                    editing_plan.set(Some(plan.clone()));
                                                                                    plan_title.set(plan.title.clone());
                                                                                    plan_day.set(plan.day.clone());
                                                                                    plan_note.set(plan.note.clone());
                                                                                    plan_trainer_user_id.set(plan.trainer_user_id);
                                                                                    plan_template_ids.set(plan.templates.iter().map(|template| template.id).collect());
                                                                                    plan_sheet_open.set(true);
                                                                                }
                                                                            },
                                                                            "Bearbeiten"
                                                                        }
                                                                        Button {
                                                                            variant: ButtonVariant::Destructive,
                                                                            size: ButtonSize::Sm,
                                                                            onclick: {
                                                                                let plan = plan.clone();
                                                                                move |_| delete_plan_target.set(Some(plan.clone()))
                                                                            },
                                                                            "Löschen"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Sheet {
            open: template_sheet_open(),
            on_open_change: move |open: bool| template_sheet_open.set(open),
            div { class: "mobile-form-sheet",
                SheetHeader {
                    SheetTitle {
                        {if editing_template().is_some() { "Vorlage bearbeiten" } else { "Neue Vorlage" }}
                    }
                    SheetDescription {
                        "Pflege Titel, Zielwerte und optionale Kegel-Konstellationen für diese Gruppe."
                    }
                    SheetContentClose {}
                }
                div { class: "mobile-sheet-body mobile-form-stack",
                    div { class: "form-grid",
                        div { class: "form-grid",
                            Label { html_for: "template-title", "Titel" }
                            Input {
                                id: "template-title",
                                value: template_title(),
                                placeholder: "Zum Beispiel Abräumen unter Druck",
                                oninput: move |event: FormEvent| template_title.set(event.value()),
                            }
                        }
                        div { class: "form-grid",
                            Label { html_for: "template-description", "Beschreibung" }
                            Textarea {
                                id: "template-description",
                                value: template_description(),
                                placeholder: "Ablauf, Fokus und Hinweise für das Training beschreiben",
                                oninput: move |event: FormEvent| template_description.set(event.value()),
                            }
                        }
                        div { class: "form-grid-2",
                            div { class: "form-grid",
                                Label { html_for: "template-throws", "Wurfanzahl" }
                                Input {
                                    id: "template-throws",
                                    r#type: "number",
                                    min: "0",
                                    value: template_throws(),
                                    placeholder: "Optional",
                                    oninput: move |event: FormEvent| template_throws.set(event.value()),
                                }
                            }
                            div { class: "form-grid",
                                Label { html_for: "template-target-score", "Zielpunktzahl" }
                                Input {
                                    id: "template-target-score",
                                    r#type: "number",
                                    min: "0",
                                    value: template_target_score(),
                                    placeholder: "Optional",
                                    oninput: move |event: FormEvent| template_target_score.set(event.value()),
                                }
                            }
                        }
                        div { class: "form-grid",
                            Label { html_for: "template-standing-pins", "Stehende Kegel" }
                            Input {
                                id: "template-standing-pins",
                                value: template_standing_pins(),
                                placeholder: "Zum Beispiel 1,4,9",
                                oninput: move |event: FormEvent| template_standing_pins.set(event.value()),
                            }
                            p { class: "section-meta", "Leer lassen, wenn keine feste Kegelkonstellation vorgegeben ist." }
                        }
                        label { class: "trainer-checkbox-row",
                            input {
                                r#type: "checkbox",
                                checked: template_clear_pins(),
                                onchange: move |event| {
                                    let checked = matches!(event.value().as_str(), "true" | "on");
                                    template_clear_pins.set(checked);
                                },
                            }
                            span { "Abräumen nach jedem Versuch einplanen" }
                        }
                    }
                    div { class: "section-actions mobile-form-actions",
                        Button {
                            variant: ButtonVariant::Secondary,
                            disabled: template_busy(),
                            onclick: move |_| {
                                let Some(group) = selected_group_for_template_submit.clone() else {
                                    show_error_toast(
                                        toast,
                                        "Vorlage konnte nicht gespeichert werden",
                                        "Es ist keine Gruppe ausgewählt.",
                                    );
                                    return;
                                };

                                let number_of_throws = match parse_optional_number("Die Wurfanzahl", &template_throws()) {
                                    Ok(value) => value,
                                    Err(error) => {
                                        show_error_toast(toast, "Vorlage konnte nicht gespeichert werden", error);
                                        return;
                                    }
                                };
                                let target_score = match parse_optional_number("Die Zielpunktzahl", &template_target_score()) {
                                    Ok(value) => value,
                                    Err(error) => {
                                        show_error_toast(toast, "Vorlage konnte nicht gespeichert werden", error);
                                        return;
                                    }
                                };
                                let standing_pins = match parse_standing_pins(&template_standing_pins()) {
                                    Ok(value) => value,
                                    Err(error) => {
                                        show_error_toast(toast, "Vorlage konnte nicht gespeichert werden", error);
                                        return;
                                    }
                                };

                                let editing = editing_template();
                                let title = template_title();
                                let description = template_description();
                                let clear_pins = Some(template_clear_pins());
                                spawn(async move {
                                    template_busy.set(true);
                                    let result = if let Some(existing) = editing {
                                        update_training_template(UpdateTrainingTemplateInput {
                                            template_id: existing.id,
                                            club_id: group.club_id,
                                            group_id: group.group_id,
                                            title,
                                            description,
                                            number_of_throws,
                                            target_score,
                                            standing_pins,
                                            clear_pins,
                                        }).await
                                    } else {
                                        create_training_template(CreateTrainingTemplateInput {
                                            club_id: group.club_id,
                                            group_id: group.group_id,
                                            title,
                                            description,
                                            number_of_throws,
                                            target_score,
                                            standing_pins,
                                            clear_pins,
                                        }).await
                                    };
                                    template_busy.set(false);

                                    match result {
                                        Ok(_) => {
                                            template_sheet_open.set(false);
                                            refresh.with_mut(|value| *value += 1);
                                            show_success_toast(
                                                toast,
                                                "Vorlage gespeichert",
                                                "Die Trainingsvorlage wurde erfolgreich aktualisiert.",
                                            );
                                        }
                                        Err(error) => show_error_toast(
                                            toast,
                                            "Vorlage konnte nicht gespeichert werden",
                                            error.to_string(),
                                        ),
                                    }
                                });
                            },
                            {if template_busy() { "Speichert..." } else { "Vorlage speichern" }}
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            disabled: template_busy(),
                            onclick: move |_| template_sheet_open.set(false),
                            "Abbrechen"
                        }
                    }
                }
            }
        }

        Sheet {
            open: plan_sheet_open(),
            on_open_change: move |open: bool| plan_sheet_open.set(open),
            div { class: "mobile-form-sheet",
                SheetHeader {
                    SheetTitle {
                        {if editing_plan().is_some() { "Trainingstag bearbeiten" } else { "Neuer Trainingstag" }}
                    }
                    SheetDescription {
                        "Lege Datum, Trainer und die passenden Vorlagen für diesen Trainingstag fest."
                    }
                    SheetContentClose {}
                }
                div { class: "mobile-sheet-body mobile-form-stack",
                    div { class: "form-grid",
                        div { class: "form-grid",
                            Label { html_for: "plan-title", "Titel" }
                            Input {
                                id: "plan-title",
                                value: plan_title(),
                                placeholder: "Zum Beispiel Freitagstraining Jugend",
                                oninput: move |event: FormEvent| plan_title.set(event.value()),
                            }
                        }
                        div { class: "form-grid",
                            Label { html_for: "plan-day", "Datum" }
                            Input {
                                id: "plan-day",
                                r#type: "date",
                                value: plan_day(),
                                oninput: move |event: FormEvent| plan_day.set(event.value()),
                            }
                        }
                        div { class: "form-grid",
                            Label { html_for: "plan-trainer", "Verantwortlicher Trainer" }
                            select {
                                id: "plan-trainer",
                                class: "trainer-select",
                                value: plan_trainer_user_id().map(|value| value.to_string()).unwrap_or_default(),
                                onchange: move |event| {
                                    let value = event.value();
                                    if value.trim().is_empty() {
                                        plan_trainer_user_id.set(None);
                                    } else {
                                        plan_trainer_user_id.set(value.parse::<i32>().ok());
                                    }
                                },
                                option { value: "", "Kein Trainer festgelegt" }
                                if let Some(Ok(trainers)) = trainer_state.clone() {
                                    for trainer in trainers {
                                        option {
                                            key: "{trainer.user_id}",
                                            value: "{trainer.user_id}",
                                            "{trainer.username}"
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "form-grid",
                            Label { html_for: "plan-note", "Notiz" }
                            Textarea {
                                id: "plan-note",
                                value: plan_note(),
                                placeholder: "Besondere Schwerpunkte, Material oder Hinweise notieren",
                                oninput: move |event: FormEvent| plan_note.set(event.value()),
                            }
                        }
                        div { class: "form-grid",
                            p { class: "section-label", "Vorlagen für diesen Trainingstag" }
                            match template_state.clone() {
                                None => rsx! {
                                    p { class: "section-meta", "Vorlagen werden geladen..." }
                                },
                                Some(Err(error)) => rsx! {
                                    p { class: "training-scope-card__warning", "{error}" }
                                },
                                Some(Ok(templates)) if templates.is_empty() => rsx! {
                                    p { class: "section-meta", "Lege zuerst mindestens eine Vorlage für diese Gruppe an." }
                                },
                                Some(Ok(templates)) => rsx! {
                                    div { class: "trainer-checkbox-list",
                                        for template in templates {
                                            label { key: "{template.id}", class: "trainer-checkbox-row trainer-checkbox-row--card",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: plan_template_ids().contains(&template.id),
                                                    onchange: {
                                                        let template_id = template.id;
                                                        move |event| {
                                                            let checked = matches!(event.value().as_str(), "true" | "on");
                                                            plan_template_ids.with_mut(|selected| {
                                                                if checked {
                                                                    selected.insert(template_id);
                                                                } else {
                                                                    selected.remove(&template_id);
                                                                }
                                                            });
                                                        }
                                                    },
                                                }
                                                div { class: "detail-row-copy",
                                                    span { class: "detail-row-title", "{template.title}" }
                                                    p { class: "detail-row-meta", "{template.meta_line()}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "section-actions mobile-form-actions",
                        Button {
                            variant: ButtonVariant::Secondary,
                            disabled: plan_busy(),
                            onclick: move |_| {
                                let Some(group) = selected_group_for_plan_submit.clone() else {
                                    show_error_toast(
                                        toast,
                                        "Trainingstag konnte nicht gespeichert werden",
                                        "Es ist keine Gruppe ausgewählt.",
                                    );
                                    return;
                                };

                                let editing = editing_plan();
                                let title = plan_title();
                                let day = plan_day();
                                let note = plan_note();
                                let trainer_user_id = plan_trainer_user_id();
                                let template_ids = plan_template_ids().into_iter().collect::<Vec<_>>();

                                spawn(async move {
                                    plan_busy.set(true);
                                    let result = if let Some(existing) = editing {
                                        update_training_plan(UpdateTrainingPlanInput {
                                            plan_id: existing.id,
                                            club_id: group.club_id,
                                            group_id: group.group_id,
                                            title,
                                            day,
                                            note,
                                            trainer_user_id,
                                            template_ids,
                                        }).await
                                    } else {
                                        create_training_plan(CreateTrainingPlanInput {
                                            club_id: group.club_id,
                                            group_id: group.group_id,
                                            title,
                                            day,
                                            note,
                                            trainer_user_id,
                                            template_ids,
                                        }).await
                                    };
                                    plan_busy.set(false);

                                    match result {
                                        Ok(_) => {
                                            plan_sheet_open.set(false);
                                            refresh.with_mut(|value| *value += 1);
                                            show_success_toast(
                                                toast,
                                                "Trainingstag gespeichert",
                                                "Der Trainingstag wurde erfolgreich aktualisiert.",
                                            );
                                        }
                                        Err(error) => show_error_toast(
                                            toast,
                                            "Trainingstag konnte nicht gespeichert werden",
                                            error.to_string(),
                                        ),
                                    }
                                });
                            },
                            {if plan_busy() { "Speichert..." } else { "Trainingstag speichern" }}
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            disabled: plan_busy(),
                            onclick: move |_| plan_sheet_open.set(false),
                            "Abbrechen"
                        }
                    }
                }
            }
        }

        if let Some(template) = delete_template_target() {
            ConfirmActionDialog {
                open: true,
                title: "Vorlage löschen".to_string(),
                description: format!("Die Vorlage „{}“ wird dauerhaft entfernt.", template.title),
                confirm_label: "Vorlage löschen".to_string(),
                busy: delete_template_busy(),
                on_open_change: move |open: bool| {
                    if !open {
                        delete_template_target.set(None);
                    }
                },
                on_confirm: move |_| {
                    let template = template.clone();
                    spawn(async move {
                        delete_template_busy.set(true);
                        let result = delete_training_template(template.id).await;
                        delete_template_busy.set(false);

                        match result {
                            Ok(()) => {
                                delete_template_target.set(None);
                                refresh.with_mut(|value| *value += 1);
                                show_success_toast(
                                    toast,
                                    "Vorlage gelöscht",
                                    "Die Trainingsvorlage wurde entfernt.",
                                );
                            }
                            Err(error) => show_error_toast(
                                toast,
                                "Vorlage konnte nicht gelöscht werden",
                                error.to_string(),
                            ),
                        }
                    });
                },
            }
        }

        if let Some(plan) = delete_plan_target() {
            ConfirmActionDialog {
                open: true,
                title: "Trainingstag löschen".to_string(),
                description: format!("Der Trainingstag „{}“ am {} wird dauerhaft entfernt.", plan.title, plan.day),
                confirm_label: "Trainingstag löschen".to_string(),
                busy: delete_plan_busy(),
                on_open_change: move |open: bool| {
                    if !open {
                        delete_plan_target.set(None);
                    }
                },
                on_confirm: move |_| {
                    let plan = plan.clone();
                    spawn(async move {
                        delete_plan_busy.set(true);
                        let result = delete_training_plan(plan.id).await;
                        delete_plan_busy.set(false);

                        match result {
                            Ok(()) => {
                                delete_plan_target.set(None);
                                refresh.with_mut(|value| *value += 1);
                                show_success_toast(
                                    toast,
                                    "Trainingstag gelöscht",
                                    "Der Trainingstag wurde entfernt.",
                                );
                            }
                            Err(error) => show_error_toast(
                                toast,
                                "Trainingstag konnte nicht gelöscht werden",
                                error.to_string(),
                            ),
                        }
                    });
                },
            }
        }
    }
}

trait TrainingTemplateViewExt {
    fn description_or_fallback(&self) -> String;
    fn meta_line(&self) -> String;
}

impl TrainingTemplateViewExt for TrainingTemplateSummary {
    fn description_or_fallback(&self) -> String {
        let description = self.description.trim();
        if description.is_empty() {
            "Keine Beschreibung hinterlegt".to_string()
        } else {
            description.to_string()
        }
    }

    fn meta_line(&self) -> String {
        let mut parts = Vec::new();

        if let Some(score) = self.target_score {
            parts.push(format!("Zielpunktzahl {score}"));
        }

        if let Some(pins) = self.standing_pins.as_deref() {
            parts.push(format!("Stehende Kegel {}", format_standing_pins(Some(pins))));
        }

        match self.clear_pins {
            Some(true) => parts.push("Mit Abräumen".to_string()),
            Some(false) => parts.push("Ohne Abräumen".to_string()),
            None => {}
        }

        if parts.is_empty() {
            "Keine Zusatzparameter hinterlegt".to_string()
        } else {
            parts.join(" · ")
        }
    }
}

trait TrainingPlanViewExt {
    fn plan_meta_line(&self) -> String;
}

impl TrainingPlanViewExt for TrainingPlanSummary {
    fn plan_meta_line(&self) -> String {
        let mut parts = Vec::new();

        if let Some(name) = &self.trainer_username {
            parts.push(format!("Trainer {name}"));
        }

        if self.templates.is_empty() {
            parts.push("Noch keine Vorlagen zugeordnet".to_string());
        } else {
            parts.push(format!("{} Vorlagen zugeordnet", self.templates.len()));
        }

        parts.join(" · ")
    }
}

fn parse_optional_number(label: &str, value: &str) -> Result<Option<i32>, String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }

    value
        .parse::<i32>()
        .map(Some)
        .map_err(|_| format!("{label} muss eine ganze Zahl sein."))
}

fn parse_standing_pins(value: &str) -> Result<Option<Vec<u8>>, String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }

    let mut pins = Vec::new();
    for part in value.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        let pin = trimmed
            .parse::<u8>()
            .map_err(|_| "Stehende Kegel müssen als Zahlen von 1 bis 9 angegeben werden.".to_string())?;
        pins.push(pin);
    }

    if pins.is_empty() {
        Ok(None)
    } else {
        Ok(Some(pins))
    }
}

fn format_standing_pins(pins: Option<&[u8]>) -> String {
    pins.map(|pins| {
        pins.iter()
            .map(|pin| pin.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    })
    .unwrap_or_else(|| "Keine".to_string())
}

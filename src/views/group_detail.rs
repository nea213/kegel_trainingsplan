use crate::club_memberships::{
    assign_player_to_team, list_unassigned_club_members, PlayerAssignmentInput,
};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::input::Input;
use crate::components::ui::item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemGroup, ItemSeparator, ItemTitle,
};
use crate::components::ui::label::Label;
use crate::components::ui::textarea::Textarea;
use crate::components::{
    EmptyStatePanel, LoadingPanel, PageHeader, SectionPanel, StatusBanner, StatusBannerTone,
};
use crate::dashboard::get_dashboard_context;
use crate::teams::list_teams_for_group;
use crate::training::{
    create_training_session, format_training_range, list_group_training_sessions,
    training_scope_label, CreateTrainingSessionInput,
};
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum TrainingScopeSelection {
    WholeGroup,
    ActiveTeam,
}

#[component]
pub fn GroupDetail(group_id: i32) -> Element {
    let mut refresh = use_signal(|| 0_u64);
    let context_resource = use_server_future(move || {
        let _ = refresh();
        async move { get_dashboard_context().await }
    })?;
    let context_state = context_resource.read().as_ref().cloned();
    let mut selected_team = use_signal(|| None::<i32>);
    let mut assigning = use_signal(|| None::<i32>);
    let mut status = use_signal(|| None::<(bool, String)>);
    let mut training_title = use_signal(String::new);
    let mut training_description = use_signal(String::new);
    let mut training_location = use_signal(String::new);
    let mut training_start_at = use_signal(String::new);
    let mut training_end_at = use_signal(String::new);
    let mut training_scope = use_signal(|| TrainingScopeSelection::WholeGroup);
    let mut creating_training = use_signal(|| false);

    match context_state {
        None => rsx! {
            section { class: "page-section",
                div { class: "page-stack",
                    LoadingPanel { title: "Gruppenbereich".to_string(), lines: 4 }
                }
            }
        },
        Some(Err(error)) => rsx! {
            section { class: "page-section",
                div { class: "page-stack",
                    PageHeader {
                        title: "Gruppenbereich".to_string(),
                        description: "Die Arbeitsseite konnte nicht geladen werden.".to_string(),
                        eyebrow: Some("Trainerbereich".to_string()),
                    }
                    StatusBanner {
                        tone: StatusBannerTone::Error,
                        title: Some("Laden fehlgeschlagen".to_string()),
                        message: error.to_string(),
                    }
                }
            }
        },
        Some(Ok(context)) => {
            let Some(group) = context
                .managed_groups
                .iter()
                .find(|group| group.group_id == group_id)
                .cloned()
            else {
                return rsx! {
                    section { class: "page-section",
                        div { class: "page-stack",
                            PageHeader {
                                title: "Gruppe nicht verfügbar".to_string(),
                                description: "Du kannst nur Gruppen öffnen, die dir als Trainer zugewiesen sind.".to_string(),
                                eyebrow: Some("Trainerbereich".to_string()),
                            }
                            StatusBanner {
                                tone: StatusBannerTone::Info,
                                title: Some("Kein Zugriff".to_string()),
                                message: "Bitte öffne eine Gruppe aus deinem Dashboard.".to_string(),
                            }
                        }
                    }
                };
            };

            let teams_resource = use_server_future(move || {
                let _ = refresh();
                async move { list_teams_for_group(group_id).await }
            })?;
            let members_resource = use_server_future(move || {
                let _ = refresh();
                async move { list_unassigned_club_members(group.club_id).await }
            })?;
            let training_resource = use_server_future(move || {
                let _ = refresh();
                async move { list_group_training_sessions(group_id).await }
            })?;
            let teams_state = teams_resource.read().as_ref().cloned();
            let members_state = members_resource.read().as_ref().cloned();
            let training_state = training_resource.read().as_ref().cloned();
            let active_team_name = match &teams_state {
                Some(Ok(teams)) => teams
                    .iter()
                    .find(|team| Some(team.id) == selected_team())
                    .map(|team| team.name.clone()),
                _ => None,
            };
            let active_team_ready =
                training_scope() != TrainingScopeSelection::ActiveTeam || selected_team().is_some();

            rsx! {
                section { class: "page-section",
                    div { class: "page-stack page-stack--spacious",
                        PageHeader {
                            title: group.group_name.clone(),
                            description: "Arbeite Schritt für Schritt: Mannschaft festlegen, Spieler organisieren und danach Trainings planen.".to_string(),
                            eyebrow: Some(group.club_name.clone()),
                        }

                        if let Some((success, message)) = status() {
                            StatusBanner {
                                tone: if success {
                                    StatusBannerTone::Success
                                } else {
                                    StatusBannerTone::Error
                                },
                                message,
                            }
                        }

                        div { class: "workflow-grid",
                            div { class: "workflow-column",
                                SectionPanel {
                                    title: "1. Mannschaft wählen".to_string(),
                                    description: "Lege fest, mit welcher Mannschaft du gerade arbeitest.".to_string(),
                                    div { class: "workflow-context",
                                        div { class: "detail-card detail-card-muted",
                                            p { class: "section-label", "Aktive Mannschaft" }
                                            p { class: "detail-card-title",
                                                {active_team_name.clone().unwrap_or_else(|| "Noch keine Mannschaft ausgewählt".to_string())}
                                            }
                                            p { class: "section-meta",
                                                "Diese Auswahl wird für Spielerzuweisungen genutzt und kann optional auch das Ziel neuer Trainings sein."
                                            }
                                        }
                                        match teams_state {
                                            None => rsx! {
                                                LoadingPanel { title: "Mannschaften".to_string(), lines: 4 }
                                            },
                                            Some(Err(error)) => rsx! {
                                                StatusBanner {
                                                    tone: StatusBannerTone::Error,
                                                    title: Some("Mannschaften konnten nicht geladen werden".to_string()),
                                                    message: error.to_string(),
                                                }
                                            },
                                            Some(Ok(teams)) if teams.is_empty() => rsx! {
                                                EmptyStatePanel {
                                                    title: "Keine Mannschaft vorhanden".to_string(),
                                                    message: "In dieser Gruppe wurden noch keine Mannschaften angelegt.".to_string(),
                                                }
                                            },
                                            Some(Ok(teams)) => {
                                                let team_count = teams.len();

                                                rsx! {
                                                    ItemGroup {
                                                        for (index, team) in teams.into_iter().enumerate() {
                                                            Item {
                                                                class: "content-list-item",
                                                                ItemContent {
                                                                    ItemTitle { "{team.name}" }
                                                                    ItemDescription {
                                                                        {if selected_team() == Some(team.id) {
                                                                            "Aktive Mannschaft".to_string()
                                                                        } else {
                                                                            "Für Zuweisungen und teambezogene Trainings verfügbar".to_string()
                                                                        }}
                                                                    }
                                                                }
                                                                ItemActions {
                                                                    Button {
                                                                        variant: if selected_team() == Some(team.id) {
                                                                            ButtonVariant::Secondary
                                                                        } else {
                                                                            ButtonVariant::Outline
                                                                        },
                                                                        onclick: move |_| selected_team.set(Some(team.id)),
                                                                        {if selected_team() == Some(team.id) { "Aktiv" } else { "Auswählen" }}
                                                                    }
                                                                }
                                                            }
                                                            if index + 1 < team_count {
                                                                ItemSeparator {}
                                                            }
                                                        }
                                                    }
                                                }
                                            },
                                        }
                                    }
                                }

                                SectionPanel {
                                    title: "2. Spieler organisieren".to_string(),
                                    description: "Ordne wartende Spieler direkt der aktiven Mannschaft zu.".to_string(),
                                    div { class: "workflow-context",
                                        div { class: "detail-card detail-card-muted",
                                            p { class: "section-label", "Ziel für neue Zuweisungen" }
                                            p { class: "detail-card-title",
                                                {active_team_name.clone().unwrap_or_else(|| "Bitte zuerst eine Mannschaft auswählen".to_string())}
                                            }
                                        }

                                        match members_state {
                                            None => rsx! {
                                                LoadingPanel { title: "Spieler ohne Mannschaft".to_string(), lines: 4 }
                                            },
                                            Some(Err(error)) => rsx! {
                                                StatusBanner {
                                                    tone: StatusBannerTone::Error,
                                                    title: Some("Vereinsmitglieder konnten nicht geladen werden".to_string()),
                                                    message: error.to_string(),
                                                }
                                            },
                                            Some(Ok(members)) if members.is_empty() => rsx! {
                                                EmptyStatePanel {
                                                    title: "Keine offenen Zuweisungen".to_string(),
                                                    message: "Aktuell warten keine Spieler auf eine Mannschaftszuteilung.".to_string(),
                                                }
                                            },
                                            Some(Ok(members)) => {
                                                let member_count = members.len();

                                                rsx! {
                                                    ItemGroup {
                                                        for (index, member) in members.into_iter().enumerate() {
                                                            {
                                                                let member_user_id = member.user_id;
                                                                let member_club_id = member.club_id;
                                                                let member_username = member.username.clone();

                                                                rsx! {
                                                                    Item {
                                                                        class: "content-list-item",
                                                                        ItemContent {
                                                                            ItemTitle { "{member.username}" }
                                                                            ItemDescription { "Mitglied in {member.club_name}" }
                                                                        }
                                                                        ItemActions {
                                                                            Button {
                                                                                variant: ButtonVariant::Outline,
                                                                                disabled: selected_team().is_none() || assigning() == Some(member_user_id),
                                                                                onclick: move |_| {
                                                                                    let Some(team_id) = selected_team() else {
                                                                                        status.set(Some((false, "Wähle zuerst eine aktive Mannschaft aus.".to_string())));
                                                                                        return;
                                                                                    };

                                                                                    status.set(None);
                                                                                    let success_name = member_username.clone();
                                                                                    spawn(async move {
                                                                                        assigning.set(Some(member_user_id));
                                                                                        let result = assign_player_to_team(PlayerAssignmentInput {
                                                                                            club_id: member_club_id,
                                                                                            team_id,
                                                                                            user_id: member_user_id,
                                                                                        })
                                                                                        .await;
                                                                                        assigning.set(None);

                                                                                        match result {
                                                                                            Ok(()) => {
                                                                                                status.set(Some((true, format!("{} wurde der aktiven Mannschaft zugewiesen.", success_name))));
                                                                                                refresh.with_mut(|value| *value += 1);
                                                                                            }
                                                                                            Err(error) => {
                                                                                                status.set(Some((false, format!("Spieler konnte nicht zugewiesen werden: {error}"))));
                                                                                            }
                                                                                        }
                                                                                    });
                                                                                },
                                                                                {if assigning() == Some(member_user_id) { "Zuweisung..." } else { "Zuweisen" }}
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            if index + 1 < member_count {
                                                                ItemSeparator {}
                                                            }
                                                        }
                                                    }
                                                }
                                            },
                                        }
                                    }
                                }
                            }

                            div { class: "workflow-column",
                                SectionPanel {
                                    title: "3. Training planen".to_string(),
                                    description: "Wähle bewusst, ob das Training für die ganze Gruppe oder für die aktive Mannschaft gedacht ist.".to_string(),
                                    div { class: "section-stack",
                                        div { class: "detail-card detail-card-muted",
                                            p { class: "section-label", "Ziel des Trainings" }
                                            div { class: "section-actions",
                                                Button {
                                                    variant: if training_scope() == TrainingScopeSelection::WholeGroup {
                                                        ButtonVariant::Secondary
                                                    } else {
                                                        ButtonVariant::Outline
                                                    },
                                                    onclick: move |_| training_scope.set(TrainingScopeSelection::WholeGroup),
                                                    "Ganze Gruppe"
                                                }
                                                Button {
                                                    variant: if training_scope() == TrainingScopeSelection::ActiveTeam {
                                                        ButtonVariant::Secondary
                                                    } else {
                                                        ButtonVariant::Outline
                                                    },
                                                    disabled: active_team_name.is_none(),
                                                    onclick: move |_| training_scope.set(TrainingScopeSelection::ActiveTeam),
                                                    "Aktive Mannschaft"
                                                }
                                            }
                                            p { class: "section-meta",
                                                {
                                                    match training_scope() {
                                                        TrainingScopeSelection::WholeGroup => {
                                                            "Das Training wird für die gesamte Gruppe angelegt.".to_string()
                                                        }
                                                        TrainingScopeSelection::ActiveTeam => {
                                                            format!(
                                                                "Das Training wird für {} angelegt.",
                                                                active_team_name.clone().unwrap_or_else(|| "die aktive Mannschaft".to_string())
                                                            )
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        div { class: "form-grid",
                                            div { class: "auth-field",
                                                Label { html_for: "training-title", "Titel" }
                                                Input {
                                                    id: "training-title",
                                                    value: training_title(),
                                                    placeholder: "z. B. Techniktraining",
                                                    disabled: creating_training(),
                                                    oninput: move |event: FormEvent| training_title.set(event.value()),
                                                }
                                            }
                                            div { class: "auth-field",
                                                Label { html_for: "training-location", "Ort" }
                                                Input {
                                                    id: "training-location",
                                                    value: training_location(),
                                                    placeholder: "z. B. Vereinsheim Bahn 1-4",
                                                    disabled: creating_training(),
                                                    oninput: move |event: FormEvent| training_location.set(event.value()),
                                                }
                                            }
                                            div { class: "form-grid-2",
                                                div { class: "auth-field",
                                                    Label { html_for: "training-start-at", "Start" }
                                                    Input {
                                                        id: "training-start-at",
                                                        r#type: "datetime-local",
                                                        value: training_start_at(),
                                                        disabled: creating_training(),
                                                        oninput: move |event: FormEvent| training_start_at.set(event.value()),
                                                    }
                                                }
                                                div { class: "auth-field",
                                                    Label { html_for: "training-end-at", "Ende" }
                                                    Input {
                                                        id: "training-end-at",
                                                        r#type: "datetime-local",
                                                        value: training_end_at(),
                                                        disabled: creating_training(),
                                                        oninput: move |event: FormEvent| training_end_at.set(event.value()),
                                                    }
                                                }
                                            }
                                            div { class: "auth-field",
                                                Label { html_for: "training-description", "Beschreibung" }
                                                Textarea {
                                                    id: "training-description",
                                                    value: training_description(),
                                                    rows: "4",
                                                    placeholder: "Fokus, Ablauf oder Hinweise für das Training",
                                                    disabled: creating_training(),
                                                    oninput: move |event: FormEvent| training_description.set(event.value()),
                                                }
                                            }
                                        }

                                        div { class: "section-actions",
                                            Button {
                                                variant: ButtonVariant::Secondary,
                                                disabled: creating_training() || !active_team_ready,
                                                onclick: move |_| {
                                                    if creating_training() {
                                                        return;
                                                    }

                                                    let team_id = match training_scope() {
                                                        TrainingScopeSelection::WholeGroup => None,
                                                        TrainingScopeSelection::ActiveTeam => {
                                                            let Some(team_id) = selected_team() else {
                                                                status.set(Some((false, "Wähle zuerst eine aktive Mannschaft aus.".to_string())));
                                                                return;
                                                            };
                                                            Some(team_id)
                                                        }
                                                    };

                                                    status.set(None);
                                                    let input = CreateTrainingSessionInput {
                                                        club_id: group.club_id,
                                                        group_id,
                                                        team_id,
                                                        title: training_title(),
                                                        description: training_description(),
                                                        location: training_location(),
                                                        start_at: training_start_at(),
                                                        end_at: training_end_at(),
                                                    };

                                                    spawn(async move {
                                                        creating_training.set(true);
                                                        let result = create_training_session(input).await;
                                                        creating_training.set(false);

                                                        match result {
                                                            Ok(created_training) => {
                                                                training_title.set(String::new());
                                                                training_description.set(String::new());
                                                                training_location.set(String::new());
                                                                training_start_at.set(String::new());
                                                                training_end_at.set(String::new());
                                                                training_scope.set(TrainingScopeSelection::WholeGroup);
                                                                status.set(Some((true, format!("Training '{}' wurde angelegt.", created_training.title))));
                                                                refresh.with_mut(|value| *value += 1);
                                                            }
                                                            Err(error) => {
                                                                status.set(Some((false, format!("Training konnte nicht angelegt werden: {error}"))));
                                                            }
                                                        }
                                                    });
                                                },
                                                {if creating_training() { "Speichert..." } else { "Training anlegen" }}
                                            }
                                        }
                                    }
                                }

                                match training_state {
                                    None => rsx! {
                                        LoadingPanel { title: "4. Kommende Trainings".to_string(), lines: 4 }
                                    },
                                    Some(Err(error)) => rsx! {
                                        SectionPanel {
                                            title: "4. Kommende Trainings".to_string(),
                                            description: "Behalte die nächsten Termine der Gruppe im Blick.".to_string(),
                                            StatusBanner {
                                                tone: StatusBannerTone::Error,
                                                title: Some("Trainings konnten nicht geladen werden".to_string()),
                                                message: error.to_string(),
                                            }
                                        }
                                    },
                                    Some(Ok(trainings)) if trainings.is_empty() => rsx! {
                                        SectionPanel {
                                            title: "4. Kommende Trainings".to_string(),
                                            description: "Behalte die nächsten Termine der Gruppe im Blick.".to_string(),
                                            EmptyStatePanel {
                                                title: "Noch kein Training geplant".to_string(),
                                                message: "Für diese Gruppe wurden noch keine Trainings angelegt.".to_string(),
                                            }
                                        }
                                    },
                                    Some(Ok(trainings)) => {
                                        let training_count = trainings.len();

                                        rsx! {
                                            SectionPanel {
                                                title: "4. Kommende Trainings".to_string(),
                                                description: "Behalte die nächsten Termine der Gruppe im Blick.".to_string(),
                                                ItemGroup {
                                                    for (index, training) in trainings.into_iter().enumerate() {
                                                        Item {
                                                            class: "content-list-item",
                                                            ItemContent {
                                                                ItemTitle { "{training.title}" }
                                                                div { class: "detail-badges",
                                                                    Badge {
                                                                        variant: BadgeVariant::Secondary,
                                                                        "{training_scope_label(&training)}"
                                                                    }
                                                                }
                                                                div { class: "training-meta-stack",
                                                                    ItemDescription {
                                                                        "{format_training_range(training.start_at, training.end_at)}"
                                                                    }
                                                                    if !training.location.trim().is_empty() {
                                                                        ItemDescription { "Ort: {training.location}" }
                                                                    }
                                                                    if !training.description.trim().is_empty() {
                                                                        ItemDescription { "{training.description}" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        if index + 1 < training_count {
                                                            ItemSeparator {}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

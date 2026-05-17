use crate::club_memberships::{assign_player_to_team, list_unassigned_club_members, PlayerAssignmentInput};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::input::Input;
use crate::components::ui::item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemGroup, ItemSeparator, ItemTitle,
};
use crate::components::ui::label::Label;
use crate::components::ui::textarea::Textarea;
use crate::dashboard::get_dashboard_context;
use crate::teams::list_teams_for_group;
use crate::training::{
    create_training_session, format_training_range, list_group_training_sessions,
    training_scope_label, CreateTrainingSessionInput,
};
use dioxus::prelude::*;

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
    let mut training_team_id = use_signal(|| None::<i32>);
    let mut creating_training = use_signal(|| false);

    match context_state {
        None => rsx! {
            section { id: "home-intro",
                div { class: "auth-status",
                    p { class: "auth-help", "Gruppenbereich wird geladen..." }
                }
            }
        },
        Some(Err(error)) => rsx! {
            section { id: "home-intro",
                Card { class: "home-intro-card",
                    CardHeader {
                        Badge { variant: BadgeVariant::Destructive, "Fehler" }
                        CardTitle { "Gruppenbereich konnte nicht geladen werden" }
                        CardDescription { "{error}" }
                    }
                }
            }
        },
        Some(Ok(context)) => {
            let Some(group) = context.managed_groups.iter().find(|group| group.group_id == group_id).cloned() else {
                return rsx! {
                    section { id: "home-intro",
                        Card { class: "home-intro-card",
                            CardHeader {
                                Badge { variant: BadgeVariant::Destructive, "Kein Zugriff" }
                                CardTitle { "Gruppe nicht verfuegbar" }
                                CardDescription {
                                    "Du kannst nur Gruppen oeffnen, die dir als Trainer zugewiesen sind."
                                }
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

            rsx! {
                section { id: "home-intro",
                    Card { class: "home-intro-card",
                        CardHeader {
                            div { class: "home-badges",
                                Badge { "Trainerbereich" }
                                Badge { variant: BadgeVariant::Outline, "{group.club_name}" }
                            }
                            CardTitle { "{group.group_name}" }
                            CardDescription {
                                "Weise registrierte Vereinsmitglieder einer Mannschaft deiner Gruppe zu."
                            }
                        }
                        CardContent {
                            match teams_state {
                                None => rsx! { p { class: "auth-help", "Mannschaften werden geladen..." } },
                                Some(Err(error)) => rsx! {
                                    div { class: "auth-status",
                                        Badge { variant: BadgeVariant::Destructive, "Fehler" }
                                        p { class: "auth-help", "Mannschaften konnten nicht geladen werden: {error}" }
                                    }
                                },
                                Some(Ok(teams)) if teams.is_empty() => rsx! {
                                    p { class: "auth-help", "Es wurden noch keine Mannschaften in dieser Gruppe angelegt." }
                                },
                                Some(Ok(teams)) => {
                                    let team_count = teams.len();

                                    rsx! {
                                    ItemGroup {
                                        for (index, team) in teams.into_iter().enumerate() {
                                            Item {
                                                ItemContent {
                                                    ItemTitle { "{team.name}" }
                                                    ItemDescription { "Sortierung: {team.sort_order}" }
                                                }
                                                ItemActions {
                                                    Button {
                                                        variant: if selected_team() == Some(team.id) {
                                                            ButtonVariant::Secondary
                                                        } else {
                                                            ButtonVariant::Outline
                                                        },
                                                        onclick: move |_| selected_team.set(Some(team.id)),
                                                        {if selected_team() == Some(team.id) { "Ausgewaehlt" } else { "Auswaehlen" }}
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
                }

                section { id: "home-intro",
                    Card { class: "home-intro-card",
                        CardHeader {
                            CardTitle { "Trainingsplanung" }
                            CardDescription {
                                "Plane hier gruppenweite oder mannschaftsspezifische Trainings fuer diese Gruppe."
                            }
                        }
                        CardContent {
                            div { style: "display: grid; gap: 0.75rem; margin-bottom: 1rem;",
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
                                div { style: "display: grid; gap: 0.75rem; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));",
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
                                    Label { html_for: "training-team-id", "Mannschaft optional" }
                                    Input {
                                        id: "training-team-id",
                                        value: training_team_id().map(|team_id| team_id.to_string()).unwrap_or_default(),
                                        placeholder: "Leer lassen fuer ganze Gruppe oder Team-ID eintragen",
                                        disabled: creating_training(),
                                        oninput: move |event: FormEvent| {
                                            let value = event.value();
                                            let value = value.trim().to_string();
                                            if value.is_empty() {
                                                training_team_id.set(None);
                                            } else if let Ok(team_id) = value.parse::<i32>() {
                                                training_team_id.set(Some(team_id));
                                            }
                                        },
                                    }
                                }
                                div { class: "auth-field",
                                    Label { html_for: "training-description", "Beschreibung" }
                                    Textarea {
                                        id: "training-description",
                                        value: training_description(),
                                        rows: "4",
                                        placeholder: "Fokus, Ablauf oder Hinweise fuer das Training",
                                        disabled: creating_training(),
                                        oninput: move |event: FormEvent| training_description.set(event.value()),
                                    }
                                }
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    disabled: creating_training(),
                                    onclick: move |_| {
                                        if creating_training() {
                                            return;
                                        }

                                        status.set(None);
                                        let input = CreateTrainingSessionInput {
                                            club_id: group.club_id,
                                            group_id,
                                            team_id: training_team_id(),
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
                                                    training_team_id.set(None);
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

                            match training_state {
                                None => rsx! { p { class: "auth-help", "Trainings werden geladen..." } },
                                Some(Err(error)) => rsx! {
                                    div { class: "auth-status",
                                        Badge { variant: BadgeVariant::Destructive, "Fehler" }
                                        p { class: "auth-help", "Trainings konnten nicht geladen werden: {error}" }
                                    }
                                },
                                Some(Ok(trainings)) if trainings.is_empty() => rsx! {
                                    p { class: "auth-help", "Fuer diese Gruppe wurden noch keine Trainings geplant." }
                                },
                                Some(Ok(trainings)) => {
                                    let training_count = trainings.len();

                                    rsx! {
                                        ItemGroup {
                                            for (index, training) in trainings.into_iter().enumerate() {
                                                Item {
                                                    ItemContent {
                                                        ItemTitle { "{training.title}" }
                                                        ItemDescription {
                                                            "{training_scope_label(&training)} | {format_training_range(training.start_at, training.end_at)}"
                                                        }
                                                        if !training.location.trim().is_empty() {
                                                            ItemDescription { "Ort: {training.location}" }
                                                        }
                                                        if !training.description.trim().is_empty() {
                                                            ItemDescription { "{training.description}" }
                                                        }
                                                    }
                                                    ItemActions {
                                                        Badge {
                                                            variant: BadgeVariant::Secondary,
                                                            "{training.status}"
                                                        }
                                                    }
                                                }
                                                if index + 1 < training_count {
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

                section { id: "home-intro",
                    Card { class: "home-intro-card",
                        CardHeader {
                            CardTitle { "Spieler ohne Mannschaft" }
                            CardDescription {
                                "Diese Benutzer sind bereits im Verein registriert, aber noch keinem Team zugewiesen."
                            }
                        }
                        CardContent {
                            match members_state {
                                None => rsx! { p { class: "auth-help", "Vereinsmitglieder werden geladen..." } },
                                Some(Err(error)) => rsx! {
                                    div { class: "auth-status",
                                        Badge { variant: BadgeVariant::Destructive, "Fehler" }
                                        p { class: "auth-help", "Vereinsmitglieder konnten nicht geladen werden: {error}" }
                                    }
                                },
                                Some(Ok(members)) if members.is_empty() => rsx! {
                                    p { class: "auth-help", "Aktuell warten keine Spieler auf eine Mannschaftszuteilung." }
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
                                                                        status.set(Some((false, "Waehle zuerst eine Mannschaft aus.".to_string())));
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
                                                                                status.set(Some((true, format!("{} wurde einer Mannschaft zugewiesen.", success_name))));
                                                                                refresh.with_mut(|value| *value += 1);
                                                                            }
                                                                            Err(error) => {
                                                                                status.set(Some((false, format!("Spieler konnte nicht zugewiesen werden: {error}"))));
                                                                            }
                                                                        }
                                                                    });
                                                                },
                                                                {if assigning() == Some(member_user_id) { "Zuweisung..." } else { "Zu Mannschaft zuweisen" }}
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

                            if let Some((success, message)) = status() {
                                div { class: "auth-status",
                                    Badge {
                                        variant: if success { BadgeVariant::Secondary } else { BadgeVariant::Destructive },
                                        {if success { "Status" } else { "Fehler" }}
                                    }
                                    p { class: "auth-help", "{message}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

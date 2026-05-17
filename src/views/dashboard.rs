use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemGroup, ItemSeparator, ItemTitle,
};
use crate::dashboard::get_dashboard_context;
use crate::training::{format_training_range, list_my_training_sessions, training_scope_label};
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Dashboard() -> Element {
    let context_resource = use_server_future(move || async move { get_dashboard_context().await })?;
    let training_resource = use_server_future(move || async move { list_my_training_sessions().await })?;
    let context_state = context_resource.read().as_ref().cloned();
    let training_state = training_resource.read().as_ref().cloned();
    let nav = navigator();

    match context_state {
        None => rsx! {
            section { id: "home-intro",
                div { class: "auth-status",
                    p { class: "auth-help", "Dashboard wird geladen..." }
                }
            }
        },
        Some(Err(error)) => rsx! {
            section { id: "home-intro",
                Card { class: "home-intro-card",
                    CardHeader {
                        Badge { variant: BadgeVariant::Destructive, "Fehler" }
                        CardTitle { "Dashboard konnte nicht geladen werden" }
                        CardDescription { "{error}" }
                    }
                }
            }
        },
        Some(Ok(context)) => {
            let show_waiting = context.teams.is_empty() && !context.awaiting_assignment_clubs.is_empty();
            let managed_groups = context.managed_groups.clone();
            let team_count = context.teams.len();
            let waiting_clubs = context.awaiting_assignment_clubs.clone();
            let assigned_teams = context.teams.clone();

            rsx! {
                section { id: "home-intro",
                    Card { class: "home-intro-card",
                        CardHeader {
                            div { class: "home-badges",
                                Badge { "Dashboard" }
                                if context.user.is_system_admin {
                                    Badge { variant: BadgeVariant::Secondary, "System-Admin" }
                                }
                                if !managed_groups.is_empty() {
                                    Badge { variant: BadgeVariant::Outline, "Trainer" }
                                }
                                if team_count > 0 {
                                    Badge { variant: BadgeVariant::Outline, "Spieler" }
                                }
                            }
                            CardTitle { "Willkommen, {context.user.username}" }
                            CardDescription {
                                "Dieses Dashboard zeigt dir nur die Bereiche und Aufgaben, die zu deiner aktuellen Rolle passen."
                            }
                        }
                        CardContent {
                            if context.user.is_system_admin {
                                RoleCard {
                                    title: "System-Admin",
                                    description: "Verwalte Vereine, Einladungen und offene Zuweisungen zentral aus einem Bereich.",
                                    primary_action: Some(("Vereine".to_string(), Route::Clubs {})),
                                    items: vec![
                                        format!("{} Vereine sichtbar", context.member_clubs.len().max(1)),
                                        format!("{} Trainergruppen im System", context.managed_groups.len()),
                                        format!("{} aktive Mannschaftszuweisungen fuer dich", context.teams.len()),
                                    ]
                                }
                            }

                            if !managed_groups.is_empty() {
                                Card { class: "home-intro-card", style: "margin-top: 1rem;",
                                    CardHeader {
                                        CardTitle { "Meine Gruppen" }
                                        CardDescription {
                                            "Als Trainer siehst du hier nur deine eigenen Gruppen."
                                        }
                                    }
                                    CardContent {
                                        ItemGroup {
                                            for (index, group) in managed_groups.into_iter().enumerate() {
                                                Item {
                                                    ItemContent {
                                                        ItemTitle { "{group.group_name}" }
                                                        ItemDescription { "{group.club_name}" }
                                                    }
                                                    ItemActions {
                                                        Button {
                                                            variant: ButtonVariant::Outline,
                                                            onclick: move |_| {
                                                                let _ = nav.push(Route::GroupDetail { group_id: group.group_id });
                                                            },
                                                            "Oeffnen"
                                                        }
                                                    }
                                                }
                                                if index + 1 < context.managed_groups.len() {
                                                    ItemSeparator {}
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if show_waiting {
                                Card { class: "home-intro-card", style: "margin-top: 1rem;",
                                    CardHeader {
                                        Badge { variant: BadgeVariant::Outline, "Wartestatus" }
                                        CardTitle { "Du bist registriert" }
                                        CardDescription {
                                            "Dein Konto ist aktiv, aber du wurdest noch keiner Mannschaft zugewiesen."
                                        }
                                    }
                                    CardContent {
                                        for (_, club_name) in waiting_clubs {
                                            p { class: "auth-help", "Warte auf Mannschaftszuteilung in {club_name}." }
                                        }
                                    }
                                }
                            }

                            if team_count > 0 {
                                Card { class: "home-intro-card", style: "margin-top: 1rem;",
                                    CardHeader {
                                        CardTitle { "Meine Mannschaften" }
                                        CardDescription {
                                            "Diese Mannschaften sind dir aktuell zugewiesen."
                                        }
                                    }
                                    CardContent {
                                        ItemGroup {
                                            for (index, team) in assigned_teams.into_iter().enumerate() {
                                                Item {
                                                    ItemContent {
                                                        ItemTitle { "{team.name}" }
                                                        ItemDescription { "Mannschaft #{team.id}" }
                                                    }
                                                    ItemActions {
                                                        Badge { variant: BadgeVariant::Secondary, "Aktiv" }
                                                    }
                                                }
                                                if index + 1 < team_count {
                                                    ItemSeparator {}
                                                }
                                            }
                                        }
                                    }
                                }

                                Card { class: "home-intro-card", style: "margin-top: 1rem;",
                                    CardHeader {
                                        CardTitle { "Meine kommenden Trainings" }
                                        CardDescription {
                                            "Du siehst hier gruppenweite und mannschaftsspezifische Trainings, die fuer dich relevant sind."
                                        }
                                    }
                                    CardContent {
                                        match training_state {
                                            None => rsx! { p { class: "auth-help", "Trainings werden geladen..." } },
                                            Some(Err(error)) => rsx! {
                                                div { class: "auth-status",
                                                    Badge { variant: BadgeVariant::Destructive, "Fehler" }
                                                    p { class: "auth-help", "Trainings konnten nicht geladen werden: {error}" }
                                                }
                                            },
                                            Some(Ok(trainings)) if trainings.is_empty() => rsx! {
                                                p { class: "auth-help", "Aktuell sind keine relevanten Trainings fuer dich geplant." }
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
                                                                        "{training.club_name} | {training_scope_label(&training)}"
                                                                    }
                                                                    ItemDescription {
                                                                        "{format_training_range(training.start_at, training.end_at)}"
                                                                    }
                                                                    if !training.location.trim().is_empty() {
                                                                        ItemDescription { "Ort: {training.location}" }
                                                                    }
                                                                }
                                                                ItemActions {
                                                                    Badge { variant: BadgeVariant::Secondary, "{training.status}" }
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
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RoleCard(
    title: String,
    description: String,
    items: Vec<String>,
    primary_action: Option<(String, Route)>,
) -> Element {
    let nav = navigator();

    rsx! {
        Card { class: "home-intro-card", style: "margin-top: 1rem;",
            CardHeader {
                CardTitle { "{title}" }
                CardDescription { "{description}" }
            }
            CardContent {
                ItemGroup {
                    for (index, item) in items.iter().enumerate() {
                        Item {
                            ItemContent {
                                ItemTitle { "{item}" }
                            }
                        }
                        if index + 1 < items.len() {
                            ItemSeparator {}
                        }
                    }
                }
                if let Some((label, route)) = primary_action {
                    div { style: "margin-top: 1rem;",
                        Button {
                            variant: ButtonVariant::Secondary,
                            onclick: move |_| {
                                let _ = nav.push(route.clone());
                            },
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}

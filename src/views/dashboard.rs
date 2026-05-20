use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::item::{Item, ItemActions, ItemContent, ItemDescription, ItemGroup, ItemSeparator, ItemTitle};
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
            section { class: "page-section",
                div { class: "auth-status",
                    p { class: "auth-help", "Dashboard wird geladen..." }
                }
            }
        },
        Some(Err(error)) => rsx! {
            section { class: "page-section",
                Card { class: "home-intro-card",
                    CardHeader {
                        CardTitle { "Dashboard konnte nicht geladen werden" }
                        CardDescription { "{error}" }
                    }
                }
            }
        },
        Some(Ok(context)) => {
            let managed_groups = context.managed_groups.clone();
            let managed_group_count = managed_groups.len();
            let assigned_teams = context.teams.clone();
            let team_count = assigned_teams.len();
            let waiting_clubs = context.awaiting_assignment_clubs.clone();
            let show_waiting = team_count == 0 && !waiting_clubs.is_empty();

            rsx! {
                section { class: "page-section",
                    div { class: "page-stack",
                        Card { class: "home-intro-card",
                            CardHeader {
                                CardTitle { "Willkommen, {context.user.username}" }
                                CardDescription {
                                    {
                                        let mut lines = Vec::new();
                                        if context.user.is_system_admin {
                                            lines.push("Du verwaltest Vereine und Zuweisungen.".to_string());
                                        }
                                        if managed_group_count > 0 {
                                            lines.push(format!("{} Gruppen brauchen aktuell deine Aufmerksamkeit.", managed_group_count));
                                        }
                                        if team_count > 0 {
                                            lines.push(format!("{} Mannschaften sind dir zugeordnet.", team_count));
                                        }
                                        if lines.is_empty() {
                                            "Hier siehst du nur die Bereiche, die für deine aktuelle Rolle wichtig sind.".to_string()
                                        } else {
                                            lines.join(" ")
                                        }
                                    }
                                }
                            }
                            CardContent {
                                if context.user.is_system_admin {
                                    div { class: "section-actions",
                                        Button {
                                            variant: ButtonVariant::Secondary,
                                            onclick: move |_| {
                                                let _ = nav.push(Route::Clubs {});
                                            },
                                            "Vereine verwalten"
                                        }
                                    }
                                }
                            }
                        }

                        if managed_group_count > 0 {
                            Card { class: "home-intro-card",
                                CardHeader {
                                    CardTitle { "Meine Gruppen" }
                                    CardDescription {
                                        "Öffne die Gruppen, die du aktuell betreust."
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
                                                        "Öffnen"
                                                    }
                                                }
                                            }
                                            if index + 1 < managed_group_count {
                                                ItemSeparator {}
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if show_waiting {
                            Card { class: "home-intro-card",
                                CardHeader {
                                    CardTitle { "Warte auf Zuweisung" }
                                    CardDescription {
                                        "Dein Konto ist aktiv. Für diese Vereine fehlt noch die Mannschaftszuteilung."
                                    }
                                }
                                CardContent {
                                    div { class: "detail-list",
                                        for (_, club_name) in waiting_clubs {
                                            div { class: "detail-row",
                                                div { class: "detail-row-copy",
                                                    span { class: "detail-row-title", "{club_name}" }
                                                    p { class: "detail-row-meta", "Du kannst nach der Zuteilung direkt weiterarbeiten." }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if team_count > 0 {
                            Card { class: "home-intro-card",
                                CardHeader {
                                    CardTitle { "Meine Mannschaften" }
                                    CardDescription {
                                        "Diese Mannschaften sind dir aktuell zugeordnet."
                                    }
                                }
                                CardContent {
                                    ItemGroup {
                                        for (index, team) in assigned_teams.into_iter().enumerate() {
                                            Item {
                                                ItemContent {
                                                    ItemTitle { "{team.name}" }
                                                }
                                            }
                                            if index + 1 < team_count {
                                                ItemSeparator {}
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        Card { class: "home-intro-card",
                            CardHeader {
                                CardTitle { "Kommende Trainings" }
                                CardDescription {
                                    "Die nächsten relevanten Termine für deine Gruppen und Mannschaften."
                                }
                            }
                            CardContent {
                                match training_state {
                                    None => rsx! { p { class: "auth-help", "Trainings werden geladen..." } },
                                    Some(Err(error)) => rsx! {
                                        div { class: "auth-status auth-status--error",
                                            p { class: "auth-help", "Trainings konnten nicht geladen werden: {error}" }
                                        }
                                    },
                                    Some(Ok(trainings)) if trainings.is_empty() => rsx! {
                                        p { class: "auth-help", "Aktuell sind keine relevanten Trainings für dich geplant." }
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

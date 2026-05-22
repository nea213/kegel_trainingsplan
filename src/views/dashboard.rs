use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemGroup, ItemSeparator, ItemTitle,
};
use crate::components::{
    EmptyStatePanel, LoadingPanel, MetricCard, PageHeader, SectionPanel, StatusBanner,
    StatusBannerTone,
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
            section { class: "page-section",
                div { class: "page-stack",
                    LoadingPanel { title: "Dashboard".to_string(), lines: 4 }
                }
            }
        },
        Some(Err(error)) => rsx! {
            section { class: "page-section",
                div { class: "page-stack",
                    PageHeader {
                        title: "Dashboard".to_string(),
                        description: "Die persönliche Startseite konnte nicht geladen werden.".to_string(),
                        eyebrow: Some("Arbeitsbereich".to_string()),
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
            let managed_groups = context.managed_groups.clone();
            let managed_group_count = managed_groups.len();
            let assigned_teams = context.teams.clone();
            let team_count = assigned_teams.len();
            let waiting_clubs = context.awaiting_assignment_clubs.clone();
            let waiting_count = waiting_clubs.len();
            let upcoming_training_count = match &training_state {
                Some(Ok(trainings)) => trainings.len(),
                _ => 0,
            };
            let welcome_description = {
                let mut lines = Vec::new();
                if context.user.is_system_admin {
                    lines.push("Du verwaltest Vereine und Zuweisungen.".to_string());
                }
                if managed_group_count > 0 {
                    lines.push(format!(
                        "{} Gruppen brauchen aktuell deine Aufmerksamkeit.",
                        managed_group_count
                    ));
                }
                if team_count > 0 {
                    lines.push(format!("{} Mannschaften sind dir zugeordnet.", team_count));
                }
                if lines.is_empty() {
                    "Hier siehst du nur die Bereiche, die für deine aktuelle Rolle wichtig sind."
                        .to_string()
                } else {
                    lines.join(" ")
                }
            };

            rsx! {
                section { class: "page-section",
                    div { class: "page-stack page-stack--spacious",
                        PageHeader {
                            title: format!("Willkommen, {}", context.user.username),
                            description: welcome_description,
                            eyebrow: Some("Dashboard".to_string()),
                            actions: if context.user.is_system_admin {
                                Some(rsx! {
                                    Button {
                                        variant: ButtonVariant::Secondary,
                                        onclick: move |_| {
                                            let _ = nav.push(Route::Clubs {});
                                        },
                                        "Vereine verwalten"
                                    }
                                })
                            } else {
                                None
                            },
                        }

                        div { class: "metrics-grid",
                            MetricCard {
                                label: "Betreute Gruppen".to_string(),
                                value: managed_group_count.to_string(),
                                detail: Some("Direkt aus deinem Trainerbereich.".to_string()),
                            }
                            MetricCard {
                                label: "Zugewiesene Mannschaften".to_string(),
                                value: team_count.to_string(),
                                detail: Some("Aktuell für dich freigeschaltet.".to_string()),
                            }
                            MetricCard {
                                label: "Wartende Vereine".to_string(),
                                value: waiting_count.to_string(),
                                detail: Some("Dort fehlt noch eine Mannschaftszuteilung.".to_string()),
                            }
                            MetricCard {
                                label: "Kommende Trainings".to_string(),
                                value: upcoming_training_count.to_string(),
                                detail: Some("Deine nächsten relevanten Termine.".to_string()),
                            }
                        }

                        if managed_group_count > 0 {
                            SectionPanel {
                                title: "Meine Gruppen".to_string(),
                                description: "Öffne die Gruppen, die du aktuell betreust.".to_string(),
                                ItemGroup {
                                    for (index, group) in managed_groups.into_iter().enumerate() {
                                        Item {
                                            class: "content-list-item",
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
                        } else {
                            SectionPanel {
                                title: "Meine Gruppen".to_string(),
                                description: "Hier erscheinen alle Gruppen, die du betreust.".to_string(),
                                EmptyStatePanel {
                                    title: "Noch keine Gruppe verfügbar".to_string(),
                                    message: "Sobald dir eine Gruppe zugewiesen wurde, kannst du hier direkt in deinen Arbeitsbereich springen.".to_string(),
                                }
                            }
                        }

                        if team_count > 0 {
                            SectionPanel {
                                title: "Meine Mannschaften".to_string(),
                                description: "Diese Mannschaften sind dir aktuell zugeordnet.".to_string(),
                                ItemGroup {
                                    for (index, team) in assigned_teams.into_iter().enumerate() {
                                        Item {
                                            class: "content-list-item",
                                            ItemContent {
                                                ItemTitle { "{team.name}" }
                                                ItemDescription { "Für Training und Organisation verfügbar" }
                                            }
                                        }
                                        if index + 1 < team_count {
                                            ItemSeparator {}
                                        }
                                    }
                                }
                            }
                        } else if waiting_count > 0 {
                            SectionPanel {
                                title: "Warte auf Zuweisung".to_string(),
                                description: "Dein Konto ist aktiv. Für diese Vereine fehlt noch die Mannschaftszuteilung.".to_string(),
                                div { class: "state-list",
                                    for (_, club_name) in waiting_clubs {
                                        div { class: "detail-row",
                                            div { class: "detail-row-copy",
                                                span { class: "detail-row-title", "{club_name}" }
                                                p { class: "detail-row-meta", "Nach der Zuteilung kannst du direkt weiterarbeiten." }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        match training_state {
                            None => rsx! {
                                LoadingPanel {
                                    title: "Kommende Trainings".to_string(),
                                    lines: 4,
                                }
                            },
                            Some(Err(error)) => rsx! {
                                SectionPanel {
                                    title: "Kommende Trainings".to_string(),
                                    description: "Die nächsten relevanten Termine für deine Gruppen und Mannschaften.".to_string(),
                                    StatusBanner {
                                        tone: StatusBannerTone::Error,
                                        title: Some("Trainings konnten nicht geladen werden".to_string()),
                                        message: error.to_string(),
                                    }
                                }
                            },
                            Some(Ok(trainings)) if trainings.is_empty() => rsx! {
                                SectionPanel {
                                    title: "Kommende Trainings".to_string(),
                                    description: "Die nächsten relevanten Termine für deine Gruppen und Mannschaften.".to_string(),
                                    EmptyStatePanel {
                                        title: "Keine Trainings geplant".to_string(),
                                        message: "Aktuell sind für dich keine relevanten Trainings angelegt.".to_string(),
                                    }
                                }
                            },
                            Some(Ok(trainings)) => {
                                let training_count = trainings.len();

                                rsx! {
                                    SectionPanel {
                                        title: "Kommende Trainings".to_string(),
                                        description: "Die nächsten relevanten Termine für deine Gruppen und Mannschaften.".to_string(),
                                        ItemGroup {
                                            for (index, training) in trainings.into_iter().enumerate() {
                                                Item {
                                                    class: "content-list-item",
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
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

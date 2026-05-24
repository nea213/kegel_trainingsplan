use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemGroup, ItemSeparator, ItemTitle,
};
use crate::components::{
    EmptyStatePanel, LoadingPanel, MetricCard, PageHeader, SectionPanel, StatusBanner,
    StatusBannerTone,
};
use crate::dashboard::get_dashboard_context;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Dashboard() -> Element {
    let context_resource = use_server_future(move || async move { get_dashboard_context().await })?;
    let context_state = context_resource.read().as_ref().cloned();
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
            let primary_title = if context.user.is_system_admin {
                "Verwalten, zuweisen und Überblick behalten"
            } else {
                "Dein Trainingsalltag auf einen Blick"
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
                            title: format!("{primary_title}, {}", context.user.username),
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

                        div { class: "dashboard-highlight-row",
                            div { class: "dashboard-highlight-card surface-card",
                                div { class: "dashboard-highlight-card__header",
                                    p { class: "metric-card__label", "Priorität heute" }
                                    Badge {
                                        variant: BadgeVariant::Outline,
                                        if context.user.is_system_admin {
                                            "Administration"
                                        } else {
                                            "Trainerbereich"
                                        }
                                    }
                                }
                                p { class: "dashboard-highlight-card__title",
                                    {
                                        if context.user.is_system_admin {
                                            if waiting_count > 0 {
                                                format!("{waiting_count} Vereine warten noch auf eine Mannschaftszuteilung.")
                                            } else {
                                                "Alle Vereine sind aktuell ohne offene Zuweisungsstufe.".to_string()
                                            }
                                        } else if managed_group_count > 0 {
                                            format!("{managed_group_count} Gruppen sind direkt für dich verfügbar.")
                                        } else if waiting_count > 0 {
                                            format!("{waiting_count} Vereine warten noch auf deine Zuteilung.")
                                        } else {
                                            "Sobald dir eine Gruppe zugewiesen wurde, erscheint sie hier zuerst.".to_string()
                                        }
                                    }
                                }
                                p { class: "dashboard-highlight-card__copy",
                                    {
                                        if context.user.is_system_admin {
                                            "Öffne die Vereinsverwaltung, um Strukturen zu ergänzen, Codes zu erstellen oder offene Zuweisungen vorzubereiten."
                                        } else {
                                            "Starte mit deinen Gruppen, prüfe Mannschaften und organisiere danach die nächsten Schritte im Team."
                                        }
                                    }
                                }
                            }
                        }

                        div { class: "metrics-grid",
                            MetricCard {
                                label: "Betreute Gruppen".to_string(),
                                value: managed_group_count.to_string(),
                                detail: Some("Direkt aus deinem Arbeitsbereich erreichbar.".to_string()),
                            }
                            MetricCard {
                                label: "Zugewiesene Mannschaften".to_string(),
                                value: team_count.to_string(),
                                detail: Some("Aktuell für Organisation und Betreuung aktiv.".to_string()),
                            }
                            MetricCard {
                                label: "Wartende Vereine".to_string(),
                                value: waiting_count.to_string(),
                                detail: Some("Dort fehlt noch eine nächste Verwaltungsaktion.".to_string()),
                            }
                        }

                        if managed_group_count > 0 {
                            SectionPanel {
                                title: "Meine Gruppen".to_string(),
                                description: "Hier beginnst du direkt mit deinem nächsten Arbeitsschritt in der Gruppe.".to_string(),
                                ItemGroup {
                                    for (index, group) in managed_groups.into_iter().enumerate() {
                                        Item {
                                            class: "content-list-item actionable-list-item",
                                            ItemContent {
                                                ItemTitle { "{group.group_name}" }
                                                ItemDescription { "{group.club_name}" }
                                                div { class: "detail-badges",
                                                    Badge { variant: BadgeVariant::Secondary, "Direkt öffnen" }
                                                }
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
                                    message: "Sobald dir eine Gruppe zugewiesen wurde, kannst du hier direkt mit Mannschaften und Zuweisungen starten.".to_string(),
                                }
                            }
                        }

                        if team_count > 0 {
                            SectionPanel {
                                title: "Meine Mannschaften".to_string(),
                                description: "Diese Mannschaften stehen dir aktuell für Organisation und Betreuung zur Verfügung.".to_string(),
                                ItemGroup {
                                    for (index, team) in assigned_teams.into_iter().enumerate() {
                                        Item {
                                            class: "content-list-item actionable-list-item",
                                            ItemContent {
                                                ItemTitle { "{team.name}" }
                                                ItemDescription { "Für Organisation und Betreuung verfügbar" }
                                                div { class: "detail-badges",
                                                    Badge { variant: BadgeVariant::Outline, "Mannschaft aktiv" }
                                                }
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
                                        div { class: "detail-row detail-row--soft",
                                            div { class: "detail-row-copy",
                                                span { class: "detail-row-title", "{club_name}" }
                                                p { class: "detail-row-meta", "Nach der Zuteilung kannst du direkt weiterarbeiten." }
                                            }
                                            Badge { variant: BadgeVariant::Outline, "Wartet" }
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

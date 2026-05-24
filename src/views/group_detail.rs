use crate::club_memberships::{
    assign_player_to_team, list_unassigned_club_members, PlayerAssignmentInput,
};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemGroup, ItemSeparator, ItemTitle,
};
use crate::components::{
    show_error_toast, show_success_toast, EmptyStatePanel, LoadingPanel, PageHeader,
    SectionPanel, StatusBanner, StatusBannerTone,
};
use crate::dashboard::get_dashboard_context;
use crate::teams::list_teams_for_group;
use dioxus::prelude::*;
use dioxus_primitives::toast::use_toast;

#[component]
pub fn GroupDetail(group_id: i32) -> Element {
    let mut refresh = use_signal(|| 0_u64);
    let context_resource = use_server_future(move || {
        let _ = refresh();
        async move { get_dashboard_context().await }
    })?;
    let toast = use_toast();
    let context_state = context_resource.read().as_ref().cloned();
    let mut selected_team = use_signal(|| None::<i32>);
    let mut assigning = use_signal(|| None::<i32>);

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
            let teams_state = teams_resource.read().as_ref().cloned();
            let members_state = members_resource.read().as_ref().cloned();
            let active_team_name = match &teams_state {
                Some(Ok(teams)) => teams
                    .iter()
                    .find(|team| Some(team.id) == selected_team())
                    .map(|team| team.name.clone()),
                _ => None,
            };

            rsx! {
                section { class: "page-section",
                    div { class: "page-stack page-stack--spacious",
                        PageHeader {
                            title: group.group_name.clone(),
                            description: "Arbeite Schritt für Schritt: Mannschaft festlegen und danach Spieler organisieren.".to_string(),
                            eyebrow: Some(group.club_name.clone()),
                        }
                        div { class: "workflow-summary-card surface-card",
                            div { class: "workflow-summary-card__header",
                                div { class: "workflow-summary-card__copy",
                                    p { class: "metric-card__label", "Aktiver Arbeitskontext" }
                                    p { class: "workflow-summary-card__title",
                                        {active_team_name.clone().unwrap_or_else(|| "Noch keine Mannschaft ausgewählt".to_string())}
                                    }
                                    p { class: "workflow-summary-card__text",
                                        {
                                            if active_team_name.is_some() {
                                                "Spielerzuweisungen orientieren sich jetzt an dieser Auswahl.".to_string()
                                            } else {
                                                "Wähle zuerst eine Mannschaft aus. Danach werden Zuweisungen freigeschaltet.".to_string()
                                            }
                                        }
                                    }
                                }
                                div { class: "detail-badges",
                                    Badge {
                                        variant: if active_team_name.is_some() {
                                            BadgeVariant::Secondary
                                        } else {
                                            BadgeVariant::Outline
                                        },
                                        {if active_team_name.is_some() { "Bereit" } else { "Schritt 1 offen" }}
                                    }
                                    Badge { variant: BadgeVariant::Outline, "{group.club_name}" }
                                }
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
                                                "Diese Auswahl wird für Spielerzuweisungen genutzt."
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
                                                        class: "team-selection-list",
                                                        for (index, team) in teams.into_iter().enumerate() {
                                                            Item {
                                                                class: if selected_team() == Some(team.id) {
                                                                    "content-list-item actionable-list-item team-selection-item team-selection-item--active"
                                                                } else {
                                                                    "content-list-item actionable-list-item team-selection-item"
                                                                },
                                                                ItemContent {
                                                                    ItemTitle { "{team.name}" }
                                                                    div { class: "detail-badges",
                                                                        Badge {
                                                                            variant: if selected_team() == Some(team.id) {
                                                                                BadgeVariant::Secondary
                                                                            } else {
                                                                                BadgeVariant::Outline
                                                                            },
                                                                            {if selected_team() == Some(team.id) {
                                                                                "Aktive Mannschaft".to_string()
                                                                            } else {
                                                                                "Auswählbar".to_string()
                                                                            }}
                                                                        }
                                                                    }
                                                                    ItemDescription {
                                                                        {if selected_team() == Some(team.id) {
                                                                            "Aktive Mannschaft".to_string()
                                                                        } else {
                                                                            "Für Spielerzuweisungen verfügbar".to_string()
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
                                        div { class: if active_team_name.is_some() {
                                            "detail-card detail-card-muted workflow-focus-card"
                                        } else {
                                            "detail-card detail-card-muted workflow-focus-card workflow-focus-card--pending"
                                        },
                                            p { class: "section-label", "Ziel für neue Zuweisungen" }
                                            p { class: "detail-card-title",
                                                {active_team_name.clone().unwrap_or_else(|| "Bitte zuerst eine Mannschaft auswählen".to_string())}
                                            }
                                            p { class: "section-meta",
                                                {
                                                    if active_team_name.is_some() {
                                                        "Neue Spieler werden direkt dieser Mannschaft zugeordnet.".to_string()
                                                    } else {
                                                        "Ohne aktive Mannschaft bleiben Zuweisungen gesperrt.".to_string()
                                                    }
                                                }
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
                                                                        class: "content-list-item actionable-list-item",
                                                                        ItemContent {
                                                                            ItemTitle { "{member.username}" }
                                                                            ItemDescription { "Mitglied in {member.club_name}" }
                                                                            if selected_team().is_none() {
                                                                                div { class: "detail-badges",
                                                                                    Badge { variant: BadgeVariant::Outline, "Mannschaft wählen" }
                                                                                }
                                                                            }
                                                                        }
                                                                        ItemActions {
                                                                            Button {
                                                                                variant: ButtonVariant::Outline,
                                                                                disabled: selected_team().is_none() || assigning() == Some(member_user_id),
                                                                                onclick: move |_| {
                                                                                    let Some(team_id) = selected_team() else {
                                                                                        show_error_toast(
                                                                                            toast,
                                                                                            "Mannschaft fehlt",
                                                                                            "Wähle zuerst eine aktive Mannschaft aus.",
                                                                                        );
                                                                                        return;
                                                                                    };

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
                                                                                                show_success_toast(
                                                                                                    toast,
                                                                                                    "Spieler zugewiesen",
                                                                                                    format!(
                                                                                                        "{} wurde der aktiven Mannschaft zugeordnet.",
                                                                                                        success_name
                                                                                                    ),
                                                                                                );
                                                                                                refresh.with_mut(|value| *value += 1);
                                                                                            }
                                                                                            Err(error) => {
                                                                                                show_error_toast(
                                                                                                    toast,
                                                                                                    "Spieler konnte nicht zugewiesen werden",
                                                                                                    error.to_string(),
                                                                                                );
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
                        }
                    }
                }
            }
        }
    }
}

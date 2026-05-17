use crate::club_memberships::{assign_player_to_team, list_unassigned_club_members, PlayerAssignmentInput};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemGroup, ItemSeparator, ItemTitle,
};
use crate::dashboard::get_dashboard_context;
use crate::teams::list_teams_for_group;
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
            let teams_state = teams_resource.read().as_ref().cloned();
            let members_state = members_resource.read().as_ref().cloned();

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

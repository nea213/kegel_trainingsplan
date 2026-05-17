use crate::auth::current_user;
use crate::clubs::{get_club_detail, ClubDetail as ClubDetailData};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::input::Input;
use crate::components::ui::item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemGroup, ItemSeparator, ItemTitle,
};
use crate::components::ui::label::Label;
use crate::groups::{create_group, CreateGroupInput};
use crate::teams::{create_team, CreateTeamInput};
use dioxus::prelude::*;

#[component]
pub fn ClubDetail(club_id: i32) -> Element {
    let mut refresh = use_signal(|| 0_u64);
    let user_resource = use_server_future(move || async move { current_user().await.ok().flatten() })?;
    let detail_resource = use_server_future(move || {
        let _ = refresh();
        async move { get_club_detail(club_id).await }
    })?;
    let user_state = user_resource.read().as_ref().cloned();
    let detail_state = detail_resource.read().as_ref().cloned();
    let mut group_name = use_signal(String::new);
    let mut group_sort_order = use_signal(|| "0".to_string());
    let mut team_names = use_signal(std::collections::HashMap::<i32, String>::new);
    let mut team_sort_orders = use_signal(std::collections::HashMap::<i32, String>::new);
    let mut busy_group = use_signal(|| false);
    let mut busy_team = use_signal(|| None::<i32>);
    let mut status = use_signal(|| None::<(bool, String)>);

    match user_state {
        None => rsx! {
            section { id: "home-intro",
                div { class: "auth-status",
                    p { class: "auth-help", "Berechtigungen werden geladen..." }
                }
            }
        },
        Some(None) => rsx! {},
        Some(Some(user)) if !user.is_system_admin => rsx! {
            section { id: "home-intro",
                Card { class: "home-intro-card",
                    CardHeader {
                        Badge { variant: BadgeVariant::Destructive, "Kein Zugriff" }
                        CardTitle { "Vereinsdetails" }
                        CardDescription {
                            "Nur System-Admins duerfen Vereine, Gruppen und Mannschaften verwalten."
                        }
                    }
                }
            }
        },
        Some(Some(_)) => rsx! {
            match detail_state {
                None => rsx! {
                    section { id: "home-intro",
                        div { class: "auth-status",
                            p { class: "auth-help", "Vereinsdetails werden geladen..." }
                        }
                    }
                },
                Some(Err(error)) => rsx! {
                    section { id: "home-intro",
                        Card { class: "home-intro-card",
                            CardHeader {
                                Badge { variant: BadgeVariant::Destructive, "Fehler" }
                                CardTitle { "Verein konnte nicht geladen werden" }
                                CardDescription { "{error}" }
                            }
                        }
                    }
                },
                Some(Ok(detail)) => rsx! {
                    section { id: "home-intro",
                        Card { class: "home-intro-card",
                            CardHeader {
                                div { class: "home-badges",
                                    Badge { "Verein" }
                                    Badge { variant: BadgeVariant::Outline, "#{detail.club.id}" }
                                }
                                CardTitle { "{detail.club.name}" }
                                CardDescription {
                                    "Lege Gruppen manuell an und ordne anschliessend Mannschaften innerhalb der passenden Gruppe zu."
                                }
                            }
                            CardContent {
                                div { style: "display: grid; gap: 0.75rem;",
                                    div { class: "auth-field",
                                        Label { html_for: "group-name", "Neue Gruppe" }
                                        Input {
                                            id: "group-name",
                                            value: group_name(),
                                            placeholder: "z. B. Maenner",
                                            disabled: busy_group(),
                                            oninput: move |event: FormEvent| group_name.set(event.value()),
                                        }
                                    }
                                    div { class: "auth-field",
                                        Label { html_for: "group-sort-order", "Sortierung" }
                                        Input {
                                            id: "group-sort-order",
                                            value: group_sort_order(),
                                            placeholder: "0",
                                            disabled: busy_group(),
                                            oninput: move |event: FormEvent| group_sort_order.set(event.value()),
                                        }
                                    }
                                    Button {
                                        variant: ButtonVariant::Secondary,
                                        disabled: busy_group(),
                                        onclick: move |_| {
                                            if busy_group() {
                                                return;
                                            }

                                            status.set(None);
                                            let sort_order = parse_sort_order(&group_sort_order());
                                            let name = group_name();
                                            spawn(async move {
                                                let sort_order = match sort_order {
                                                    Ok(sort_order) => sort_order,
                                                    Err(error) => {
                                                        status.set(Some((false, format!("Gruppe konnte nicht angelegt werden: {error}"))));
                                                        return;
                                                    }
                                                };

                                                busy_group.set(true);
                                                let result = create_group(CreateGroupInput {
                                                    club_id,
                                                    name,
                                                    sort_order,
                                                })
                                                .await;
                                                busy_group.set(false);

                                                match result {
                                                    Ok(created_group) => {
                                                        group_name.set(String::new());
                                                        group_sort_order.set("0".to_string());
                                                        status.set(Some((true, format!("Gruppe '{}' wurde angelegt.", created_group.name))));
                                                        refresh.with_mut(|value| *value += 1);
                                                    }
                                                    Err(error) => {
                                                        status.set(Some((false, format!("Gruppe konnte nicht angelegt werden: {error}"))));
                                                    }
                                                }
                                            });
                                        },
                                        {if busy_group() { "Speichert..." } else { "Gruppe anlegen" }}
                                    }
                                }
                            }
                        }
                    }

                    section { id: "home-intro",
                        Card { class: "home-intro-card",
                            CardHeader {
                                CardTitle { "Gruppen und Mannschaften" }
                                CardDescription {
                                    "Jede Mannschaft wird innerhalb genau einer Gruppe angelegt."
                                }
                            }
                            CardContent {
                                if detail.groups.is_empty() {
                                    p { class: "auth-help", "Es wurden noch keine Gruppen fuer diesen Verein angelegt." }
                                } else {
                                    GroupList {
                                        detail,
                                        team_names,
                                        team_sort_orders,
                                        busy_team,
                                        on_create_team: move |group_id| {
                                            if busy_team().is_some() {
                                                return;
                                            }

                                            status.set(None);
                                            let name = team_names().get(&group_id).cloned().unwrap_or_default();
                                            let sort_order = team_sort_orders().get(&group_id).cloned().unwrap_or_else(|| "0".to_string());
                                            spawn(async move {
                                                let sort_order = match parse_sort_order(&sort_order) {
                                                    Ok(sort_order) => sort_order,
                                                    Err(error) => {
                                                        status.set(Some((false, format!("Mannschaft konnte nicht angelegt werden: {error}"))));
                                                        return;
                                                    }
                                                };

                                                busy_team.set(Some(group_id));
                                                let result = create_team(CreateTeamInput {
                                                    club_id,
                                                    group_id,
                                                    name,
                                                    sort_order,
                                                })
                                                .await;
                                                busy_team.set(None);

                                                match result {
                                                    Ok(created_team) => {
                                                        team_names.with_mut(|entries| {
                                                            entries.insert(group_id, String::new());
                                                        });
                                                        team_sort_orders.with_mut(|entries| {
                                                            entries.insert(group_id, "0".to_string());
                                                        });
                                                        status.set(Some((true, format!("Mannschaft '{}' wurde angelegt.", created_team.name))));
                                                        refresh.with_mut(|value| *value += 1);
                                                    }
                                                    Err(error) => {
                                                        status.set(Some((false, format!("Mannschaft konnte nicht angelegt werden: {error}"))));
                                                    }
                                                }
                                            });
                                        },
                                    }
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
                },
            }
        },
    }
}

#[component]
fn GroupList(
    detail: ClubDetailData,
    team_names: Signal<std::collections::HashMap<i32, String>>,
    team_sort_orders: Signal<std::collections::HashMap<i32, String>>,
    busy_team: Signal<Option<i32>>,
    on_create_team: EventHandler<i32>,
) -> Element {
    let group_count = detail.groups.len();

    rsx! {
        ItemGroup {
            for (index, section) in detail.groups.into_iter().enumerate() {
                Item {
                    ItemContent {
                        ItemTitle { "{section.group.name}" }
                        ItemDescription { "Sortierung: {section.group.sort_order}" }
                        if section.teams.is_empty() {
                            p { class: "auth-help", "Noch keine Mannschaften angelegt." }
                        } else {
                            div { style: "display: grid; gap: 0.5rem; margin-top: 0.75rem;",
                                for team in section.teams {
                                    div { style: "display: flex; align-items: center; justify-content: space-between; gap: 1rem; padding: 0.65rem 0.8rem; border-radius: 0.75rem; background: var(--accent);",
                                        span { style: "font-weight: 600;", "{team.name}" }
                                        Badge { variant: BadgeVariant::Outline, "Sortierung {team.sort_order}" }
                                    }
                                }
                            }
                        }
                        div { style: "display: grid; gap: 0.75rem; margin-top: 1rem;",
                            div { class: "auth-field",
                                Label { html_for: format!("team-name-{}", section.group.id), "Neue Mannschaft" }
                                Input {
                                    id: format!("team-name-{}", section.group.id),
                                    value: team_names().get(&section.group.id).cloned().unwrap_or_default(),
                                    placeholder: "z. B. Maenner 1",
                                    disabled: busy_team() == Some(section.group.id),
                                    oninput: move |event: FormEvent| {
                                        team_names.with_mut(|entries| {
                                            entries.insert(section.group.id, event.value());
                                        });
                                    },
                                }
                            }
                            div { class: "auth-field",
                                Label { html_for: format!("team-sort-order-{}", section.group.id), "Sortierung" }
                                Input {
                                    id: format!("team-sort-order-{}", section.group.id),
                                    value: team_sort_orders().get(&section.group.id).cloned().unwrap_or_else(|| "0".to_string()),
                                    placeholder: "0",
                                    disabled: busy_team() == Some(section.group.id),
                                    oninput: move |event: FormEvent| {
                                        team_sort_orders.with_mut(|entries| {
                                            entries.insert(section.group.id, event.value());
                                        });
                                    },
                                }
                            }
                        }
                    }
                    ItemActions {
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: busy_team().is_some(),
                            onclick: move |_| on_create_team.call(section.group.id),
                            {if busy_team() == Some(section.group.id) { "Speichert..." } else { "Mannschaft anlegen" }}
                        }
                    }
                }
                if index + 1 < group_count {
                    ItemSeparator {}
                }
            }
        }
    }
}

fn parse_sort_order(value: &str) -> Result<i32, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(0);
    }

    trimmed
        .parse::<i32>()
        .map_err(|_| "Die Sortierung muss eine ganze Zahl sein.".to_string())
}

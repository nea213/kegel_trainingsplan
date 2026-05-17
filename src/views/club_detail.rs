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
use crate::group_trainers::{assign_group_trainer, remove_group_trainer, AssignGroupTrainerInput};
use crate::groups::{create_group, CreateGroupInput};
use crate::team_players::{assign_team_player, remove_team_player, AssignTeamPlayerInput};
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
    let mut trainer_names = use_signal(std::collections::HashMap::<i32, String>::new);
    let mut new_team_names = use_signal(std::collections::HashMap::<i32, String>::new);
    let mut player_names = use_signal(std::collections::HashMap::<i32, String>::new);
    let mut team_sort_orders = use_signal(std::collections::HashMap::<i32, String>::new);
    let mut busy_group = use_signal(|| false);
    let mut busy_trainer = use_signal(|| None::<i32>);
    let mut busy_team = use_signal(|| None::<i32>);
    let mut removing_trainer = use_signal(|| None::<(i32, i32)>);
    let mut removing_player = use_signal(|| None::<(i32, i32)>);
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
                                        trainer_names,
                                        new_team_names,
                                        player_names,
                                        team_sort_orders,
                                        busy_trainer,
                                        busy_team,
                                        removing_trainer,
                                        removing_player,
                                        on_assign_trainer: move |group_id| {
                                            if busy_trainer().is_some() {
                                                return;
                                            }

                                            status.set(None);
                                            let username = trainer_names().get(&group_id).cloned().unwrap_or_default();
                                            spawn(async move {
                                                busy_trainer.set(Some(group_id));
                                                let result = assign_group_trainer(AssignGroupTrainerInput {
                                                    group_id,
                                                    username,
                                                })
                                                .await;
                                                busy_trainer.set(None);

                                                match result {
                                                    Ok(trainer) => {
                                                        trainer_names.with_mut(|entries| {
                                                            entries.insert(group_id, String::new());
                                                        });
                                                        status.set(Some((true, format!("Trainer '{}' wurde zugewiesen.", trainer.username))));
                                                        refresh.with_mut(|value| *value += 1);
                                                    }
                                                    Err(error) => {
                                                        status.set(Some((false, format!("Trainer konnte nicht zugewiesen werden: {error}"))));
                                                    }
                                                }
                                            });
                                        },
                                        on_remove_trainer: move |(group_id, user_id)| {
                                            if removing_trainer().is_some() {
                                                return;
                                            }

                                            status.set(None);
                                            spawn(async move {
                                                removing_trainer.set(Some((group_id, user_id)));
                                                let result = remove_group_trainer(group_id, user_id).await;
                                                removing_trainer.set(None);

                                                match result {
                                                    Ok(()) => {
                                                        status.set(Some((true, "Trainer wurde entfernt.".to_string())));
                                                        refresh.with_mut(|value| *value += 1);
                                                    }
                                                    Err(error) => {
                                                        status.set(Some((false, format!("Trainer konnte nicht entfernt werden: {error}"))));
                                                    }
                                                }
                                            });
                                        },
                                        on_create_team: move |group_id| {
                                            if busy_team().is_some() {
                                                return;
                                            }

                                            status.set(None);
                                            let name = new_team_names().get(&group_id).cloned().unwrap_or_default();
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
                                                        new_team_names.with_mut(|entries| {
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
                                        on_assign_player: move |team_id| {
                                            if busy_team().is_some() {
                                                return;
                                            }

                                            status.set(None);
                                            let username = player_names().get(&team_id).cloned().unwrap_or_default();
                                            spawn(async move {
                                                busy_team.set(Some(team_id));
                                                let result = assign_team_player(AssignTeamPlayerInput {
                                                    team_id,
                                                    username,
                                                })
                                                .await;
                                                busy_team.set(None);

                                                match result {
                                                    Ok(player) => {
                                                        player_names.with_mut(|entries| {
                                                            entries.insert(team_id, String::new());
                                                        });
                                                        status.set(Some((true, format!("Spieler '{}' wurde zugewiesen.", player.username))));
                                                        refresh.with_mut(|value| *value += 1);
                                                    }
                                                    Err(error) => {
                                                        status.set(Some((false, format!("Spieler konnte nicht zugewiesen werden: {error}"))));
                                                    }
                                                }
                                            });
                                        },
                                        on_remove_player: move |(team_id, user_id)| {
                                            if removing_player().is_some() {
                                                return;
                                            }

                                            status.set(None);
                                            spawn(async move {
                                                removing_player.set(Some((team_id, user_id)));
                                                let result = remove_team_player(team_id, user_id).await;
                                                removing_player.set(None);

                                                match result {
                                                    Ok(()) => {
                                                        status.set(Some((true, "Spieler wurde entfernt.".to_string())));
                                                        refresh.with_mut(|value| *value += 1);
                                                    }
                                                    Err(error) => {
                                                        status.set(Some((false, format!("Spieler konnte nicht entfernt werden: {error}"))));
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
    trainer_names: Signal<std::collections::HashMap<i32, String>>,
    new_team_names: Signal<std::collections::HashMap<i32, String>>,
    player_names: Signal<std::collections::HashMap<i32, String>>,
    team_sort_orders: Signal<std::collections::HashMap<i32, String>>,
    busy_trainer: Signal<Option<i32>>,
    busy_team: Signal<Option<i32>>,
    removing_trainer: Signal<Option<(i32, i32)>>,
    removing_player: Signal<Option<(i32, i32)>>,
    on_assign_trainer: EventHandler<i32>,
    on_remove_trainer: EventHandler<(i32, i32)>,
    on_create_team: EventHandler<i32>,
    on_assign_player: EventHandler<i32>,
    on_remove_player: EventHandler<(i32, i32)>,
) -> Element {
    let group_count = detail.groups.len();

    rsx! {
        ItemGroup {
            for (index, section) in detail.groups.into_iter().enumerate() {
                Item {
                    ItemContent {
                        ItemTitle { "{section.group.name}" }
                        ItemDescription { "Sortierung: {section.group.sort_order}" }
                        div { style: "display: grid; gap: 0.5rem; margin-top: 0.75rem;",
                            p { class: "auth-help", "Trainer" }
                            if section.trainers.is_empty() {
                                p { class: "auth-help", "Noch keine Trainer zugewiesen." }
                            } else {
                                for trainer in section.trainers {
                                    div { style: "display: flex; align-items: center; justify-content: space-between; gap: 1rem; padding: 0.65rem 0.8rem; border-radius: 0.75rem; background: var(--accent);",
                                        span { style: "font-weight: 600;", "{trainer.username}" }
                                        Button {
                                            variant: ButtonVariant::Ghost,
                                            disabled: removing_trainer() == Some((section.group.id, trainer.user_id)),
                                            onclick: move |_| on_remove_trainer.call((section.group.id, trainer.user_id)),
                                            {if removing_trainer() == Some((section.group.id, trainer.user_id)) { "Entfernt..." } else { "Entfernen" }}
                                        }
                                    }
                                }
                            }
                        }
                        div { style: "display: grid; gap: 0.75rem; margin-top: 1rem;",
                            div { class: "auth-field",
                                Label { html_for: format!("trainer-name-{}", section.group.id), "Trainer per Benutzername" }
                                Input {
                                    id: format!("trainer-name-{}", section.group.id),
                                    value: trainer_names().get(&section.group.id).cloned().unwrap_or_default(),
                                    placeholder: "Benutzername",
                                    disabled: busy_trainer() == Some(section.group.id),
                                    oninput: move |event: FormEvent| {
                                        trainer_names.with_mut(|entries| {
                                            entries.insert(section.group.id, event.value());
                                        });
                                    },
                                }
                            }
                        }
                        if section.teams.is_empty() {
                            p { class: "auth-help", "Noch keine Mannschaften angelegt." }
                        } else {
                            div { style: "display: grid; gap: 0.5rem; margin-top: 0.75rem;",
                                for team_section in section.teams {
                                    div { style: "display: grid; gap: 0.75rem; padding: 0.85rem; border-radius: 0.85rem; background: var(--accent);",
                                        div { style: "display: flex; align-items: center; justify-content: space-between; gap: 1rem;",
                                            span { style: "font-weight: 600;", "{team_section.team.name}" }
                                            Badge { variant: BadgeVariant::Outline, "Sortierung {team_section.team.sort_order}" }
                                        }
                                        div { style: "display: grid; gap: 0.5rem;",
                                            p { class: "auth-help", "Spieler" }
                                            if team_section.players.is_empty() {
                                                p { class: "auth-help", "Noch keine Spieler zugewiesen." }
                                            } else {
                                                for player in team_section.players {
                                                    div { style: "display: flex; align-items: center; justify-content: space-between; gap: 1rem; padding: 0.65rem 0.8rem; border-radius: 0.75rem; background: var(--background);",
                                                        span { style: "font-weight: 600;", "{player.username}" }
                                                        Button {
                                                            variant: ButtonVariant::Ghost,
                                                            disabled: removing_player() == Some((team_section.team.id, player.user_id)),
                                                            onclick: move |_| on_remove_player.call((team_section.team.id, player.user_id)),
                                                            {if removing_player() == Some((team_section.team.id, player.user_id)) { "Entfernt..." } else { "Entfernen" }}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        div { style: "display: grid; gap: 0.75rem;",
                                            div { class: "auth-field",
                                Label { html_for: format!("player-name-{}", team_section.team.id), "Spieler per Benutzername" }
                                                Input {
                                                    id: format!("player-name-{}", team_section.team.id),
                                                    value: player_names().get(&team_section.team.id).cloned().unwrap_or_default(),
                                                    placeholder: "Benutzername",
                                                    disabled: busy_team() == Some(team_section.team.id),
                                                    oninput: move |event: FormEvent| {
                                                        player_names.with_mut(|entries| {
                                                            entries.insert(team_section.team.id, event.value());
                                                        });
                                                    },
                                                }
                                            }
                                            Button {
                                                variant: ButtonVariant::Outline,
                                                disabled: busy_team().is_some(),
                                                onclick: move |_| on_assign_player.call(team_section.team.id),
                                                {if busy_team() == Some(team_section.team.id) { "Speichert..." } else { "Spieler zuweisen" }}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { style: "display: grid; gap: 0.75rem; margin-top: 1rem;",
                            div { class: "auth-field",
                                Label { html_for: format!("team-name-{}", section.group.id), "Neue Mannschaft" }
                                Input {
                                    id: format!("team-name-{}", section.group.id),
                                    value: new_team_names().get(&section.group.id).cloned().unwrap_or_default(),
                                    placeholder: "z. B. Maenner 1",
                                    disabled: busy_team() == Some(section.group.id),
                                    oninput: move |event: FormEvent| {
                                        new_team_names.with_mut(|entries| {
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
                        div { style: "display: grid; gap: 0.5rem;",
                            Button {
                                variant: ButtonVariant::Outline,
                                disabled: busy_trainer().is_some(),
                                onclick: move |_| on_assign_trainer.call(section.group.id),
                                {if busy_trainer() == Some(section.group.id) { "Speichert..." } else { "Trainer zuweisen" }}
                            }
                            Button {
                                variant: ButtonVariant::Outline,
                                disabled: busy_team().is_some(),
                                onclick: move |_| on_create_team.call(section.group.id),
                                {if busy_team() == Some(section.group.id) { "Speichert..." } else { "Mannschaft anlegen" }}
                            }
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

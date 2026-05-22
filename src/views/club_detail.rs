use crate::auth::current_user;
use crate::clubs::{get_club_detail, ClubDetail as ClubDetailData, ClubGroupWithTeams};
use crate::components::ui::accordion::{
    Accordion, AccordionContent, AccordionItem, AccordionTrigger,
};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::input::Input;
use crate::components::ui::label::Label;
use crate::components::ui::tabs::{TabContent, TabList, TabTrigger, Tabs, TabsVariant};
use crate::components::{
    EmptyStatePanel, LoadingPanel, MetricCard, PageHeader, SectionPanel, StatusBanner,
    StatusBannerTone,
};
use crate::group_trainers::{assign_group_trainer, remove_group_trainer, AssignGroupTrainerInput};
use crate::groups::{create_group, CreateGroupInput};
use crate::invitations::{
    create_invitation, revoke_invitation, CreateInvitationInput, CreatedInvitation, InvitationRole,
};
use crate::team_players::{assign_team_player, remove_team_player, AssignTeamPlayerInput};
use crate::teams::{create_team, CreateTeamInput};
use crate::training::format_timestamp_label;
use dioxus::prelude::*;
use std::collections::HashMap;

#[component]
pub fn ClubDetail(club_id: i32) -> Element {
    let mut refresh = use_signal(|| 0_u64);
    let user_resource =
        use_server_future(move || async move { current_user().await.ok().flatten() })?;
    let detail_resource = use_server_future(move || {
        let _ = refresh();
        async move { get_club_detail(club_id).await }
    })?;
    let user_state = user_resource.read().as_ref().cloned();
    let detail_state = detail_resource.read().as_ref().cloned();
    let mut active_tab = use_signal(|| Some("overview".to_string()));
    let mut group_name = use_signal(String::new);
    let mut group_sort_order = use_signal(|| "0".to_string());
    let mut invitation_days = use_signal(|| "7".to_string());
    let mut latest_invitation = use_signal(|| None::<CreatedInvitation>);
    let mut trainer_names = use_signal(HashMap::<i32, String>::new);
    let mut new_team_names = use_signal(HashMap::<i32, String>::new);
    let mut player_names = use_signal(HashMap::<i32, String>::new);
    let mut team_sort_orders = use_signal(HashMap::<i32, String>::new);
    let mut busy_group = use_signal(|| false);
    let mut busy_invitation = use_signal(|| None::<Option<i32>>);
    let mut busy_trainer = use_signal(|| None::<i32>);
    let mut busy_team = use_signal(|| None::<i32>);
    let mut revoking_invitation = use_signal(|| None::<i32>);
    let mut removing_trainer = use_signal(|| None::<(i32, i32)>);
    let mut removing_player = use_signal(|| None::<(i32, i32)>);
    let mut status = use_signal(|| None::<(bool, String)>);

    match user_state {
        None => rsx! {
            section { class: "page-section",
                div { class: "page-stack",
                    LoadingPanel { title: "Vereinsdetails".to_string(), lines: 4 }
                }
            }
        },
        Some(None) => rsx! {},
        Some(Some(user)) if !user.is_system_admin => rsx! {
            section { class: "page-section",
                div { class: "page-stack",
                    PageHeader {
                        title: "Vereinsdetails".to_string(),
                        description: "Nur System-Admins dürfen Vereine, Gruppen und Mannschaften verwalten.".to_string(),
                        eyebrow: Some("Administration".to_string()),
                    }
                    StatusBanner {
                        tone: StatusBannerTone::Info,
                        title: Some("Kein Zugriff".to_string()),
                        message: "Bitte melde dich mit einem System-Admin-Konto an.".to_string(),
                    }
                }
            }
        },
        Some(Some(_)) => rsx! {
            match detail_state {
                None => rsx! {
                    section { class: "page-section",
                        div { class: "page-stack",
                            LoadingPanel { title: "Vereinsdetails".to_string(), lines: 5 }
                        }
                    }
                },
                Some(Err(error)) => rsx! {
                    section { class: "page-section",
                        div { class: "page-stack",
                            PageHeader {
                                title: "Verein konnte nicht geladen werden".to_string(),
                                description: "Der gewünschte Vereinsbereich steht gerade nicht zur Verfügung.".to_string(),
                                eyebrow: Some("Administration".to_string()),
                            }
                            StatusBanner {
                                tone: StatusBannerTone::Error,
                                title: Some("Laden fehlgeschlagen".to_string()),
                                message: error.to_string(),
                            }
                        }
                    }
                },
                Some(Ok(detail)) => {
                    let group_count = detail.groups.len();
                    let trainer_count =
                        detail.groups.iter().map(|section| section.trainers.len()).sum::<usize>();
                    let team_count =
                        detail.groups.iter().map(|section| section.teams.len()).sum::<usize>();
                    let invitation_count = detail
                        .groups
                        .iter()
                        .map(|section| section.invitations.len())
                        .sum::<usize>();

                    rsx! {
                        section { class: "page-section",
                            div { class: "page-stack page-stack--spacious",
                                PageHeader {
                                    title: detail.club.name.clone(),
                                    description: "Verwalte Gruppen, Einladungen und Mannschaften in einer ruhigen, klar gegliederten Vereinsansicht.".to_string(),
                                    eyebrow: Some("Vereinsverwaltung".to_string()),
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

                                div { class: "metrics-grid",
                                    MetricCard {
                                        label: "Gruppen".to_string(),
                                        value: group_count.to_string(),
                                        detail: Some("Organisierte Bereiche im Verein.".to_string()),
                                    }
                                    MetricCard {
                                        label: "Trainer".to_string(),
                                        value: trainer_count.to_string(),
                                        detail: Some("Aktuell in Gruppen eingetragen.".to_string()),
                                    }
                                    MetricCard {
                                        label: "Mannschaften".to_string(),
                                        value: team_count.to_string(),
                                        detail: Some("Über alle Gruppen hinweg.".to_string()),
                                    }
                                    MetricCard {
                                        label: "Aktive Einladungen".to_string(),
                                        value: invitation_count.to_string(),
                                        detail: Some("Noch gültige Gruppen-Codes.".to_string()),
                                    }
                                }

                                Tabs {
                                    class: "club-detail-tabs".to_string(),
                                    value: ReadSignal::new(active_tab),
                                    default_value: "overview".to_string(),
                                    on_value_change: move |value| active_tab.set(Some(value)),
                                    variant: TabsVariant::Ghost,
                                    TabList {
                                        TabTrigger {
                                            index: 0usize,
                                            value: "overview".to_string(),
                                            "Übersicht"
                                        }
                                        TabTrigger {
                                            index: 1usize,
                                            value: "groups".to_string(),
                                            "Gruppen"
                                        }
                                    }

                                    TabContent {
                                        index: 0usize,
                                        value: "overview".to_string(),
                                        div { class: "tab-section",
                                            SectionPanel {
                                                title: "Spieler-Code für den Verein".to_string(),
                                                description: "Erstelle bei Bedarf einen vereinsweiten Zugangscode für neue Spieler.".to_string(),
                                                div { class: "section-stack",
                                                    div { class: "form-grid-2",
                                                        div { class: "auth-field",
                                                            Label { html_for: "player-invitation-days", "Gültig für Tage" }
                                                            Input {
                                                                id: "player-invitation-days",
                                                                value: invitation_days(),
                                                                placeholder: "7",
                                                                disabled: busy_invitation() == Some(None),
                                                                oninput: move |event: FormEvent| invitation_days.set(event.value()),
                                                            }
                                                        }
                                                    }
                                                    div { class: "section-actions",
                                                        Button {
                                                            variant: ButtonVariant::Outline,
                                                            disabled: busy_invitation().is_some(),
                                                            onclick: move |_| {
                                                                if busy_invitation().is_some() {
                                                                    return;
                                                                }

                                                                status.set(None);
                                                                latest_invitation.set(None);
                                                                let expires_in_days = parse_invitation_days(&invitation_days());
                                                                spawn(async move {
                                                                    let expires_in_days = match expires_in_days {
                                                                        Ok(days) => days,
                                                                        Err(error) => {
                                                                            status.set(Some((false, format!("Spieler-Code konnte nicht erstellt werden: {error}"))));
                                                                            return;
                                                                        }
                                                                    };

                                                                    busy_invitation.set(Some(None));
                                                                    let result = create_invitation(CreateInvitationInput {
                                                                        club_id,
                                                                        group_id: None,
                                                                        role: InvitationRole::Player,
                                                                        expires_in_days,
                                                                    })
                                                                    .await;
                                                                    busy_invitation.set(None);

                                                                    match result {
                                                                        Ok(created_invitation) => {
                                                                            latest_invitation.set(Some(created_invitation.clone()));
                                                                            status.set(Some((true, "Spieler-Code für den Verein wurde erstellt.".to_string())));
                                                                            refresh.with_mut(|value| *value += 1);
                                                                        }
                                                                        Err(error) => {
                                                                            status.set(Some((false, format!("Spieler-Code konnte nicht erstellt werden: {error}"))));
                                                                        }
                                                                    }
                                                                });
                                                            },
                                                            {if busy_invitation() == Some(None) { "Erstellt..." } else { "Spieler-Code erstellen" }}
                                                        }
                                                    }
                                                    if let Some(created_invitation) = latest_invitation() {
                                                        if created_invitation.invitation.group_id.is_none() {
                                                            StatusBanner {
                                                                tone: StatusBannerTone::Success,
                                                                title: Some("Neuer Spieler-Code".to_string()),
                                                                message: created_invitation.plain_code.clone(),
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            SectionPanel {
                                                title: "Neue Gruppe anlegen".to_string(),
                                                description: "Lege weitere Gruppen an und gib ihnen direkt eine Reihenfolge für die Vereinsansicht.".to_string(),
                                                div { class: "section-stack",
                                                    div { class: "form-grid-2",
                                                        div { class: "auth-field",
                                                            Label { html_for: "group-name", "Gruppenname" }
                                                            Input {
                                                                id: "group-name",
                                                                value: group_name(),
                                                                placeholder: "z. B. Männer",
                                                                disabled: busy_group(),
                                                                oninput: move |event: FormEvent| group_name.set(event.value()),
                                                            }
                                                        }
                                                        div { class: "auth-field",
                                                            Label { html_for: "group-sort-order", "Reihenfolge" }
                                                            Input {
                                                                id: "group-sort-order",
                                                                value: group_sort_order(),
                                                                placeholder: "0",
                                                                disabled: busy_group(),
                                                                oninput: move |event: FormEvent| group_sort_order.set(event.value()),
                                                            }
                                                        }
                                                    }
                                                    div { class: "section-actions",
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
                                    }

                                    TabContent {
                                        index: 1usize,
                                        value: "groups".to_string(),
                                        SectionPanel {
                                            title: "Gruppen und Mannschaften".to_string(),
                                            description: "Öffne die Gruppe, in der du gerade arbeitest. Details bleiben bis dahin kompakt.".to_string(),
                                            {if detail.groups.is_empty() {
                                                rsx! {
                                                    EmptyStatePanel {
                                                        title: "Noch keine Gruppen vorhanden".to_string(),
                                                        message: "Lege in der Übersicht zuerst die erste Gruppe für diesen Verein an.".to_string(),
                                                    }
                                                }
                                            } else {
                                                rsx! {
                                                    GroupAccordion {
                                                        detail,
                                                        trainer_names,
                                                        new_team_names,
                                                        player_names,
                                                        team_sort_orders,
                                                        invitation_days,
                                                        latest_invitation,
                                                        busy_invitation,
                                                        busy_trainer,
                                                        busy_team,
                                                        revoking_invitation,
                                                        removing_trainer,
                                                        removing_player,
                                                        on_create_invitation: move |(group_id, role)| {
                                                            if busy_invitation().is_some() {
                                                                return;
                                                            }

                                                            status.set(None);
                                                            latest_invitation.set(None);
                                                            let expires_in_days = parse_invitation_days(&invitation_days());
                                                            spawn(async move {
                                                                let expires_in_days = match expires_in_days {
                                                                    Ok(days) => days,
                                                                    Err(error) => {
                                                                        status.set(Some((false, format!("Einladung konnte nicht erstellt werden: {error}"))));
                                                                        return;
                                                                    }
                                                                };

                                                                busy_invitation.set(Some(Some(group_id)));
                                                                let result = create_invitation(CreateInvitationInput {
                                                                    club_id,
                                                                    group_id: Some(group_id),
                                                                    role,
                                                                    expires_in_days,
                                                                })
                                                                .await;
                                                                busy_invitation.set(None);

                                                                match result {
                                                                    Ok(created_invitation) => {
                                                                        latest_invitation.set(Some(created_invitation.clone()));
                                                                        let label = match role {
                                                                            InvitationRole::Trainer => "Trainer-Code",
                                                                            InvitationRole::Player => "Spieler-Code",
                                                                        };
                                                                        status.set(Some((true, format!("{label} für die Gruppe wurde erstellt."))));
                                                                        refresh.with_mut(|value| *value += 1);
                                                                    }
                                                                    Err(error) => {
                                                                        status.set(Some((false, format!("Einladung konnte nicht erstellt werden: {error}"))));
                                                                    }
                                                                }
                                                            });
                                                        },
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
                                                                        status.set(Some((true, format!("{} wurde als Trainer zugewiesen.", trainer.username))));
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
                                                                        status.set(Some((true, "Trainer wurde aus der Gruppe entfernt.".to_string())));
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
                                                            let sort_order = team_sort_orders()
                                                                .get(&group_id)
                                                                .cloned()
                                                                .unwrap_or_else(|| "0".to_string());
                                                            spawn(async move {
                                                                let sort_order = match parse_sort_order(&sort_order) {
                                                                    Ok(value) => value,
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
                                                                    Ok(team) => {
                                                                        new_team_names.with_mut(|entries| {
                                                                            entries.insert(group_id, String::new());
                                                                        });
                                                                        team_sort_orders.with_mut(|entries| {
                                                                            entries.insert(group_id, "0".to_string());
                                                                        });
                                                                        status.set(Some((true, format!("Mannschaft '{}' wurde angelegt.", team.name))));
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
                                                                let result = assign_team_player(AssignTeamPlayerInput { team_id, username }).await;
                                                                busy_team.set(None);

                                                                match result {
                                                                    Ok(player) => {
                                                                        player_names.with_mut(|entries| {
                                                                            entries.insert(team_id, String::new());
                                                                        });
                                                                        status.set(Some((true, format!("{} wurde der Mannschaft zugewiesen.", player.username))));
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
                                                                        status.set(Some((true, "Spieler wurde aus der Mannschaft entfernt.".to_string())));
                                                                        refresh.with_mut(|value| *value += 1);
                                                                    }
                                                                    Err(error) => {
                                                                        status.set(Some((false, format!("Spieler konnte nicht entfernt werden: {error}"))));
                                                                    }
                                                                }
                                                            });
                                                        },
                                                        on_revoke_invitation: move |invitation_id| {
                                                            if revoking_invitation().is_some() {
                                                                return;
                                                            }

                                                            status.set(None);
                                                            spawn(async move {
                                                                revoking_invitation.set(Some(invitation_id));
                                                                let result = revoke_invitation(invitation_id).await;
                                                                revoking_invitation.set(None);

                                                                match result {
                                                                    Ok(()) => {
                                                                        status.set(Some((true, "Einladung wurde widerrufen.".to_string())));
                                                                        refresh.with_mut(|value| *value += 1);
                                                                    }
                                                                    Err(error) => {
                                                                        status.set(Some((false, format!("Einladung konnte nicht widerrufen werden: {error}"))));
                                                                    }
                                                                }
                                                            });
                                                        }
                                                    }
                                                }
                                            }}
                                        }
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
fn GroupAccordion(
    detail: ClubDetailData,
    trainer_names: Signal<HashMap<i32, String>>,
    new_team_names: Signal<HashMap<i32, String>>,
    player_names: Signal<HashMap<i32, String>>,
    team_sort_orders: Signal<HashMap<i32, String>>,
    invitation_days: Signal<String>,
    latest_invitation: Signal<Option<CreatedInvitation>>,
    busy_invitation: Signal<Option<Option<i32>>>,
    busy_trainer: Signal<Option<i32>>,
    busy_team: Signal<Option<i32>>,
    revoking_invitation: Signal<Option<i32>>,
    removing_trainer: Signal<Option<(i32, i32)>>,
    removing_player: Signal<Option<(i32, i32)>>,
    on_create_invitation: EventHandler<(i32, InvitationRole)>,
    on_assign_trainer: EventHandler<i32>,
    on_remove_trainer: EventHandler<(i32, i32)>,
    on_create_team: EventHandler<i32>,
    on_assign_player: EventHandler<i32>,
    on_remove_player: EventHandler<(i32, i32)>,
    on_revoke_invitation: EventHandler<i32>,
) -> Element {
    rsx! {
        Accordion {
            collapsible: true,
            allow_multiple_open: true,
            class: "group-accordion",
            for (index, section) in detail.groups.into_iter().enumerate() {
                {
                    let group_id = section.group.id;
                    let trainer_count = section.trainers.len();
                    let team_count = section.teams.len();
                    let invitation_count = section.invitations.len();
                    let group_name = section.group.name.clone();
                    let group_sort_label = format!("Reihenfolge {}", section.group.sort_order);

                    rsx! {
                        AccordionItem { index,
                            AccordionTrigger {
                                div { class: "group-accordion-trigger",
                                    div { class: "group-accordion-copy",
                                        p { class: "detail-card-title", "{group_name}" }
                                        p { class: "section-meta", "{group_sort_label}" }
                                    }
                                    div { class: "group-accordion-stats",
                                        span { class: "group-summary-pill", "{trainer_count} Trainer" }
                                        span { class: "group-summary-pill", "{team_count} Mannschaften" }
                                        span { class: "group-summary-pill", "{invitation_count} Einladungen" }
                                    }
                                }
                            }
                            AccordionContent {
                                div { class: "group-accordion-panel",
                                    SectionPanel {
                                        title: "Trainer".to_string(),
                                        description: "Verwalte die Trainer, die in dieser Gruppe arbeiten dürfen.".to_string(),
                                        GroupTrainerSection {
                                            section: section.clone(),
                                            trainer_names,
                                            busy_trainer,
                                            removing_trainer,
                                            on_assign_trainer,
                                            on_remove_trainer,
                                        }
                                    }

                                    SectionPanel {
                                        title: "Einladungen".to_string(),
                                        description: "Erstelle und widerrufe Codes für Trainer und Spieler dieser Gruppe.".to_string(),
                                        GroupInvitationSection {
                                            group_id,
                                            invitations: section.invitations.clone(),
                                            invitation_days,
                                            latest_invitation,
                                            busy_invitation,
                                            revoking_invitation,
                                            on_create_invitation,
                                            on_revoke_invitation,
                                        }
                                    }

                                    SectionPanel {
                                        title: "Mannschaften".to_string(),
                                        description: "Lege Mannschaften an und ordne Spieler gezielt zu.".to_string(),
                                        GroupTeamsSection {
                                            group_id,
                                            teams: section.teams.clone(),
                                            new_team_names,
                                            player_names,
                                            team_sort_orders,
                                            busy_team,
                                            removing_player,
                                            on_create_team,
                                            on_assign_player,
                                            on_remove_player,
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
fn GroupTrainerSection(
    section: ClubGroupWithTeams,
    trainer_names: Signal<HashMap<i32, String>>,
    busy_trainer: Signal<Option<i32>>,
    removing_trainer: Signal<Option<(i32, i32)>>,
    on_assign_trainer: EventHandler<i32>,
    on_remove_trainer: EventHandler<(i32, i32)>,
) -> Element {
    let group_id = section.group.id;

    rsx! {
        div { class: "section-stack",
            if section.trainers.is_empty() {
                EmptyStatePanel {
                    title: "Noch keine Trainer zugewiesen".to_string(),
                    message: "Füge den ersten Trainer hinzu, damit die Gruppe betreut werden kann.".to_string(),
                }
            } else {
                div { class: "detail-list",
                    for trainer in section.trainers {
                        {
                            let trainer_user_id = trainer.user_id;
                            let trainer_name = trainer.username.clone();

                            rsx! {
                                div { class: "detail-row",
                                    div { class: "detail-row-copy",
                                        span { class: "detail-row-title", "{trainer_name}" }
                                    }
                                    Button {
                                        variant: ButtonVariant::Ghost,
                                        disabled: removing_trainer() == Some((group_id, trainer_user_id)),
                                        onclick: move |_| on_remove_trainer.call((group_id, trainer_user_id)),
                                        {if removing_trainer() == Some((group_id, trainer_user_id)) { "Entfernt..." } else { "Entfernen" }}
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "form-grid",
                div { class: "auth-field",
                    Label { html_for: format!("trainer-name-{}", group_id), "Trainer zuweisen" }
                    Input {
                        id: format!("trainer-name-{}", group_id),
                        value: trainer_names().get(&group_id).cloned().unwrap_or_default(),
                        placeholder: "Benutzername",
                        disabled: busy_trainer() == Some(group_id),
                        oninput: move |event: FormEvent| {
                            trainer_names.with_mut(|entries| {
                                entries.insert(group_id, event.value());
                            });
                        },
                    }
                }
            }
            div { class: "section-actions",
                Button {
                    variant: ButtonVariant::Outline,
                    disabled: busy_trainer().is_some(),
                    onclick: move |_| on_assign_trainer.call(group_id),
                    {if busy_trainer() == Some(group_id) { "Speichert..." } else { "Trainer zuweisen" }}
                }
            }
        }
    }
}

#[component]
fn GroupInvitationSection(
    group_id: i32,
    invitations: Vec<crate::invitations::InvitationSummary>,
    invitation_days: Signal<String>,
    latest_invitation: Signal<Option<CreatedInvitation>>,
    busy_invitation: Signal<Option<Option<i32>>>,
    revoking_invitation: Signal<Option<i32>>,
    on_create_invitation: EventHandler<(i32, InvitationRole)>,
    on_revoke_invitation: EventHandler<i32>,
) -> Element {
    rsx! {
        div { class: "section-stack",
            div { class: "form-grid-2",
                div { class: "auth-field",
                    Label { html_for: format!("invitation-days-{}", group_id), "Code gültig für Tage" }
                    Input {
                        id: format!("invitation-days-{}", group_id),
                        value: invitation_days(),
                        placeholder: "7",
                        disabled: busy_invitation().is_some(),
                        oninput: move |event: FormEvent| invitation_days.set(event.value()),
                    }
                }
            }
            div { class: "section-actions",
                Button {
                    variant: ButtonVariant::Outline,
                    disabled: busy_invitation().is_some(),
                    onclick: move |_| on_create_invitation.call((group_id, InvitationRole::Trainer)),
                    {if busy_invitation() == Some(Some(group_id)) { "Erstellt..." } else { "Trainer-Code" }}
                }
                Button {
                    variant: ButtonVariant::Outline,
                    disabled: busy_invitation().is_some(),
                    onclick: move |_| on_create_invitation.call((group_id, InvitationRole::Player)),
                    {if busy_invitation() == Some(Some(group_id)) { "Erstellt..." } else { "Spieler-Code" }}
                }
            }
            if let Some(created_invitation) = latest_invitation() {
                if created_invitation.invitation.group_id == Some(group_id) {
                    StatusBanner {
                        tone: StatusBannerTone::Success,
                        title: Some("Neuer Code".to_string()),
                        message: created_invitation.plain_code.clone(),
                    }
                }
            }

            if invitations.is_empty() {
                EmptyStatePanel {
                    title: "Keine aktiven Einladungen".to_string(),
                    message: "Erstelle bei Bedarf einen neuen Code für Trainer oder Spieler.".to_string(),
                }
            } else {
                div { class: "detail-list",
                    for invitation in invitations {
                        {
                            let invitation_id = invitation.id;
                            let role_label = role_label(invitation.role);

                            rsx! {
                                div { class: "detail-row",
                                    div { class: "detail-row-copy",
                                        span { class: "detail-row-title", "{role_label}-Code" }
                                        p { class: "detail-row-meta", "Gültig bis {format_timestamp_label(invitation.expires_at)}" }
                                    }
                                    Button {
                                        variant: ButtonVariant::Ghost,
                                        disabled: revoking_invitation() == Some(invitation_id),
                                        onclick: move |_| on_revoke_invitation.call(invitation_id),
                                        {if revoking_invitation() == Some(invitation_id) { "Widerruft..." } else { "Widerrufen" }}
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
fn GroupTeamsSection(
    group_id: i32,
    teams: Vec<crate::clubs::TeamWithPlayers>,
    new_team_names: Signal<HashMap<i32, String>>,
    player_names: Signal<HashMap<i32, String>>,
    team_sort_orders: Signal<HashMap<i32, String>>,
    busy_team: Signal<Option<i32>>,
    removing_player: Signal<Option<(i32, i32)>>,
    on_create_team: EventHandler<i32>,
    on_assign_player: EventHandler<i32>,
    on_remove_player: EventHandler<(i32, i32)>,
) -> Element {
    rsx! {
        div { class: "section-stack",
            if teams.is_empty() {
                EmptyStatePanel {
                    title: "Noch keine Mannschaften angelegt".to_string(),
                    message: "Lege die erste Mannschaft dieser Gruppe direkt hier an.".to_string(),
                }
            } else {
                for team_section in teams {
                    {
                        let team_id = team_section.team.id;
                        let team_name = team_section.team.name.clone();

                        rsx! {
                            div { class: "detail-card",
                                div { class: "detail-card-header",
                                    div { class: "detail-card-copy",
                                        p { class: "detail-card-title", "{team_name}" }
                                        p { class: "section-meta", "{team_section.players.len()} zugewiesene Spieler" }
                                    }
                                }
                                if team_section.players.is_empty() {
                                    EmptyStatePanel {
                                        title: "Noch keine Spieler zugewiesen".to_string(),
                                        message: "Ordne dieser Mannschaft den ersten Spieler zu.".to_string(),
                                    }
                                } else {
                                    div { class: "detail-list",
                                        for player in team_section.players {
                                            {
                                                let player_user_id = player.user_id;
                                                let player_name = player.username.clone();

                                                rsx! {
                                                    div { class: "detail-row",
                                                        div { class: "detail-row-copy",
                                                            span { class: "detail-row-title", "{player_name}" }
                                                        }
                                                        Button {
                                                            variant: ButtonVariant::Ghost,
                                                            disabled: removing_player() == Some((team_id, player_user_id)),
                                                            onclick: move |_| on_remove_player.call((team_id, player_user_id)),
                                                            {if removing_player() == Some((team_id, player_user_id)) { "Entfernt..." } else { "Entfernen" }}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                div { class: "form-grid",
                                    div { class: "auth-field",
                                        Label { html_for: format!("player-name-{}", team_id), "Spieler zuweisen" }
                                        Input {
                                            id: format!("player-name-{}", team_id),
                                            value: player_names().get(&team_id).cloned().unwrap_or_default(),
                                            placeholder: "Benutzername",
                                            disabled: busy_team() == Some(team_id),
                                            oninput: move |event: FormEvent| {
                                                player_names.with_mut(|entries| {
                                                    entries.insert(team_id, event.value());
                                                });
                                            },
                                        }
                                    }
                                }
                                div { class: "section-actions",
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        disabled: busy_team().is_some(),
                                        onclick: move |_| on_assign_player.call(team_id),
                                        {if busy_team() == Some(team_id) { "Speichert..." } else { "Spieler zuweisen" }}
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "detail-card detail-card-muted",
                p { class: "section-label", "Neue Mannschaft" }
                div { class: "form-grid-2",
                    div { class: "auth-field",
                        Label { html_for: format!("team-name-{}", group_id), "Name" }
                        Input {
                            id: format!("team-name-{}", group_id),
                            value: new_team_names().get(&group_id).cloned().unwrap_or_default(),
                            placeholder: "z. B. Männer 1",
                            disabled: busy_team() == Some(group_id),
                            oninput: move |event: FormEvent| {
                                new_team_names.with_mut(|entries| {
                                    entries.insert(group_id, event.value());
                                });
                            },
                        }
                    }
                    div { class: "auth-field",
                        Label { html_for: format!("team-sort-order-{}", group_id), "Reihenfolge" }
                        Input {
                            id: format!("team-sort-order-{}", group_id),
                            value: team_sort_orders().get(&group_id).cloned().unwrap_or_else(|| "0".to_string()),
                            placeholder: "0",
                            disabled: busy_team() == Some(group_id),
                            oninput: move |event: FormEvent| {
                                team_sort_orders.with_mut(|entries| {
                                    entries.insert(group_id, event.value());
                                });
                            },
                        }
                    }
                }
                div { class: "section-actions",
                    Button {
                        variant: ButtonVariant::Secondary,
                        disabled: busy_team().is_some(),
                        onclick: move |_| on_create_team.call(group_id),
                        {if busy_team() == Some(group_id) { "Speichert..." } else { "Mannschaft anlegen" }}
                    }
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

fn parse_invitation_days(value: &str) -> Result<i32, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(7);
    }

    trimmed
        .parse::<i32>()
        .map_err(|_| "Die Gültigkeit muss eine ganze Zahl in Tagen sein.".to_string())
}

fn role_label(role: InvitationRole) -> &'static str {
    match role {
        InvitationRole::Trainer => "Trainer",
        InvitationRole::Player => "Spieler",
    }
}

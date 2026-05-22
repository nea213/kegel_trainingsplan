use crate::auth::current_user;
use crate::clubs::{create_club, list_clubs, ClubSummary, CreateClubInput};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::input::Input;
use crate::components::ui::item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemGroup, ItemSeparator, ItemTitle,
};
use crate::components::ui::label::Label;
use crate::components::ui::sheet::{
    Sheet, SheetContentClose, SheetDescription, SheetHeader, SheetTitle,
};
use crate::components::{
    EmptyStatePanel, LoadingPanel, PageHeader, SectionPanel, StatusBanner, StatusBannerTone,
};
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Clubs() -> Element {
    let mut refresh = use_signal(|| 0_u64);
    let user_resource = use_server_future(move || async move { current_user().await.ok().flatten() })?;
    let clubs_resource = use_server_future(move || {
        let _ = refresh();
        async move { list_clubs().await }
    })?;
    let user_state = user_resource.read().as_ref().cloned();
    let clubs_state = clubs_resource.read().as_ref().cloned();
    let nav = navigator();
    let mut club_name = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut status = use_signal(|| None::<(bool, String)>);
    let mut create_sheet_open = use_signal(|| false);

    match user_state {
        None => rsx! {
            section { class: "page-section",
                div { class: "page-stack",
                    LoadingPanel { title: "Vereine".to_string(), lines: 4 }
                }
            }
        },
        Some(None) => rsx! {},
        Some(Some(user)) if !user.is_system_admin => rsx! {
            section { class: "page-section",
                div { class: "page-stack",
                    PageHeader {
                        title: "Vereinsverwaltung".to_string(),
                        description: "Nur System-Admins dürfen Vereine, Gruppen und Mannschaften verwalten.".to_string(),
                        eyebrow: Some("Administration".to_string()),
                    }
                    StatusBanner {
                        tone: StatusBannerTone::Info,
                        title: Some("Kein Zugriff".to_string()),
                        message: "Melde dich mit einem System-Admin-Konto an, um Vereine zu pflegen.".to_string(),
                    }
                }
            }
        },
        Some(Some(_)) => rsx! {
            section { class: "page-section",
                div { class: "page-stack page-stack--spacious",
                    PageHeader {
                        title: "Vereine".to_string(),
                        description: "Lege Vereine an und öffne anschließend deren Detailbereiche für Gruppen und Mannschaften.".to_string(),
                        eyebrow: Some("Administration".to_string()),
                        actions: Some(rsx! {
                            div { class: "mobile-only clubs-header-actions",
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    onclick: move |_| create_sheet_open.set(true),
                                    "Verein anlegen"
                                }
                            }
                        }),
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

                    div { class: "desktop-only",
                        SectionPanel {
                            title: "Neuen Verein erfassen".to_string(),
                            description: "Der Verein erscheint sofort in der Übersicht und kann direkt weiter aufgebaut werden.".to_string(),
                            div { class: "clubs-create-callout",
                                div { class: "clubs-create-callout__copy",
                                    p { class: "section-label", "Schneller Einstieg" }
                                    p { class: "section-meta", "Lege neue Vereine an und öffne danach direkt die Detailansicht für Gruppen, Mannschaften und Einladungen." }
                                }
                                ClubCreateForm {
                                    club_name,
                                    busy,
                                    id_prefix: "club-desktop".to_string(),
                                    on_submit: move |_| {
                                        if busy() {
                                            return;
                                        }

                                        status.set(None);
                                        let name = club_name();
                                        spawn(async move {
                                            busy.set(true);
                                            let result = create_club(CreateClubInput { name }).await;
                                            busy.set(false);

                                            match result {
                                                Ok(created_club) => {
                                                    club_name.set(String::new());
                                                    status.set(Some((
                                                        true,
                                                        format!("Verein '{}' wurde angelegt.", created_club.name),
                                                    )));
                                                    refresh.with_mut(|value| *value += 1);
                                                }
                                                Err(error) => {
                                                    status.set(Some((
                                                        false,
                                                        format!("Verein konnte nicht angelegt werden: {error}"),
                                                    )));
                                                }
                                            }
                                        });
                                    }
                                }
                            }
                        }
                    }

                    Sheet {
                        open: create_sheet_open(),
                        on_open_change: move |open| create_sheet_open.set(open),
                        class: "mobile-form-sheet".to_string(),
                        "data-side": "bottom",
                        SheetContentClose {}
                        SheetHeader {
                            SheetTitle { "Verein anlegen" }
                            SheetDescription { "Erfasse einen neuen Verein, ohne die Übersicht zu verlassen." }
                        }
                        div { class: "mobile-sheet-body",
                            ClubCreateForm {
                                club_name,
                                busy,
                                id_prefix: "club-sheet".to_string(),
                                on_submit: move |_| {
                                    if busy() {
                                        return;
                                    }

                                    status.set(None);
                                    let name = club_name();
                                    spawn(async move {
                                        busy.set(true);
                                        let result = create_club(CreateClubInput { name }).await;
                                        busy.set(false);

                                        match result {
                                            Ok(created_club) => {
                                                club_name.set(String::new());
                                                create_sheet_open.set(false);
                                                status.set(Some((
                                                    true,
                                                    format!("Verein '{}' wurde angelegt.", created_club.name),
                                                )));
                                                refresh.with_mut(|value| *value += 1);
                                            }
                                            Err(error) => {
                                                status.set(Some((
                                                    false,
                                                    format!("Verein konnte nicht angelegt werden: {error}"),
                                                )));
                                            }
                                        }
                                    });
                                }
                            }
                        }
                    }

                    match clubs_state {
                        None => rsx! {
                            LoadingPanel {
                                title: "Bestehende Vereine".to_string(),
                                lines: 5,
                            }
                        },
                        Some(Err(error)) => rsx! {
                            SectionPanel {
                                title: "Bestehende Vereine".to_string(),
                                description: "Wähle einen Verein aus, um Gruppen und Mannschaften zu pflegen.".to_string(),
                                StatusBanner {
                                    tone: StatusBannerTone::Error,
                                    title: Some("Vereine konnten nicht geladen werden".to_string()),
                                    message: error.to_string(),
                                }
                            }
                        },
                        Some(Ok(clubs)) if clubs.is_empty() => rsx! {
                            SectionPanel {
                                title: "Bestehende Vereine".to_string(),
                                description: "Wähle einen Verein aus, um Gruppen und Mannschaften zu pflegen.".to_string(),
                                EmptyStatePanel {
                                    title: "Noch kein Verein vorhanden".to_string(),
                                    message: "Lege oben den ersten Verein an, um mit der Strukturierung zu beginnen.".to_string(),
                                }
                            }
                        },
                        Some(Ok(clubs)) => rsx! {
                            SectionPanel {
                                title: "Bestehende Vereine".to_string(),
                                description: "Wähle einen Verein aus, um Gruppen und Mannschaften zu pflegen.".to_string(),
                                ClubList {
                                    clubs,
                                    on_open: move |club_id| {
                                        let _ = nav.push(Route::ClubDetail { club_id });
                                    },
                                }
                            }
                        },
                    }
                }
            }
        },
    }
}

#[component]
fn ClubCreateForm(
    club_name: Signal<String>,
    busy: Signal<bool>,
    on_submit: EventHandler<()>,
    #[props(default = "club".to_string())] id_prefix: String,
) -> Element {
    rsx! {
        div { class: "section-stack mobile-form-stack clubs-create-form",
            div { class: "form-grid-2 clubs-create-form__grid",
                div { class: "auth-field",
                    Label { html_for: format!("{id_prefix}-name"), "Vereinsname" }
                    Input {
                        id: format!("{id_prefix}-name"),
                        value: club_name(),
                        placeholder: "z. B. KV Musterstadt",
                        disabled: busy(),
                        oninput: move |event: FormEvent| club_name.set(event.value()),
                    }
                }
            }
            div { class: "section-actions mobile-form-actions",
                Button {
                    variant: ButtonVariant::Secondary,
                    disabled: busy(),
                    onclick: move |_| on_submit.call(()),
                    {if busy() { "Speichert..." } else { "Verein anlegen" }}
                }
            }
        }
    }
}

#[component]
fn ClubList(clubs: Vec<ClubSummary>, on_open: EventHandler<i32>) -> Element {
    let club_count = clubs.len();

    rsx! {
        ItemGroup {
            for (index, club) in clubs.into_iter().enumerate() {
                Item {
                    class: "content-list-item actionable-list-item club-list-item",
                    ItemContent {
                        ItemTitle { "{club.name}" }
                        ItemDescription { "Öffne den Verein für Gruppen, Einladungen und Mannschaften." }
                        div { class: "detail-badges",
                            Badge { variant: BadgeVariant::Secondary, "Verwaltung" }
                            Badge { variant: BadgeVariant::Outline, "Direkt öffnen" }
                        }
                    }
                    ItemActions {
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| on_open.call(club.id),
                            "Öffnen"
                        }
                    }
                }
                if index + 1 < club_count {
                    ItemSeparator {}
                }
            }
        }
    }
}

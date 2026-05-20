use crate::clubs::{create_club, list_clubs, ClubSummary, CreateClubInput};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::input::Input;
use crate::components::ui::item::{
    Item, ItemActions, ItemContent, ItemGroup, ItemSeparator, ItemTitle,
};
use crate::components::ui::label::Label;
use crate::auth::current_user;
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

    match user_state {
        None => rsx! {
            section { class: "page-section",
                div { class: "auth-status",
                    p { class: "auth-help", "Berechtigungen werden geladen..." }
                }
            }
        },
        Some(None) => rsx! {},
        Some(Some(user)) if !user.is_system_admin => rsx! {
            section { class: "page-section",
                Card { class: "home-intro-card",
                    CardHeader {
                        CardTitle { "Vereinsverwaltung" }
                        CardDescription {
                            "Nur System-Admins dürfen Vereine, Gruppen und Mannschaften verwalten."
                        }
                    }
                }
            }
        },
        Some(Some(_)) => rsx! {
            section { class: "page-section",
                Card { class: "home-intro-card",
                    CardHeader {
                        CardTitle { "Vereine" }
                        CardDescription {
                            "Lege Vereine an und verwalte anschließend deren Gruppen und Mannschaften im Detailbereich."
                        }
                    }
                    CardContent {
                        div { class: "section-stack",
                            div { class: "auth-field",
                                Label { html_for: "club-name", "Neuer Verein" }
                                Input {
                                    id: "club-name",
                                    value: club_name(),
                                    placeholder: "z. B. KV Musterstadt",
                                    disabled: busy(),
                                    oninput: move |event: FormEvent| club_name.set(event.value()),
                                }
                            }
                            Button {
                                variant: ButtonVariant::Secondary,
                                disabled: busy(),
                                onclick: move |_| {
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
                                                status.set(Some((true, format!("Verein '{}' wurde angelegt.", created_club.name))));
                                                refresh.with_mut(|value| *value += 1);
                                            }
                                            Err(error) => {
                                                status.set(Some((false, format!("Verein konnte nicht angelegt werden: {error}"))));
                                            }
                                        }
                                    });
                                },
                                {if busy() { "Speichert..." } else { "Verein anlegen" }}
                            }
                        }
                    }
                }
            }

            section { class: "page-section",
                Card { class: "home-intro-card",
                    CardHeader {
                        CardTitle { "Bestehende Vereine" }
                        CardDescription {
                            "Wähle einen Verein aus, um Gruppen und Mannschaften zu pflegen."
                        }
                    }
                    CardContent {
                        match clubs_state {
                            None => rsx! {
                                p { class: "auth-help", "Vereinsliste wird geladen..." }
                            },
                            Some(Err(error)) => rsx! {
                                div { class: "auth-status auth-status--error",
                                    p { class: "auth-help", "Vereine konnten nicht geladen werden: {error}" }
                                }
                            },
                            Some(Ok(clubs)) if clubs.is_empty() => rsx! {
                                p { class: "auth-help", "Es wurde noch kein Verein angelegt." }
                            },
                            Some(Ok(clubs)) => rsx! {
                                ClubList {
                                    clubs,
                                    on_open: move |club_id| {
                                        let _ = nav.push(Route::ClubDetail { club_id });
                                    },
                                }
                            },
                        }

                        if let Some((success, message)) = status() {
                            div { class: if success { "auth-status auth-status--success" } else { "auth-status auth-status--error" },
                                p { class: "auth-help", "{message}" }
                            }
                        }
                    }
                }
            }
        },
    }
}

#[component]
fn ClubList(clubs: Vec<ClubSummary>, on_open: EventHandler<i32>) -> Element {
    let club_count = clubs.len();

    rsx! {
        ItemGroup {
            for (index, club) in clubs.into_iter().enumerate() {
                Item {
                    ItemContent {
                        ItemTitle { "{club.name}" }
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

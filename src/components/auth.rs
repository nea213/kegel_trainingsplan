use crate::auth::{current_user, login_user, logout_user, register_user, LoginInput, RegisterInput};
use crate::components::ui::avatar::{Avatar, AvatarFallback, AvatarImageSize};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::{
    Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,
};
use crate::components::ui::input::Input;
use crate::components::ui::label::Label;
use crate::components::ui::separator::Separator;
use dioxus::prelude::*;

const AUTH_CSS: Asset = asset!("/assets/styling/auth.css");

#[component]
pub fn AuthPanel() -> Element {
    let mut auth_refresh = use_context::<Signal<u64>>();
    let auth_resource = use_server_future(move || {
        let _ = auth_refresh();
        async move { current_user().await.ok().flatten() }
    })?;

    let current_user = auth_resource.read().as_ref().cloned().flatten();
    let current_user_initials = current_user
        .as_ref()
        .map(|user| {
            user.username
                .chars()
                .filter(|char| char.is_alphanumeric())
                .take(2)
                .collect::<String>()
                .to_uppercase()
        })
        .filter(|initials| !initials.is_empty())
        .unwrap_or_else(|| "KT".to_string());

    let mut status = use_signal(|| None::<(bool, String)>);
    let mut busy = use_signal(|| false);

    let mut register_username = use_signal(String::new);
    let mut register_password = use_signal(String::new);
    let mut login_username = use_signal(String::new);
    let mut login_password = use_signal(String::new);

    rsx! {
        document::Link { rel: "stylesheet", href: AUTH_CSS }

        div {
            id: "auth-panel",

            if let Some(user) = current_user {
                Card {
                    class: "auth-card auth-card--wide",
                    CardHeader {
                        div { class: "auth-card-title-row",
                            div {
                                CardTitle { "Willkommen, {user.username}" }
                                CardDescription {
                                    "Deine Session liegt serverseitig in SQLite und wird über Cookies für Web und Mobile wiederverwendet."
                                }
                            }
                            CardAction {
                                Badge { variant: BadgeVariant::Secondary, "Sitzung aktiv" }
                            }
                        }
                    }
                    CardContent {
                        div { class: "auth-user-row",
                            Avatar { size: AvatarImageSize::Medium, aria_label: "Benutzerprofil",
                                AvatarFallback { "{current_user_initials}" }
                            }
                            div { class: "auth-user-meta",
                                p { class: "auth-user-copy", "Angemeldet als {user.username}" }
                                Badge { variant: BadgeVariant::Outline, "Server-Session" }
                            }
                        }
                        Separator { horizontal: true, decorative: true, style: "margin: 1rem 0;" }
                        p {
                            class: "auth-help",
                            "Du kannst dich jederzeit wieder ausloggen. Danach aktualisieren wir den Session-Status sofort in der Navigation und auf dieser Seite."
                        }
                    }
                    CardFooter {
                        class: "auth-actions",
                        Button {
                            variant: ButtonVariant::Secondary,
                            disabled: busy(),
                            onclick: move |_| {
                                if busy() {
                                    return;
                                }

                                spawn(async move {
                                    busy.set(true);
                                    let result = logout_user().await;
                                    busy.set(false);

                                    match result {
                                        Ok(()) => {
                                            status.set(Some((true, "Du wurdest erfolgreich ausgeloggt.".to_string())));
                                            auth_refresh.with_mut(|value| *value += 1);
                                        }
                                        Err(error) => {
                                            status.set(Some((false, format!("Logout fehlgeschlagen: {error}"))));
                                        }
                                    }
                                });
                            },
                            {if busy() { "Logout läuft..." } else { "Logout" }}
                        }
                    }
                }
            } else {
                div {
                    class: "auth-grid",
                    Card {
                        class: "auth-card",
                        CardHeader {
                            div { class: "auth-card-title-row",
                                CardTitle { "Registrieren" }
                                CardAction {
                                    Badge { variant: BadgeVariant::Secondary, "Neu" }
                                }
                            }
                            CardDescription {
                                "Lege deinen ersten Benutzer an. Nach erfolgreicher Registrierung loggen wir dich direkt ein."
                            }
                        }
                        CardContent {
                            div { class: "auth-form",
                                div { class: "auth-field",
                                    Label { html_for: "register-username", "Benutzername" }
                                    Input {
                                        id: "register-username",
                                        value: register_username(),
                                        placeholder: "Benutzername",
                                        disabled: busy(),
                                        oninput: move |event: FormEvent| register_username.set(event.value()),
                                    }
                                }
                                div { class: "auth-field",
                                    Label { html_for: "register-password", "Passwort" }
                                    Input {
                                        id: "register-password",
                                        r#type: "password",
                                        value: register_password(),
                                        placeholder: "Passwort",
                                        disabled: busy(),
                                        oninput: move |event: FormEvent| register_password.set(event.value()),
                                    }
                                }
                            }
                        }
                        CardFooter {
                            Button {
                                style: "width: 100%;",
                                disabled: busy(),
                                onclick: move |_| {
                                    if busy() {
                                        return;
                                    }

                                    spawn(async move {
                                        busy.set(true);
                                        let result = register_user(RegisterInput {
                                            username: register_username(),
                                            password: register_password(),
                                        })
                                        .await;
                                        busy.set(false);

                                        match result {
                                            Ok(user) => {
                                                register_password.set(String::new());
                                                login_password.set(String::new());
                                                status.set(Some((
                                                    true,
                                                    format!("Benutzer {} wurde angelegt und eingeloggt.", user.username),
                                                )));
                                                auth_refresh.with_mut(|value| *value += 1);
                                            }
                                            Err(error) => {
                                                status.set(Some((
                                                    false,
                                                    format!("Registrierung fehlgeschlagen: {error}"),
                                                )));
                                            }
                                        }
                                    });
                                },
                                {if busy() { "Speichert..." } else { "Account anlegen" }}
                            }
                        }
                    }

                    Card {
                        class: "auth-card",
                        CardHeader {
                            div { class: "auth-card-title-row",
                                CardTitle { "Login" }
                                CardAction {
                                    Badge { variant: BadgeVariant::Outline, "Bestehend" }
                                }
                            }
                            CardDescription {
                                "Melde dich mit deinem Benutzernamen und Passwort an."
                            }
                        }
                        CardContent {
                            div { class: "auth-form",
                                div { class: "auth-field",
                                    Label { html_for: "login-username", "Benutzername" }
                                    Input {
                                        id: "login-username",
                                        value: login_username(),
                                        placeholder: "Benutzername",
                                        disabled: busy(),
                                        oninput: move |event: FormEvent| login_username.set(event.value()),
                                    }
                                }
                                div { class: "auth-field",
                                    Label { html_for: "login-password", "Passwort" }
                                    Input {
                                        id: "login-password",
                                        r#type: "password",
                                        value: login_password(),
                                        placeholder: "Passwort",
                                        disabled: busy(),
                                        oninput: move |event: FormEvent| login_password.set(event.value()),
                                    }
                                }
                            }
                        }
                        CardFooter {
                            Button {
                                variant: ButtonVariant::Secondary,
                                style: "width: 100%;",
                                disabled: busy(),
                                onclick: move |_| {
                                    if busy() {
                                        return;
                                    }

                                    spawn(async move {
                                        busy.set(true);
                                        let result = login_user(LoginInput {
                                            username: login_username(),
                                            password: login_password(),
                                        })
                                        .await;
                                        busy.set(false);

                                        match result {
                                            Ok(user) => {
                                                login_password.set(String::new());
                                                register_password.set(String::new());
                                                status.set(Some((
                                                    true,
                                                    format!("Willkommen zurück, {}.", user.username),
                                                )));
                                                auth_refresh.with_mut(|value| *value += 1);
                                            }
                                            Err(error) => {
                                                status.set(Some((false, format!("Login fehlgeschlagen: {error}"))));
                                            }
                                        }
                                    });
                                },
                                {if busy() { "Prüft..." } else { "Einloggen" }}
                            }
                        }
                    }
                }
            }

            if let Some((success, message)) = status() {
                div {
                    class: "auth-status",
                    Badge {
                        variant: if success {
                            BadgeVariant::Secondary
                        } else {
                            BadgeVariant::Destructive
                        },
                        {if success { "Status" } else { "Fehler" }}
                    }
                    p {
                        class: "auth-help",
                        "{message}"
                    }
                }
            }
        }
    }
}

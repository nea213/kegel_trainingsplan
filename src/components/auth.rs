use crate::auth::{
    current_user, login_user, logout_user, register_user, LoginInput, PublicUser, RegisterInput,
};
use crate::components::ui::avatar::{Avatar, AvatarFallback, AvatarImageSize};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::{
    Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,
};
use crate::components::ui::input::Input;
use crate::components::ui::label::Label;
use crate::components::ui::separator::Separator;
use crate::theme::ThemeContext;
use crate::Route;
use dioxus::prelude::*;

const AUTH_CSS: Asset = asset!("/assets/styling/auth.css");

pub(crate) fn sanitize_return_to(return_to: Option<String>) -> Option<String> {
    return_to.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty()
            || !trimmed.starts_with('/')
            || trimmed.starts_with("//")
            || trimmed.contains("://")
            || trimmed.starts_with("/login")
            || trimmed.starts_with("/register")
        {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(crate) fn use_auth_user_resource() -> Result<Resource<Option<PublicUser>>, RenderError> {
    let auth_refresh = use_context::<Signal<u64>>();
    use_server_future(move || {
        let _ = auth_refresh();
        async move { current_user().await.ok().flatten() }
    })
}

fn user_initials(user: &PublicUser) -> String {
    user.username
        .chars()
        .filter(|char| char.is_alphanumeric())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

#[component]
pub fn AccountPanel() -> Element {
    let auth_resource = use_auth_user_resource()?;
    let auth_state = auth_resource.read().as_ref().cloned();
    let mut auth_refresh = use_context::<Signal<u64>>();
    let theme = use_context::<ThemeContext>();
    let nav = navigator();
    let mut status = use_signal(|| None::<(bool, String)>);
    let mut busy = use_signal(|| false);

    let theme_sync_target = auth_state.as_ref().and_then(|state| state.clone());

    use_effect(move || {
        if let Some(user) = theme_sync_target.clone() {
            if theme.needs_sync(user.id) {
                theme.sync_authenticated_user(user.id, user.theme_mode);
            }
        }
    });

    match auth_state {
        None => rsx! {
            document::Link { rel: "stylesheet", href: AUTH_CSS }
            div { id: "auth-panel", class: "auth-panel auth-panel--account" }
        },
        Some(None) => rsx! {
            document::Link { rel: "stylesheet", href: AUTH_CSS }
            div { id: "auth-panel", class: "auth-panel auth-panel--account" }
        },
        Some(Some(user)) => {
            let initials = user_initials(&user);

            rsx! {
                document::Link { rel: "stylesheet", href: AUTH_CSS }

                div {
                    id: "auth-panel",
                    class: "auth-panel auth-panel--account",
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
                                    AvatarFallback { "{initials}" }
                                }
                                div { class: "auth-user-meta",
                                    p { class: "auth-user-copy", "Angemeldet als {user.username}" }
                                    Badge { variant: BadgeVariant::Outline, "Server-Session" }
                                }
                            }
                            Separator { horizontal: true, decorative: true, style: "margin: 1rem 0;" }
                            p {
                                class: "auth-help",
                                "Du kannst dich jederzeit wieder ausloggen. Danach wirst du direkt wieder zur Login-Seite geleitet."
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

                                    let nav = nav;
                                    spawn(async move {
                                        busy.set(true);
                                        let result = logout_user().await;
                                        busy.set(false);

                                        match result {
                                            Ok(()) => {
                                                auth_refresh.with_mut(|value| *value += 1);
                                                theme.reset_to_system();
                                                let _ = nav.replace(Route::Login { return_to: None });
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
                            p { class: "auth-help", "{message}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn LoginPanel(return_to: Option<String>) -> Element {
    let mut auth_refresh = use_context::<Signal<u64>>();
    let theme = use_context::<ThemeContext>();
    let nav = navigator();
    let login_target = return_to.clone();
    let register_target = return_to.clone();
    let mut status = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);
    let mut login_username = use_signal(String::new);
    let mut login_password = use_signal(String::new);

    rsx! {
        document::Link { rel: "stylesheet", href: AUTH_CSS }

        div {
            id: "auth-panel",
            class: "auth-panel auth-panel--public",
            Card {
                class: "auth-card auth-card--wide",
                CardHeader {
                    div { class: "auth-card-title-row",
                        CardTitle { "Login" }
                        CardAction {
                            Badge { variant: BadgeVariant::Outline, "Bestehend" }
                        }
                    }
                    CardDescription {
                        "Melde dich mit deinem Benutzernamen und Passwort an, um zu deiner zuletzt angeforderten Seite zurueckzukehren."
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
                CardFooter { style: "flex-direction: column; align-items: stretch; gap: 0.75rem; margin-top: 0.75rem;",
                    Button {
                        variant: ButtonVariant::Secondary,
                        style: "width: 100%;",
                        disabled: busy(),
                        onclick: move |_| {
                            if busy() {
                                return;
                            }

                            let nav = nav;
                            let target = sanitize_return_to(login_target.clone());
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
                                        theme.set_authenticated_user_theme(user.id, user.theme_mode);
                                        login_password.set(String::new());
                                        auth_refresh.with_mut(|value| *value += 1);
                                        if let Some(path) = target {
                                            let _ = nav.replace(path);
                                        } else {
                                            let _ = nav.replace(Route::Home {});
                                        }
                                    }
                                    Err(error) => {
                                        status.set(Some(format!("Login fehlgeschlagen: {error}")));
                                    }
                                }
                            });
                        },
                        {if busy() { "Prueft..." } else { "Einloggen" }}
                    }
                    Button {
                        variant: ButtonVariant::Link,
                        onclick: move |_| {
                            let _ = nav.push(Route::Register {
                                return_to: register_target.clone(),
                            });
                        },
                        "Noch kein Konto? Jetzt registrieren"
                    }
                }
            }

            if let Some(message) = status() {
                div {
                    class: "auth-status",
                    Badge { variant: BadgeVariant::Destructive, "Fehler" }
                    p { class: "auth-help", "{message}" }
                }
            }
        }
    }
}

#[component]
pub fn RegisterPanel(return_to: Option<String>) -> Element {
    let mut auth_refresh = use_context::<Signal<u64>>();
    let theme = use_context::<ThemeContext>();
    let nav = navigator();
    let register_target = return_to.clone();
    let login_target = return_to.clone();
    let mut status = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);
    let mut register_username = use_signal(String::new);
    let mut register_password = use_signal(String::new);

    rsx! {
        document::Link { rel: "stylesheet", href: AUTH_CSS }

        div {
            id: "auth-panel",
            class: "auth-panel auth-panel--public",
            Card {
                class: "auth-card auth-card--wide",
                CardHeader {
                    div { class: "auth-card-title-row",
                        CardTitle { "Registrieren" }
                        CardAction {
                            Badge { variant: BadgeVariant::Secondary, "Neu" }
                        }
                    }
                    CardDescription {
                        "Lege einen neuen Benutzer an. Nach erfolgreicher Registrierung wirst du direkt eingeloggt."
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
                CardFooter { style: "flex-direction: column; align-items: stretch; gap: 0.75rem; margin-top: 0.75rem;",
                    Button {
                        style: "width: 100%;",
                        disabled: busy(),
                        onclick: move |_| {
                            if busy() {
                                return;
                            }

                            let nav = nav;
                            let target = sanitize_return_to(register_target.clone());
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
                                        theme.set_authenticated_user_theme(user.id, user.theme_mode);
                                        register_password.set(String::new());
                                        auth_refresh.with_mut(|value| *value += 1);
                                        if let Some(path) = target {
                                            let _ = nav.replace(path);
                                        } else {
                                            let _ = nav.replace(Route::Home {});
                                        }
                                    }
                                    Err(error) => {
                                        status.set(Some(format!("Registrierung fehlgeschlagen: {error}")));
                                    }
                                }
                            });
                        },
                        {if busy() { "Speichert..." } else { "Account anlegen" }}
                    }
                    Button {
                        variant: ButtonVariant::Link,
                        onclick: move |_| {
                            let _ = nav.push(Route::Login {
                                return_to: login_target.clone(),
                            });
                        },
                        "Schon ein Konto? Zum Login"
                    }
                }
            }

            if let Some(message) = status() {
                div {
                    class: "auth-status",
                    Badge { variant: BadgeVariant::Destructive, "Fehler" }
                    p { class: "auth-help", "{message}" }
                }
            }
        }
    }
}

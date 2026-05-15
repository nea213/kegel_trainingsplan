use crate::auth::{current_user, login_user, logout_user, register_user, LoginInput, RegisterInput};
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

    let mut status = use_signal(|| None::<String>);
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
                div {
                    class: "auth-card auth-card--wide",
                    h2 { "Willkommen, {user.username}" }
                    p {
                        class: "auth-copy",
                        "Deine Session liegt serverseitig in SQLite und wird über Cookies für Web und Mobile wiederverwendet."
                    }
                    div {
                        class: "auth-actions",
                        button {
                            class: "auth-button auth-button--secondary",
                            disabled: busy(),
                            onclick: move |_| async move {
                                busy.set(true);
                                let result = logout_user().await;
                                busy.set(false);

                                match result {
                                    Ok(()) => {
                                        status.set(Some("Du wurdest erfolgreich ausgeloggt.".to_string()));
                                        auth_refresh.with_mut(|value| *value += 1);
                                    }
                                    Err(error) => {
                                        status.set(Some(format!("Logout fehlgeschlagen: {error}")));
                                    }
                                }
                            },
                            {if busy() { "Logout läuft..." } else { "Logout" }}
                        }
                    }
                }
            } else {
                div {
                    class: "auth-grid",
                    section {
                        class: "auth-card",
                        h2 { "Registrieren" }
                        p {
                            class: "auth-copy",
                            "Lege deinen ersten Benutzer an. Nach erfolgreicher Registrierung loggen wir dich direkt ein."
                        }
                        input {
                            class: "auth-input",
                            value: register_username(),
                            placeholder: "Benutzername",
                            disabled: busy(),
                            oninput: move |event| register_username.set(event.value()),
                        }
                        input {
                            class: "auth-input",
                            r#type: "password",
                            value: register_password(),
                            placeholder: "Passwort",
                            disabled: busy(),
                            oninput: move |event| register_password.set(event.value()),
                        }
                        button {
                            class: "auth-button",
                            disabled: busy(),
                            onclick: move |_| async move {
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
                                        status.set(Some(format!("Benutzer {} wurde angelegt und eingeloggt.", user.username)));
                                        auth_refresh.with_mut(|value| *value += 1);
                                    }
                                    Err(error) => {
                                        status.set(Some(format!("Registrierung fehlgeschlagen: {error}")));
                                    }
                                }
                            },
                            {if busy() { "Speichert..." } else { "Account anlegen" }}
                        }
                    }

                    section {
                        class: "auth-card",
                        h2 { "Login" }
                        p {
                            class: "auth-copy",
                            "Melde dich mit deinem Benutzernamen und Passwort an."
                        }
                        input {
                            class: "auth-input",
                            value: login_username(),
                            placeholder: "Benutzername",
                            disabled: busy(),
                            oninput: move |event| login_username.set(event.value()),
                        }
                        input {
                            class: "auth-input",
                            r#type: "password",
                            value: login_password(),
                            placeholder: "Passwort",
                            disabled: busy(),
                            oninput: move |event| login_password.set(event.value()),
                        }
                        button {
                            class: "auth-button auth-button--secondary",
                            disabled: busy(),
                            onclick: move |_| async move {
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
                                        status.set(Some(format!("Willkommen zurück, {}.", user.username)));
                                        auth_refresh.with_mut(|value| *value += 1);
                                    }
                                    Err(error) => {
                                        status.set(Some(format!("Login fehlgeschlagen: {error}")));
                                    }
                                }
                            },
                            {if busy() { "Prüft..." } else { "Einloggen" }}
                        }
                    }
                }
            }

            if let Some(message) = status() {
                p {
                    class: "auth-status",
                    "{message}"
                }
            }
        }
    }
}

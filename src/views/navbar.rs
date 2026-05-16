use crate::components::auth::sanitize_return_to;
use crate::components::ui::avatar::{Avatar, AvatarFallback, AvatarImageSize};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::navbar::{Navbar as UiNavbar, NavbarItem};
use crate::{auth::current_user, Route};
use dioxus::prelude::*;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

#[component]
pub fn Navbar() -> Element {
    let auth_refresh = use_context::<Signal<u64>>();
    let nav = navigator();
    let current_route = router().full_route_string();
    let user_resource = use_server_future(move || {
        let _ = auth_refresh();
        async move { current_user().await.ok().flatten() }
    })?;
    let user_state = user_resource.read().as_ref().cloned();
    let user = user_state.clone().flatten();
    let login_target = sanitize_return_to(Some(current_route));
    let should_redirect_login = matches!(user_state, Some(None));

    use_effect(move || {
        if should_redirect_login {
            let _ = nav.replace(Route::Login {
                return_to: login_target.clone(),
            });
        }
    });

    let user_initials = user
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

    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }

        match user_state {
            None => rsx! {
                header {
                    id: "navbar",
                    div { class: "nav-brand",
                        h1 { class: "nav-title", "Kegel Trainingsplan" }
                        p { class: "nav-subtitle", "Authentifizierung wird geprüft..." }
                    }
                }
            },
            Some(None) => rsx! {
                section {
                    id: "home-intro",
                    div { class: "auth-status",
                        p { class: "auth-help", "Du wirst zur Login-Seite weitergeleitet..." }
                    }
                }
            },
            Some(Some(user)) => rsx! {
                header {
                    id: "navbar",
                    div { class: "nav-brand",
                        h1 { class: "nav-title", "Kegel Trainingsplan" }
                        p { class: "nav-subtitle", "Trainingsplanung und Benutzerverwaltung auf einer gemeinsamen Dioxus-UI-Basis." }
                    }
                    div { class: "nav-main",
                        UiNavbar {
                            aria_label: "Hauptnavigation",
                            style: "background: #151a24; padding: 0.35rem; border-radius: 0.9rem; box-shadow: inset 0 0 0 1px #2b3447;",
                            NavbarItem {
                                index: 0usize,
                                value: "home".to_string(),
                                to: Route::Home {},
                                "Startseite"
                            }
                        }
                    }
                    div {
                        class: "nav-session",
                        div { class: "nav-user",
                            Avatar { size: AvatarImageSize::Small, aria_label: "Angemeldeter Benutzer",
                                AvatarFallback { "{user_initials}" }
                            }
                            div { class: "nav-user-copy",
                                span { class: "nav-user-name", "{user.username}" }
                                span { class: "nav-user-state", "Session aktiv" }
                            }
                            Badge { variant: BadgeVariant::Secondary, "Online" }
                        }
                    }
                }
                Outlet::<Route> {}
            },
        }
    }
}

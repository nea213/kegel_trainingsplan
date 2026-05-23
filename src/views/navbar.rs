use crate::components::auth::sanitize_return_to;
use crate::components::ui::avatar::{Avatar, AvatarFallback, AvatarImageSize};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use crate::components::ui::navbar::{Navbar as UiNavbar, NavbarItem};
use crate::components::ui::separator::Separator;
use crate::theme::{ThemeContext, ThemeMode};
use crate::{
    auth::current_user, auth::logout_user, auth::update_theme_mode, auth::PublicUser, Route,
};
use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    let auth_refresh = use_context::<Signal<u64>>();
    let theme = use_context::<ThemeContext>();
    let nav = navigator();
    let current_route = router().full_route_string();
    let user_resource = use_server_future(move || {
        let _ = auth_refresh();
        async move { current_user().await.ok().flatten() }
    })?;
    let user_state = user_resource.read().as_ref().cloned();
    let user = user_state.clone().flatten();
    let theme_sync_target = user.clone();
    let login_target = sanitize_return_to(Some(current_route));
    let should_redirect_login = matches!(user_state, Some(None));
    let redirect_nav = nav.clone();

    use_effect(move || {
        if let Some(user) = theme_sync_target.clone() {
            if theme.needs_sync(user.id) {
                theme.sync_authenticated_user(user.id, user.theme_mode);
            }
        }

        if should_redirect_login {
            let _ = redirect_nav.replace(Route::Login {
                return_to: login_target.clone(),
            });
        }
    });

    rsx! {
        match user_state {
            None => rsx! {
                header {
                    id: "navbar",
                    class: "navbar-shell navbar-shell--loading",
                    div { class: "nav-brand",
                        div { class: "nav-brand__badge", "Kegel Trainingsplan" }
                        h1 { class: "nav-title", "Kegel Trainingsplan" }
                        p { class: "nav-subtitle", "Authentifizierung wird geprüft..." }
                    }
                }
            },
            Some(None) => rsx! {
                section {
                    id: "home-intro",
                    div { class: "auth-status nav-redirect-state",
                        p { class: "auth-help", "Du wirst zur Login-Seite weitergeleitet..." }
                    }
                }
            },
            Some(Some(user)) => rsx! {
                header {
                    id: "navbar",
                    class: "navbar-shell",
                    div { class: "nav-spacer", aria_hidden: "true" }
                    div { class: "nav-main",
                        UiNavbar {
                            aria_label: "Hauptnavigation",
                            NavbarItem {
                                index: 0usize,
                                value: "dashboard".to_string(),
                                to: Route::Dashboard {},
                                "Startseite"
                            }
                            if user.is_system_admin {
                                NavbarItem {
                                    index: 1usize,
                                    value: "clubs".to_string(),
                                    to: Route::Clubs {},
                                    "Vereine"
                                }
                            }
                        }
                    }
                    div { class: "nav-session",
                        UserMenu { user }
                    }
                }
                Outlet::<Route> {}
            },
        }
    }
}

#[component]
fn UserMenu(user: PublicUser) -> Element {
    let mut auth_refresh = use_context::<Signal<u64>>();
    let theme = use_context::<ThemeContext>();
    let nav = navigator();
    let mut menu_busy = use_signal(|| false);
    let mut menu_status = use_signal(|| None::<String>);
    let active_theme = theme.current();
    let username = user.username.clone();
    let trigger_username = username.clone();
    let header_username = username.clone();
    let user_initials = user
        .username
        .chars()
        .filter(|character| character.is_alphanumeric())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    let user_initials = if user_initials.is_empty() {
        "KT".to_string()
    } else {
        user_initials
    };
    let trigger_initials = user_initials.clone();

    rsx! {
        DropdownMenu {
            DropdownMenuTrigger {
                as: move |attributes: Vec<Attribute>| rsx! {
                    button {
                        class: "nav-user-menu-trigger",
                        r#type: "button",
                        ..attributes,
                        div { class: "nav-user",
                            Avatar { size: AvatarImageSize::Small, aria_label: "Angemeldeter Benutzer",
                                AvatarFallback { "{trigger_initials}" }
                            }
                            div { class: "nav-user-copy",
                                span { class: "nav-user-kicker", "Angemeldet" }
                                span { class: "nav-user-name", "{trigger_username}" }
                            }
                            span { class: "nav-user-chevron", aria_hidden: "true" }
                        }
                    }
                },
            }
            DropdownMenuContent {
                class: "nav-user-menu",
                style: "left: auto; right: 0;",
                div { class: "nav-user-menu-header",
                    span { class: "nav-user-menu-title", "{header_username}" }
                    span { class: "nav-user-menu-subtitle", "Persönliches Erscheinungsbild" }
                }
                Separator { class: "nav-user-menu-separator", decorative: true }
                div { class: "nav-user-menu-section-label", "Theme" }
                for (index, option) in [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System].into_iter().enumerate() {
                    DropdownMenuItem {
                        index,
                        value: option,
                        disabled: menu_busy(),
                        on_select: move |selected| {
                            if menu_busy() {
                                return;
                            }

                            if selected == active_theme {
                                menu_status.set(None);
                                return;
                            }

                            let previous_theme = active_theme;
                            theme.set_authenticated_user_theme(user.id, selected);
                            menu_status.set(None);
                            spawn(async move {
                                menu_busy.set(true);
                                let result = update_theme_mode(selected).await;
                                menu_busy.set(false);

                                match result {
                                    Ok(updated_user) => {
                                        theme.set_authenticated_user_theme(updated_user.id, updated_user.theme_mode);
                                        auth_refresh.with_mut(|value| *value += 1);
                                    }
                                    Err(error) => {
                                        theme.set_authenticated_user_theme(user.id, previous_theme);
                                        menu_status.set(Some(format!("Theme konnte nicht gespeichert werden: {error}")));
                                    }
                                }
                            });
                        },
                        div { class: "nav-user-menu-item-copy",
                            span { class: "nav-user-menu-item-title", "{option.label()}" }
                            span { class: "nav-user-menu-item-description", "{option.description()}" }
                        }
                        if option == active_theme {
                            span { class: "nav-user-menu-item-state", "Aktuell" }
                        }
                    }
                }
                Separator { class: "nav-user-menu-separator", decorative: true }
                Button {
                    class: "nav-user-menu-logout",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Sm,
                    disabled: menu_busy(),
                    onclick: move |_| {
                        if menu_busy() {
                            return;
                        }

                        menu_status.set(None);
                        let nav = nav.clone();
                        spawn(async move {
                            menu_busy.set(true);
                            let result = logout_user().await;
                            menu_busy.set(false);

                            match result {
                                Ok(()) => {
                                    auth_refresh.with_mut(|value| *value += 1);
                                    theme.reset_to_system();
                                    let _ = nav.replace(Route::Login { return_to: None });
                                }
                                Err(error) => {
                                    menu_status.set(Some(format!("Logout fehlgeschlagen: {error}")));
                                }
                            }
                        });
                    },
                    {if menu_busy() { "Speichert..." } else { "Abmelden" }}
                }
                if let Some(message) = menu_status() {
                    p { class: "nav-user-menu-error", "{message}" }
                }
            }
        }
    }
}

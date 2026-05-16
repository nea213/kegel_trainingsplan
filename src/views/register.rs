use crate::components::auth::{sanitize_return_to, use_auth_user_resource};
use crate::components::RegisterPanel;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Register(return_to: Option<String>) -> Element {
    let auth_resource = use_auth_user_resource()?;
    let auth_state = auth_resource.read().as_ref().cloned();
    let nav = navigator();
    let should_redirect_home = matches!(auth_state, Some(Some(_)));

    use_effect(move || {
        if should_redirect_home {
            let _ = nav.replace(Route::Home {});
        }
    });

    match auth_state {
        None => rsx! {
            section { class: "auth-page",
                div { class: "auth-page-shell",
                    div {
                        id: "auth-panel",
                        div { class: "auth-status",
                            p { class: "auth-help", "Registrierung wird vorbereitet..." }
                        }
                    }
                }
            }
        },
        Some(Some(_)) => rsx! {},
        Some(None) => rsx! {
            section { class: "auth-page",
                div { class: "auth-page-shell",
                    RegisterPanel {
                        return_to: sanitize_return_to(return_to),
                    }
                }
            }
        },
    }
}

use crate::components::auth::{sanitize_return_to, use_auth_user_resource};
use crate::components::RegisterPanel;
use crate::theme::ThemeContext;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Register(return_to: Option<String>) -> Element {
    let auth_resource = use_auth_user_resource()?;
    let auth_state = auth_resource.read().as_ref().cloned();
    let theme = use_context::<ThemeContext>();
    let nav = navigator();
    let should_redirect_home = matches!(auth_state, Some(Some(_)));

    use_effect(move || {
        theme.reset_to_system();
        if should_redirect_home {
            let _ = nav.replace(Route::Home {});
        }
    });

    if matches!(auth_state, Some(Some(_))) {
        return rsx! {};
    }

    rsx! {
        section { class: "auth-page",
            div { class: "auth-page-shell",
                RegisterPanel {
                    return_to: sanitize_return_to(return_to),
                }
            }
        }
    }
}

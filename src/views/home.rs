use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    let nav = navigator();

    use_effect(move || {
        let _ = nav.replace(Route::Dashboard {});
    });

    rsx! {
        section { id: "home-intro",
            div { class: "auth-status",
                p { class: "auth-help", "Dashboard wird vorbereitet..." }
            }
        }
    }
}

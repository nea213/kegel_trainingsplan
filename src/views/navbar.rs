use crate::{auth::current_user, Route};
use dioxus::prelude::*;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

#[component]
pub fn Navbar() -> Element {
    let auth_refresh = use_context::<Signal<u64>>();
    let user_resource = use_server_future(move || {
        let _ = auth_refresh();
        async move { current_user().await.ok().flatten() }
    })?;
    let user = user_resource.read().as_ref().cloned().flatten();

    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }

        header {
            id: "navbar",
            h1 { class: "nav-title", "Kegel Trainingsplan" }
            div {
                class: "nav-session",
                if let Some(user) = user {
                    span { "👤 {user.username}" }
                } else {
                    span { "Nicht eingeloggt" }
                }
            }
        }

        Outlet::<Route> {}
    }
}

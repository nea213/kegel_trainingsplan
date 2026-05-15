use crate::{auth::current_user, Route};
use dioxus::prelude::*;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

/// The Navbar component that will be rendered on all pages of our app since every page is under the layout.
///
///
/// This layout component wraps the UI of [Route::Home] and [Route::Blog] in a common navbar. The contents of the Home and Blog
/// routes will be rendered under the outlet inside this component
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

        div {
            id: "navbar",
            div {
                class: "nav-links",
                Link {
                    to: Route::Home {},
                    "Home"
                }
                Link {
                    to: Route::Blog { id: 1 },
                    "Blog"
                }
            }
            div {
                class: "nav-session",
                if let Some(user) = user {
                    span { "👤 {user.username}" }
                } else {
                    span { "Gastmodus" }
                }
            }
        }

        // The `Outlet` component is used to render the next component inside the layout. In this case, it will render either
        // the [`Home`] or [`Blog`] component depending on the current route.
        Outlet::<Route> {}
    }
}

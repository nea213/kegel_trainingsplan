use dioxus::prelude::*;

use views::{Home, Navbar};

mod auth;
mod components;
#[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod server;
mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");

fn main() {
    dioxus_cookie::init();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let auth_refresh = use_signal(|| 0_u64);
    use_context_provider(|| auth_refresh);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

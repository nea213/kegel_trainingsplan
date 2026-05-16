use dioxus::prelude::*;

use views::{Home, Login, Navbar, Register};

mod auth;
mod components;
#[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod server;
mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/login?:return_to")]
    Login { return_to: Option<String> },

    #[route("/register?:return_to")]
    Register { return_to: Option<String> },

    #[layout(Navbar)]
        #[route("/")]
        Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const DX_COMPONENTS_THEME_CSS: Asset = asset!("/assets/dx-components-theme.css");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

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
        document::Link { rel: "stylesheet", href: DX_COMPONENTS_THEME_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

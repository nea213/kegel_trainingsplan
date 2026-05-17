use dioxus::prelude::*;

use theme::{ThemeContext, ThemeMode};
use views::{ClubDetail, Clubs, Home, Login, Navbar, Register};

mod auth;
mod clubs;
mod components;
mod group_trainers;
mod groups;
#[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod server;
mod team_players;
mod teams;
mod theme;
mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/login?:return_to")]
    Login { return_to: Option<String> },

    #[route("/register?:return_to")]
    Register { return_to: Option<String> },

    #[layout(Navbar)]
        #[route("/clubs")]
        Clubs {},

        #[route("/clubs/:club_id")]
        ClubDetail { club_id: i32 },

        #[route("/")]
        Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const DX_COMPONENTS_THEME_CSS: Asset = asset!("/assets/dx-components-theme.css");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let _ = dotenvy::dotenv();

    dioxus_cookie::init();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let auth_refresh = use_signal(|| 0_u64);
    let theme_mode = use_signal(ThemeMode::default);
    let synced_user_id = use_signal(|| None::<i32>);
    use_context_provider(|| auth_refresh);
    use_context_provider(|| ThemeContext {
        mode: theme_mode,
        synced_user_id,
    });

    use_effect(move || {
        let js_code = match theme_mode().document_theme() {
            Some(theme) => format!(
                r#"
                document.documentElement.setAttribute("data-theme", "{theme}");
                "#
            ),
            None => r#"
                document.documentElement.removeAttribute("data-theme");
            "#
            .to_string(),
        };

        spawn(async move {
            let _ = document::eval(&js_code);
        });
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: DX_COMPONENTS_THEME_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

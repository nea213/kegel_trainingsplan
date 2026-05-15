use crate::components::AuthPanel;
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        section {
            id: "home-intro",
            h1 { "Auth & User" }
            p {
                "Benutzer, Sessions, Registrierung und Login laufen vollständig über die serverseitige Auth-Logik."
            }
        }

        AuthPanel {}
    }
}

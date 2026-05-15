use crate::components::{AuthPanel, Hero};
use dioxus::prelude::*;

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Home() -> Element {
    rsx! {
        Hero {}

        section {
            id: "home-intro",
            h1 { "SeaORM + SQLite Auth" }
            p {
                "Die App speichert Benutzer und Sessions jetzt serverseitig in SQLite und stellt Login/Logout über Dioxus-Serverfunktionen bereit."
            }
        }

        AuthPanel {}
    }
}

use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemGroup, ItemSeparator, ItemTitle,
};
use crate::components::ui::separator::Separator;
use crate::components::AuthPanel;
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        section {
            id: "home-intro",
            Card { class: "home-intro-card",
                CardHeader {
                    div { class: "home-badges",
                        Badge { "Serverseitige Auth" }
                        Badge { variant: BadgeVariant::Outline, "SQLite Sessions" }
                        Badge { variant: BadgeVariant::Secondary, "Dioxus UI" }
                    }
                    CardTitle { "Auth & User" }
                    CardDescription {
                        "Benutzer, Sessions, Registrierung und Login laufen vollständig über die serverseitige Auth-Logik."
                    }
                }
                CardContent {
                    ItemGroup { class: "home-feature-list",
                        Item {
                            ItemContent {
                                ItemTitle { "Registrierung und Login" }
                                ItemDescription {
                                    "Neue Accounts werden angelegt und bestehende Benutzer melden sich mit derselben Server-Session wieder an."
                                }
                            }
                            ItemActions {
                                Badge { variant: BadgeVariant::Secondary, "Aktiv" }
                            }
                        }
                        ItemSeparator {}
                        Item {
                            ItemContent {
                                ItemTitle { "Cookie-basierte Session" }
                                ItemDescription {
                                    "Der eingeloggte Zustand wird serverseitig gespeichert und in Web sowie Mobile wiederverwendet."
                                }
                            }
                            ItemActions {
                                Badge { variant: BadgeVariant::Outline, "SQLite" }
                            }
                        }
                        ItemSeparator {}
                        Item {
                            ItemContent {
                                ItemTitle { "Offizielle Dioxus-Komponenten" }
                                ItemDescription {
                                    "Die Formulare und Navigationsbausteine auf dieser Seite nutzen jetzt die offiziellen Komponenten als lokale UI-Basis."
                                }
                            }
                            ItemActions {
                                Badge { "Neu" }
                            }
                        }
                    }
                }
            }
        }

        Separator {
            class: "home-divider",
            horizontal: true,
            decorative: true,
        }

        AuthPanel {}
    }
}

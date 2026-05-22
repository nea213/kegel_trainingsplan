use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::skeleton::Skeleton;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusBannerTone {
    Info,
    Success,
    Error,
}

impl StatusBannerTone {
    fn class(self) -> &'static str {
        match self {
            Self::Info => "status-banner--info",
            Self::Success => "status-banner--success",
            Self::Error => "status-banner--error",
        }
    }
}

#[component]
pub fn PageHeader(
    title: String,
    description: String,
    #[props(default)] eyebrow: Option<String>,
    #[props(default)] actions: Option<Element>,
) -> Element {
    rsx! {
        section { class: "page-header",
            div { class: "page-header__body",
                if let Some(eyebrow) = eyebrow {
                    p { class: "page-header__eyebrow", "{eyebrow}" }
                }
                h1 { class: "page-header__title", "{title}" }
                p { class: "page-header__description", "{description}" }
            }
            if let Some(actions) = actions {
                div { class: "page-header__actions", {actions} }
            }
        }
    }
}

#[component]
pub fn SectionPanel(
    title: String,
    description: String,
    #[props(default)] actions: Option<Element>,
    children: Element,
) -> Element {
    rsx! {
        Card { class: "surface-card section-panel",
            CardHeader { class: "section-panel__header",
                div { class: "section-panel__heading",
                    CardTitle { "{title}" }
                    CardDescription { "{description}" }
                }
                if let Some(actions) = actions {
                    div { class: "section-panel__actions", {actions} }
                }
            }
            CardContent { class: "section-panel__content",
                {children}
            }
        }
    }
}

#[component]
pub fn StatusBanner(
    tone: StatusBannerTone,
    message: String,
    #[props(default)] title: Option<String>,
) -> Element {
    rsx! {
        div { class: format!("status-banner {}", tone.class()),
            if let Some(title) = title {
                p { class: "status-banner__title", "{title}" }
            }
            p { class: "status-banner__message", "{message}" }
        }
    }
}

#[component]
pub fn EmptyStatePanel(
    title: String,
    message: String,
    #[props(default)] action: Option<Element>,
) -> Element {
    rsx! {
        div { class: "empty-state",
            p { class: "empty-state__title", "{title}" }
            p { class: "empty-state__message", "{message}" }
            if let Some(action) = action {
                div { class: "empty-state__action", {action} }
            }
        }
    }
}

#[component]
pub fn MetricCard(
    label: String,
    value: String,
    #[props(default)] detail: Option<String>,
) -> Element {
    rsx! {
        Card { class: "surface-card metric-card",
            CardContent { class: "metric-card__content",
                p { class: "metric-card__label", "{label}" }
                p { class: "metric-card__value", "{value}" }
                if let Some(detail) = detail {
                    p { class: "metric-card__detail", "{detail}" }
                }
            }
        }
    }
}

#[component]
pub fn LoadingPanel(title: String, #[props(default = 3)] lines: usize) -> Element {
    rsx! {
        Card { class: "surface-card section-panel",
            CardHeader {
                CardTitle { "{title}" }
                CardDescription { "Inhalte werden geladen..." }
            }
            CardContent { class: "loading-panel",
                for index in 0..lines {
                    Skeleton {
                        key: "{index}",
                        class: if index == 0 {
                            "loading-panel__line loading-panel__line--wide"
                        } else {
                            "loading-panel__line"
                        },
                    }
                }
            }
        }
    }
}

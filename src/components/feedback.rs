use crate::components::ui::alert_dialog::{
    AlertDialog, AlertDialogAction, AlertDialogActions, AlertDialogCancel,
    AlertDialogDescription, AlertDialogTitle,
};
use crate::components::ui::button::{Button, ButtonVariant};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, Toasts};

#[component]
pub fn ConfirmActionDialog(
    open: bool,
    title: String,
    description: String,
    confirm_label: String,
    busy: bool,
    on_open_change: EventHandler<bool>,
    on_confirm: EventHandler<()>,
) -> Element {
    rsx! {
        AlertDialog {
            open,
            on_open_change: move |value| on_open_change.call(value),
            AlertDialogTitle { "{title}" }
            AlertDialogDescription { "{description}" }
            AlertDialogActions {
                AlertDialogCancel { "Abbrechen" }
                AlertDialogAction {
                    on_click: move |_| on_confirm.call(()),
                    if busy { "Wird ausgeführt..." } else { "{confirm_label}" }
                }
            }
        }
    }
}

#[component]
pub fn CopyableCodeCard(
    title: String,
    description: String,
    code: String,
    copy_label: String,
    on_copy: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "detail-card",
            div { class: "detail-card-copy",
                p { class: "section-label", "{title}" }
                p { class: "detail-card-title code-result-card__code", "{code}" }
                p { class: "section-meta", "{description}" }
            }
            div { class: "section-actions section-actions--stack mobile-form-actions",
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| on_copy.call(code.clone()),
                    "{copy_label}"
                }
            }
        }
    }
}

pub fn show_success_toast(toast: Toasts, title: impl Into<String>, description: impl Into<String>) {
    toast.success(
        title.into(),
        ToastOptions::new().description(description.into()),
    );
}

pub fn show_error_toast(toast: Toasts, title: impl Into<String>, description: impl Into<String>) {
    toast.error(
        title.into(),
        ToastOptions::new().description(description.into()),
    );
}

pub async fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut eval = document::eval(
        r#"
        let value = await dioxus.recv();
        try {
            if (!navigator.clipboard || !navigator.clipboard.writeText) {
                throw new Error("Die Zwischenablage ist in diesem Browser nicht verfügbar.");
            }
            await navigator.clipboard.writeText(value);
            dioxus.send("ok");
        } catch (error) {
            dioxus.send(error?.message ?? "Der Code konnte nicht kopiert werden.");
        }
        "#,
    );

    eval.send(text)
        .map_err(|error| format!("Der Kopiervorgang konnte nicht gestartet werden: {error}"))?;

    match eval
        .recv::<String>()
        .await
        .map_err(|error| format!("Die Rückmeldung zum Kopiervorgang fehlt: {error}"))?
        .as_str()
    {
        "ok" => Ok(()),
        error => Err(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copyable_code_card_renders_code_and_button() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                CopyableCodeCard {
                    title: "Neuer Code".to_string(),
                    description: "Direkt weitergeben.".to_string(),
                    code: "ABC-123".to_string(),
                    copy_label: "Kopieren".to_string(),
                    on_copy: move |_| {},
                }
            }
        });

        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains("ABC-123"));
        assert!(html.contains("Kopieren"));
    }

    #[test]
    fn confirm_action_dialog_renders_when_open() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                ConfirmActionDialog {
                    open: true,
                    title: "Eintrag entfernen".to_string(),
                    description: "Diese Aktion kann nicht rückgängig gemacht werden.".to_string(),
                    confirm_label: "Entfernen".to_string(),
                    busy: false,
                    on_open_change: move |_| {},
                    on_confirm: move |_| {},
                }
            }
        });

        dom.rebuild_in_place();
        let _html = dioxus_ssr::render(&dom);
    }
}

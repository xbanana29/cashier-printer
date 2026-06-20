//! Shared UI components.

use dioxus::prelude::*;

/// A monospace textarea with a vertical guide line marking the receipt column
/// width. Controlled component: the parent owns `value` and updates it from
/// `oninput`.
#[component]
pub fn GuidedTextarea(
    #[props(into, default = String::new())] id: String,
    #[props(into)] value: String,
    char_width: i32,
    #[props(into, default = String::new())] placeholder: String,
    #[props(default = 8)] rows: i32,
    #[props(default = false)] disabled: bool,
    oninput: EventHandler<String>,
    onkeydown: EventHandler<Event<KeyboardData>>,
) -> Element {
    rsx! {
        textarea {
            id: "{id}",
            class: "guided-textarea",
            style: "--guide-col: {char_width}",
            placeholder: "{placeholder}",
            rows: "{rows}",
            disabled,
            value: "{value}",
            oninput: move |e| oninput.call(e.value()),
            onkeydown: move |e| onkeydown.call(e),
        }
    }
}

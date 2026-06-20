//! "Pesanan Baru" / "Tanda Terima" — create + print a new order.

use dioxus::prelude::*;

use crate::api;
use crate::components::GuidedTextarea;
use crate::types::char_width_for;
use crate::Toasts;

const RECEIPT_TEMPLATE: &str = "Jenis : \nGudang : ";

fn empty_name_msg(ot: &str) -> &'static str {
    if ot == "receipt" { "Nama penerima wajib diisi" } else { "Nama pelanggan wajib diisi" }
}
fn empty_content_msg(ot: &str) -> &'static str {
    if ot == "receipt" { "Isi tanda terima wajib diisi" } else { "Isi pesanan wajib diisi" }
}
fn success_msg(ot: &str) -> &'static str {
    if ot == "receipt" { "Tanda terima berhasil dicetak" } else { "Pesanan berhasil dicetak" }
}

fn apply_select(
    mut order_type: Signal<String>,
    mut customer_name: Signal<String>,
    mut content: Signal<String>,
    target: &str,
) {
    let current = order_type.peek().clone();
    if current == target {
        return;
    }
    let was_receipt = current == "receipt";
    order_type.set(target.to_string());

    let cur_content = content.peek().clone();
    if target == "receipt" && cur_content.trim().is_empty() {
        content.set(RECEIPT_TEMPLATE.to_string());
    } else if target == "order" && cur_content == RECEIPT_TEMPLATE {
        content.set(String::new());
    }
    if was_receipt != (target == "receipt") {
        customer_name.set(String::new());
    }
}

fn trigger_submit(
    order_type: Signal<String>,
    mut customer_name: Signal<String>,
    mut content: Signal<String>,
    mut is_submitting: Signal<bool>,
    toasts: Toasts,
) {
    let ot = order_type.peek().clone();
    let name = customer_name.peek().trim().to_string();
    let body = content.peek().trim().to_string();
    if name.is_empty() {
        toasts.show_error(empty_name_msg(&ot));
        return;
    }
    if body.is_empty() {
        toasts.show_error(empty_content_msg(&ot));
        return;
    }
    is_submitting.set(true);
    spawn(async move {
        match api::create_order(&name, &body, &ot).await {
            Ok(id) => match api::print_order(id).await {
                Ok(()) => {
                    toasts.show(success_msg(&ot));
                    customer_name.set(String::new());
                    content.set(if ot == "receipt" {
                        RECEIPT_TEMPLATE.to_string()
                    } else {
                        String::new()
                    });
                }
                Err(e) => toasts.error(e),
            },
            Err(e) => toasts.error(e),
        }
        is_submitting.set(false);
    });
}

#[component]
pub fn NewOrder() -> Element {
    let toasts = use_context::<Toasts>();
    let order_type = use_signal(|| "order".to_string());
    let customer_name = use_signal(String::new);
    let content = use_signal(String::new);
    let is_submitting = use_signal(|| false);
    let mut char_width = use_signal(|| 48i32);

    use_future(move || async move {
        if let Ok(s) = api::get_settings().await {
            char_width.set(char_width_for(&s.paper_size));
        }
    });

    let ot = order_type();
    let is_receipt = ot == "receipt";
    let submitting = is_submitting();

    let name_label = if is_receipt { "Diterima dari" } else { "Nama Pelanggan" };
    let name_placeholder = if is_receipt { "Contoh: Toko Maju" } else { "Contoh: Pak Budi" };
    let submit_label = if is_receipt { "Cetak Tanda Terima" } else { "Cetak Pesanan" };
    let content_label = if is_receipt { "Isi Tanda Terima" } else { "Isi Pesanan" };
    let content_placeholder = if is_receipt {
        "Jenis : ...\nGudang : ..."
    } else {
        "Tulis atau paste daftar pesanan di sini..."
    };
    let cw = char_width();

    let order_chip = if !is_receipt { "chip chip-selected" } else { "chip" };
    let receipt_chip = if is_receipt { "chip chip-selected" } else { "chip" };

    rsx! {
        div { class: "page-new",
            h2 { if is_receipt { "Tanda Terima" } else { "Pesanan Baru" } }

            div { class: "type-chips",
                button {
                    r#type: "button",
                    class: "{order_chip}",
                    onclick: move |_| apply_select(order_type, customer_name, content, "order"),
                    span { class: "material-symbols-outlined", "receipt" }
                    "Pesanan"
                }
                button {
                    r#type: "button",
                    class: "{receipt_chip}",
                    onclick: move |_| apply_select(order_type, customer_name, content, "receipt"),
                    span { class: "material-symbols-outlined", "assignment" }
                    "Tanda Terima"
                }
            }

            div { class: "field",
                label { r#for: "customer", class: "field-label", "{name_label}" }
                input {
                    id: "customer",
                    class: "field-input",
                    r#type: "text",
                    placeholder: "{name_placeholder}",
                    autocomplete: "off",
                    disabled: submitting,
                    value: "{customer_name}",
                    oninput: move |e| { let mut c = customer_name; c.set(e.value()); },
                    onkeydown: move |e: Event<KeyboardData>| {
                        if e.key() == Key::Enter {
                            e.prevent_default();
                            trigger_submit(order_type, customer_name, content, is_submitting, toasts);
                        }
                    },
                }
            }

            div { class: "field",
                label { r#for: "content", class: "field-label",
                    "{content_label}"
                    span { class: "label-hint", " — garis biru = batas {cw} kolom" }
                }
                GuidedTextarea {
                    id: "content",
                    value: content(),
                    char_width: cw,
                    placeholder: content_placeholder,
                    rows: 10,
                    disabled: submitting,
                    oninput: move |v: String| { let mut c = content; c.set(v); },
                    onkeydown: move |e: Event<KeyboardData>| {
                        if e.key() == Key::Enter && (e.modifiers().ctrl() || e.modifiers().meta()) {
                            e.prevent_default();
                            trigger_submit(order_type, customer_name, content, is_submitting, toasts);
                        }
                    },
                }
                span { class: "field-support", "Ctrl+Enter untuk cetak langsung" }
            }

            button {
                r#type: "button",
                class: "btn-filled",
                disabled: submitting,
                onclick: move |_| trigger_submit(order_type, customer_name, content, is_submitting, toasts),
                span { class: "material-symbols-outlined", "print" }
                if submitting { "Mencetak..." } else { "{submit_label}" }
            }
        }
    }
}

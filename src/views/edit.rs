//! "Edit Pesanan / Tanda Terima" — edit an existing order, optionally reprint.

use dioxus::prelude::*;

use crate::api;
use crate::components::GuidedTextarea;
use crate::types::char_width_for;
use crate::{Route, Toasts};

fn do_save(
    id: i64,
    customer_name: Signal<String>,
    content: Signal<String>,
    mut saving: Signal<bool>,
    route: Signal<Route>,
    toasts: Toasts,
    reprint: bool,
) {
    let name = customer_name.peek().trim().to_string();
    let body = content.peek().trim().to_string();
    if name.is_empty() {
        toasts.show_error("Nama pelanggan wajib diisi");
        return;
    }
    if body.is_empty() {
        toasts.show_error("Isi pesanan wajib diisi");
        return;
    }
    saving.set(true);
    spawn(async move {
        match api::update_order(id, &name, &body).await {
            Ok(()) => {
                if reprint {
                    match api::reprint_order(id).await {
                        Ok(()) => {
                            toasts.show("Pesanan disimpan dan dicetak ulang");
                            let mut r = route;
                            r.set(Route::History);
                        }
                        Err(e) => toasts.error(e),
                    }
                } else {
                    toasts.show("Pesanan berhasil disimpan");
                    let mut r = route;
                    r.set(Route::History);
                }
            }
            Err(e) => toasts.error(e),
        }
        saving.set(false);
    });
}

#[component]
pub fn Edit(id: i64) -> Element {
    let toasts = use_context::<Toasts>();
    let route = use_context::<Signal<Route>>();

    let mut customer_name = use_signal(String::new);
    let mut content = use_signal(String::new);
    let mut created_at = use_signal(String::new);
    let mut order_type = use_signal(|| "order".to_string());
    let mut loading = use_signal(|| true);
    let saving = use_signal(|| false);
    let mut char_width = use_signal(|| 48i32);

    use_future(move || async move {
        let order = api::get_order(id).await;
        let settings = api::get_settings().await;
        match order {
            Ok(o) => {
                customer_name.set(o.customer_name);
                content.set(o.content);
                created_at.set(o.created_at);
                order_type.set(o.order_type);
                if let Ok(s) = settings {
                    char_width.set(char_width_for(&s.paper_size));
                }
                loading.set(false);
            }
            Err(e) => {
                toasts.error(e);
                let mut r = route;
                r.set(Route::History);
            }
        }
    });

    let is_loading = loading();
    let is_saving = saving();
    let is_receipt = order_type() == "receipt";
    let title = if is_receipt { "Edit Tanda Terima" } else { "Edit Pesanan" };
    let name_label = if is_receipt { "Diterima dari" } else { "Nama Pelanggan" };
    let content_label = if is_receipt { "Isi Tanda Terima" } else { "Isi Pesanan" };
    let cw = char_width();

    rsx! {
        div { class: "page-edit",
            div { class: "page-header",
                h2 { "{title}" }
                button {
                    class: "btn-text",
                    onclick: move |_| { let mut r = route; r.set(Route::History); },
                    span { class: "material-symbols-outlined", "arrow_back" }
                    "Kembali"
                }
            }

            if is_loading {
                p { class: "state-msg", "Memuat..." }
            } else {
                div { class: "field",
                    label { r#for: "created", class: "field-label", "Tanggal Dibuat" }
                    input { id: "created", class: "field-input", r#type: "text", value: "{created_at}", disabled: true }
                }

                div { class: "field",
                    label { r#for: "customer", class: "field-label", "{name_label}" }
                    input {
                        id: "customer",
                        class: "field-input",
                        r#type: "text",
                        autocomplete: "off",
                        disabled: is_saving,
                        value: "{customer_name}",
                        oninput: move |e| customer_name.set(e.value()),
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
                        rows: 10,
                        disabled: is_saving,
                        oninput: move |v: String| { let mut c = content; c.set(v); },
                        onkeydown: move |_| {},
                    }
                }

                div { class: "actions",
                    button {
                        r#type: "button",
                        class: "btn-filled",
                        disabled: is_saving,
                        onclick: move |_| do_save(id, customer_name, content, saving, route, toasts, false),
                        span { class: "material-symbols-outlined", "save" }
                        if is_saving { "Menyimpan..." } else { "Simpan" }
                    }
                    button {
                        r#type: "button",
                        class: "btn-outlined",
                        disabled: is_saving,
                        onclick: move |_| do_save(id, customer_name, content, saving, route, toasts, true),
                        span { class: "material-symbols-outlined", "print" }
                        "Simpan & Cetak Ulang"
                    }
                }
            }
        }
    }
}

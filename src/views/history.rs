//! "Riwayat" — list, search, paginate, preview, reprint, edit, delete orders.

use dioxus::prelude::*;

use crate::api;
use crate::types::{char_width_for, Order};
use crate::{Route, Toasts};

const PAGE_SIZE: i32 = 25;

fn content_preview(content: &str) -> String {
    let first = content.split('\n').next().unwrap_or("");
    if first.chars().count() > 60 {
        let truncated: String = first.chars().take(60).collect();
        format!("{truncated}…")
    } else {
        first.to_string()
    }
}

/// Build escaped receipt HTML (first line = large customer name), matching the
/// old Svelte preview exactly. Safe to inject: every line is HTML-escaped here.
fn build_preview_html(text: &str) -> String {
    let mut out = String::new();
    for (i, line) in text.split('\n').enumerate() {
        let esc = line
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        let safe = if esc.is_empty() { "&nbsp;".to_string() } else { esc };
        if i == 0 {
            out.push_str(&format!("<div class=\"line preview-cname\">{safe}</div>"));
        } else {
            out.push_str(&format!("<div class=\"line\">{safe}</div>"));
        }
    }
    out
}

async fn load_orders(
    active_tab: Signal<String>,
    mut orders: Signal<Vec<Order>>,
    mut loading: Signal<bool>,
    mut search: Signal<String>,
    mut page: Signal<i32>,
    toasts: Toasts,
) {
    loading.set(true);
    search.set(String::new());
    page.set(1);
    let _ = api::purge_old_orders().await;
    let tab = active_tab.peek().clone();
    match api::get_orders(&tab).await {
        Ok(o) => orders.set(o),
        Err(e) => toasts.error(e),
    }
    loading.set(false);
}

fn switch_tab(
    mut active_tab: Signal<String>,
    orders: Signal<Vec<Order>>,
    loading: Signal<bool>,
    search: Signal<String>,
    page: Signal<i32>,
    toasts: Toasts,
    tab: &str,
) {
    if active_tab.peek().as_str() == tab {
        return;
    }
    active_tab.set(tab.to_string());
    spawn(load_orders(active_tab, orders, loading, search, page, toasts));
}

fn reprint(id: i64, type_label: String, toasts: Toasts) {
    spawn(async move {
        match api::reprint_order(id).await {
            Ok(()) => toasts.show(format!("{type_label} berhasil dicetak ulang")),
            Err(e) => toasts.error(e),
        }
    });
}

fn delete_order(
    id: i64,
    mut orders: Signal<Vec<Order>>,
    mut deleting_id: Signal<Option<i64>>,
    mut confirm_delete_id: Signal<Option<i64>>,
    type_label: String,
    toasts: Toasts,
) {
    deleting_id.set(Some(id));
    confirm_delete_id.set(None);
    spawn(async move {
        match api::delete_order(id).await {
            Ok(()) => {
                let kept: Vec<Order> =
                    orders.peek().iter().filter(|o| o.id != id).cloned().collect();
                orders.set(kept);
                toasts.show(format!("{type_label} dihapus"));
            }
            Err(e) => toasts.error(e),
        }
        deleting_id.set(None);
    });
}

fn open_preview(
    id: i64,
    mut preview_order_id: Signal<Option<i64>>,
    mut preview_text: Signal<String>,
    mut preview_loading: Signal<bool>,
    toasts: Toasts,
) {
    preview_order_id.set(Some(id));
    preview_text.set(String::new());
    preview_loading.set(true);
    spawn(async move {
        match api::preview_receipt(id).await {
            Ok(t) => preview_text.set(t),
            Err(e) => {
                toasts.error(e);
                preview_order_id.set(None);
            }
        }
        preview_loading.set(false);
    });
}

#[component]
pub fn History() -> Element {
    let toasts = use_context::<Toasts>();
    let route = use_context::<Signal<Route>>();

    let active_tab = use_signal(|| "order".to_string());
    let orders = use_signal(Vec::<Order>::new);
    let search = use_signal(String::new);
    let loading = use_signal(|| true);
    let deleting_id = use_signal(|| None::<i64>);
    let confirm_delete_id = use_signal(|| None::<i64>);
    let page = use_signal(|| 1i32);
    let mut paper_width = use_signal(|| 48i32);

    let preview_order_id = use_signal(|| None::<i64>);
    let preview_text = use_signal(String::new);
    let preview_loading = use_signal(|| false);

    use_future(move || async move {
        load_orders(active_tab, orders, loading, search, page, toasts).await;
        if let Ok(s) = api::get_settings().await {
            paper_width.set(char_width_for(&s.paper_size));
        }
    });

    let tab = active_tab();
    let type_label = if tab == "receipt" { "Tanda Terima" } else { "Pesanan" }.to_string();
    let type_label_lc = type_label.to_lowercase();
    let is_loading = loading();

    let search_val = search();
    let q = search_val.trim().to_lowercase();
    let all = orders();
    let filtered: Vec<Order> = if q.is_empty() {
        all.clone()
    } else {
        all.iter()
            .filter(|o| o.customer_name.to_lowercase().contains(&q))
            .cloned()
            .collect()
    };
    let total = filtered.len();
    let total_pages = (((total as f64) / (PAGE_SIZE as f64)).ceil() as i32).max(1);
    let cur_page = page().clamp(1, total_pages);
    let start = ((cur_page - 1) * PAGE_SIZE) as usize;
    let paginated: Vec<Order> = filtered
        .iter()
        .skip(start)
        .take(PAGE_SIZE as usize)
        .cloned()
        .collect();
    let page_from = if total == 0 { 0 } else { start + 1 };
    let page_to = (start + PAGE_SIZE as usize).min(total);

    let preview_id = preview_order_id();
    let order_tab = active_tab().clone();
    let order_tab_active = if order_tab == "order" { "tab tab-active" } else { "tab" };
    let receipt_tab_active = if order_tab == "receipt" { "tab tab-active" } else { "tab" };

    rsx! {
        div { class: "page-history",
            div { class: "page-header",
                h2 { "Riwayat" }
                button {
                    class: "btn-icon-sm",
                    title: "Muat ulang",
                    onclick: move |_| { spawn(load_orders(active_tab, orders, loading, search, page, toasts)); },
                    span { class: "material-symbols-outlined", "refresh" }
                }
            }

            div { class: "tabs",
                button {
                    class: "{order_tab_active}",
                    onclick: move |_| switch_tab(active_tab, orders, loading, search, page, toasts, "order"),
                    span { class: "material-symbols-outlined", "receipt" }
                    "Pesanan"
                }
                button {
                    class: "{receipt_tab_active}",
                    onclick: move |_| switch_tab(active_tab, orders, loading, search, page, toasts, "receipt"),
                    span { class: "material-symbols-outlined", "assignment" }
                    "Tanda Terima"
                }
            }

            div { class: "search-bar",
                span { class: "material-symbols-outlined search-icon", "search" }
                input {
                    class: "search-input",
                    r#type: "search",
                    placeholder: "Cari nama...",
                    value: "{search}",
                    oninput: move |e| { let mut s = search; s.set(e.value()); let mut p = page; p.set(1); },
                }
                if !search_val.is_empty() {
                    span { class: "search-count", "{total} hasil" }
                }
            }

            if is_loading {
                p { class: "state-msg", "Memuat..." }
            } else if all.is_empty() {
                p { class: "state-msg", "Belum ada {type_label_lc}." }
            } else if filtered.is_empty() {
                p { class: "state-msg",
                    "Tidak ada {type_label_lc} untuk \""
                    strong { "{search_val}" }
                    "\"."
                }
            } else {
                div { class: "list",
                    for order in paginated.iter() {
                        {
                            let oid = order.id;
                            let confirming = confirm_delete_id() == Some(oid);
                            let is_deleting = deleting_id() == Some(oid);
                            let tl = type_label.clone();
                            let tl2 = type_label.clone();
                            let preview = content_preview(&order.content);
                            rsx! {
                                div { key: "{oid}", class: "card",
                                    div { class: "card-body",
                                        div { class: "card-name", "{order.customer_name}" }
                                        div { class: "card-preview", "{preview}" }
                                        div { class: "card-meta",
                                            span { class: "card-date", "{order.created_at}" }
                                            if !order.pc_name.is_empty() {
                                                span { class: "card-pc", "{order.pc_name}" }
                                            }
                                        }
                                    }
                                    div { class: "card-actions",
                                        if confirming {
                                            span { class: "confirm-label", "Hapus?" }
                                            button {
                                                class: "icon-btn icon-btn-danger",
                                                title: "Ya, hapus",
                                                disabled: is_deleting,
                                                onclick: move |_| delete_order(oid, orders, deleting_id, confirm_delete_id, tl.clone(), toasts),
                                                span { class: "material-symbols-outlined", "check" }
                                            }
                                            button {
                                                class: "icon-btn",
                                                title: "Batal",
                                                onclick: move |_| { let mut c = confirm_delete_id; c.set(None); },
                                                span { class: "material-symbols-outlined", "close" }
                                            }
                                        } else {
                                            button {
                                                class: "icon-btn",
                                                title: "Preview struk",
                                                onclick: move |_| open_preview(oid, preview_order_id, preview_text, preview_loading, toasts),
                                                span { class: "material-symbols-outlined", "receipt_long" }
                                            }
                                            button {
                                                class: "icon-btn",
                                                title: "Edit",
                                                onclick: move |_| { let mut r = route; r.set(Route::Edit(oid)); },
                                                span { class: "material-symbols-outlined", "edit" }
                                            }
                                            button {
                                                class: "icon-btn",
                                                title: "Cetak ulang",
                                                onclick: move |_| reprint(oid, tl2.clone(), toasts),
                                                span { class: "material-symbols-outlined", "print" }
                                            }
                                            button {
                                                class: "icon-btn icon-btn-danger",
                                                title: "Hapus",
                                                onclick: move |_| { let mut c = confirm_delete_id; c.set(Some(oid)); },
                                                span { class: "material-symbols-outlined", "delete" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if total_pages > 1 {
                    div { class: "pagination",
                        button {
                            class: "pg-btn",
                            disabled: cur_page == 1,
                            onclick: move |_| { let mut p = page; p.set((cur_page - 1).max(1)); },
                            span { class: "material-symbols-outlined", "chevron_left" }
                        }
                        for p in 1..=total_pages {
                            button {
                                key: "{p}",
                                class: if p == cur_page { "pg-btn pg-active" } else { "pg-btn" },
                                onclick: move |_| { let mut pg = page; pg.set(p); },
                                "{p}"
                            }
                        }
                        button {
                            class: "pg-btn",
                            disabled: cur_page == total_pages,
                            onclick: move |_| { let mut p = page; p.set((cur_page + 1).min(total_pages)); },
                            span { class: "material-symbols-outlined", "chevron_right" }
                        }
                        span { class: "pg-info",
                            "{page_from}–{page_to} / {total}"
                        }
                    }
                }
            }

            // ── Receipt preview modal (kept inside .page-history for CSS scope) ──
            if let Some(pid) = preview_id {
                div {
                    class: "overlay",
                    onclick: move |_| {
                        let mut p = preview_order_id; p.set(None);
                        let mut t = preview_text; t.set(String::new());
                    },
                    div {
                        class: "modal",
                        onclick: move |e| e.stop_propagation(),
                        div { class: "modal-header",
                            span { class: "modal-title", "Preview Struk" }
                            div { class: "modal-header-actions",
                                button {
                                    class: "btn-filled-sm",
                                    onclick: move |_| {
                                        reprint(pid, type_label.clone(), toasts);
                                        let mut p = preview_order_id; p.set(None);
                                        let mut t = preview_text; t.set(String::new());
                                    },
                                    span { class: "material-symbols-outlined", "print" }
                                    "Cetak Ulang"
                                }
                                button {
                                    class: "modal-close",
                                    onclick: move |_| {
                                        let mut p = preview_order_id; p.set(None);
                                        let mut t = preview_text; t.set(String::new());
                                    },
                                    span { class: "material-symbols-outlined", "close" }
                                }
                            }
                        }
                        div { class: "receipt-wrapper",
                            if preview_loading() {
                                p { class: "receipt-loading", "Memuat preview..." }
                            } else {
                                div { class: "paper", style: "--cols: {paper_width}",
                                    div {
                                        class: "receipt-text",
                                        dangerous_inner_html: build_preview_html(&preview_text()),
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

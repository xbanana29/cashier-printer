//! Print Paste Order — Dioxus (WASM) frontend, served inside the Tauri shell.

mod api;
mod components;
mod types;
mod views;

use dioxus::prelude::*;

use views::{Edit, History, NewOrder, Settings};

/// Which screen is currently shown. Replaces SvelteKit file-based routing with
/// a simple in-memory router (the app is a fixed 4-view SPA).
#[derive(Clone, PartialEq)]
pub enum Route {
    New,
    History,
    Edit(i64),
    Settings,
}

#[derive(Clone, PartialEq)]
pub struct ToastMsg {
    pub text: String,
    pub error: bool,
}

/// Global toast handle, shared via context. Auto-dismisses after 3s; a
/// generation counter ensures only the latest message clears the banner.
#[derive(Clone, Copy)]
pub struct Toasts {
    msg: Signal<Option<ToastMsg>>,
    generation: Signal<u64>,
}

impl Toasts {
    pub fn show(self, text: impl Into<String>) {
        self.push(text.into(), false);
    }

    pub fn show_error(self, text: impl Into<String>) {
        self.push(text.into(), true);
    }

    pub fn error(self, err: api::AppError) {
        self.push(err.display(), true);
    }

    fn push(mut self, text: String, error: bool) {
        let my_gen = self.generation.peek().wrapping_add(1);
        self.generation.set(my_gen);
        self.msg.set(Some(ToastMsg { text, error }));

        let mut msg = self.msg;
        let generation = self.generation;
        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(3000).await;
            if *generation.peek() == my_gen {
                msg.set(None);
            }
        });
    }
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let route = use_signal(|| Route::New);
    use_context_provider(|| route);

    let msg = use_signal(|| None::<ToastMsg>);
    let generation = use_signal(|| 0u64);
    let toasts = Toasts { msg, generation };
    use_context_provider(|| toasts);

    // Retention cleanup on startup (orders older than 1 year), fire-and-forget.
    use_future(move || async move {
        let _ = api::purge_old_orders().await;
    });

    let current = route();
    let new_active = current == Route::New;
    let history_active = matches!(current, Route::History | Route::Edit(_));
    let settings_active = current == Route::Settings;

    rsx! {
        div { class: "app",
            // ── MD3 Navigation Rail ──
            nav { class: "nav-rail",
                div { class: "rail-brand",
                    span { class: "brand-logo", "PPO" }
                    span { class: "brand-name", "Print Paste"
                        br {}
                        "Order"
                    }
                }
                div { class: "rail-items",
                    NavItem { icon: "receipt", label: "Baru", active: new_active,
                        onclick: move |_| { let mut r = route; r.set(Route::New); } }
                    NavItem { icon: "history", label: "Riwayat", active: history_active,
                        onclick: move |_| { let mut r = route; r.set(Route::History); } }
                    NavItem { icon: "settings", label: "Setelan", active: settings_active,
                        onclick: move |_| { let mut r = route; r.set(Route::Settings); } }
                }
            }

            // ── Content + footer ──
            div { class: "main-area",
                main { class: "content",
                    {match current {
                        Route::New => rsx! { NewOrder {} },
                        Route::History => rsx! { History {} },
                        Route::Edit(id) => rsx! { Edit { id } },
                        Route::Settings => rsx! { Settings {} },
                    }}
                }
                footer { class: "watermark",
                    button {
                        class: "watermark-link",
                        onclick: move |_| {
                            spawn(async move {
                                let _ = api::open_external("https://rejekiamerta.com").await;
                            });
                        },
                        "CV REJEKI AMERTA JAYA \u{00a0}·\u{00a0} © 2026"
                    }
                }
            }
        }

        if let Some(t) = msg() {
            div {
                class: if t.error { "toast toast-error" } else { "toast toast-success" },
                role: "alert",
                "{t.text}"
            }
        }
    }
}

#[component]
fn NavItem(icon: &'static str, label: &'static str, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let class = if active { "rail-item active" } else { "rail-item" };
    let variation = if active {
        "'FILL' 1,'wght' 500,'GRAD' 0,'opsz' 24"
    } else {
        "'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24"
    };
    rsx! {
        button { class: "{class}", onclick: move |e| onclick.call(e),
            div { class: "rail-indicator",
                span {
                    class: "material-symbols-outlined",
                    style: "font-variation-settings: {variation}",
                    "{icon}"
                }
            }
            span { class: "rail-label", "{label}" }
        }
    }
}

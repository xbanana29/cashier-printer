//! "Pengaturan" — printer connection, paper, print toggles, template,
//! app update, and LAN sync.

use dioxus::prelude::*;

use crate::api;
use crate::types::{AppSettings, PeerInfo, PrinterInfo};
use crate::Toasts;

#[derive(Clone, Copy, PartialEq)]
enum UpdateStatus {
    Idle,
    Checking,
    UpToDate,
    Available,
    Installing,
}

/// Derive the connection-type tab from a stored printer value.
fn detect_conn_type(val: &str) -> &'static str {
    if val.starts_with("/dev/tty") || val.starts_with("/dev/cu.") {
        return "serial";
    }
    let upper = val.to_ascii_uppercase();
    if upper.starts_with("COM") {
        if let Some(c) = upper.chars().nth(3) {
            if c.is_ascii_digit() {
                return "serial";
            }
        }
    }
    if is_network_addr(val) {
        return "network";
    }
    "os"
}

fn is_network_addr(val: &str) -> bool {
    let host = val.split(':').next().unwrap_or("");
    let parts: Vec<&str> = host.split('.').collect();
    parts.len() == 4
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.len() <= 3 && p.chars().all(|c| c.is_ascii_digit()))
}

fn format_last_seen(ts: u64) -> String {
    if ts == 0 {
        return "—".to_string();
    }
    let now = (js_sys::Date::now() / 1000.0) as u64;
    let diff = now.saturating_sub(ts);
    if diff < 60 {
        format!("{diff}d lalu")
    } else if diff < 3600 {
        format!("{}m lalu", diff / 60)
    } else {
        format!("{}j lalu", diff / 3600)
    }
}

fn load_printers(
    mut printers: Signal<Vec<PrinterInfo>>,
    mut serial_ports: Signal<Vec<String>>,
    mut loading_printers: Signal<bool>,
    toasts: Toasts,
) {
    loading_printers.set(true);
    spawn(async move {
        let pr = api::list_printers().await;
        let sp = api::list_serial_ports().await;
        match (pr, sp) {
            (Ok(p), Ok(s)) => {
                printers.set(p);
                serial_ports.set(s);
            }
            (Err(e), _) | (_, Err(e)) => toasts.error(e),
        }
        loading_printers.set(false);
    });
}

#[component]
pub fn Settings() -> Element {
    let toasts = use_context::<Toasts>();

    let settings = use_signal(AppSettings::default);
    let printers = use_signal(Vec::<PrinterInfo>::new);
    let serial_ports = use_signal(Vec::<String>::new);
    let loading_printers = use_signal(|| false);
    let saving = use_signal(|| false);
    let testing = use_signal(|| false);
    let conn_type = use_signal(|| "os".to_string());

    let peers = use_signal(Vec::<PeerInfo>::new);
    let syncing = use_signal(|| false);
    let current_version = use_signal(|| "—".to_string());
    let update_status = use_signal(|| UpdateStatus::Idle);
    // Pending update info: (version, release notes).
    let update_data = use_signal(|| None::<(String, String)>);

    // Initial load: version, settings, printers, peers.
    use_future(move || async move {
        let mut cv = current_version;
        cv.set(api::get_app_version().await.unwrap_or_else(|_| "—".to_string()));

        if let Ok(s) = api::get_settings().await {
            let mut ct = conn_type;
            ct.set(detect_conn_type(&s.default_printer).to_string());
            let mut st = settings;
            st.set(s);
        }
        load_printers(printers, serial_ports, loading_printers, toasts);
        if let Ok(p) = api::get_peers().await {
            let mut pr = peers;
            pr.set(p);
        }
    });

    // Poll peers every 5s while the settings page is open.
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(5000).await;
            if let Ok(p) = api::get_peers().await {
                let mut pr = peers;
                pr.set(p);
            }
        }
    });

    let s = settings.read();
    let ct = conn_type();
    let is_loading_printers = loading_printers();
    let is_saving = saving();
    let is_testing = testing();
    let is_syncing = syncing();
    let us = update_status();
    let ui = update_data();
    let peer_list = peers();
    let version = current_version();

    let conn_os = if ct == "os" { "conn-tab conn-tab-active" } else { "conn-tab" };
    let conn_serial = if ct == "serial" { "conn-tab conn-tab-active" } else { "conn-tab" };
    let conn_network = if ct == "network" { "conn-tab conn-tab-active" } else { "conn-tab" };

    rsx! {
        div { class: "page-settings",
            h2 { "Pengaturan" }

            // ── Printer section ──
            div { class: "section",
                div { class: "section-header",
                    span { class: "material-symbols-outlined section-icon", "print" }
                    span { class: "section-title", "Printer" }
                }

                div { class: "field",
                    span { class: "field-label", "Jenis Koneksi" }
                    div { class: "conn-tabs",
                        button {
                            r#type: "button",
                            class: "{conn_os}",
                            onclick: move |_| { let mut c = conn_type; c.set("os".to_string()); let mut st = settings; st.write().default_printer = String::new(); },
                            span { class: "material-symbols-outlined", "usb" }
                            "USB / CUPS"
                        }
                        button {
                            r#type: "button",
                            class: "{conn_serial}",
                            onclick: move |_| { let mut c = conn_type; c.set("serial".to_string()); let mut st = settings; st.write().default_printer = String::new(); },
                            span { class: "material-symbols-outlined", "cable" }
                            "Serial / COM"
                        }
                        button {
                            r#type: "button",
                            class: "{conn_network}",
                            onclick: move |_| { let mut c = conn_type; c.set("network".to_string()); let mut st = settings; st.write().default_printer = String::new(); },
                            span { class: "material-symbols-outlined", "wifi" }
                            "Jaringan (LAN)"
                        }
                    }
                }

                if ct == "os" {
                    div { class: "field",
                        label { r#for: "printer", class: "field-label", "Pilih Printer" }
                        div { class: "row-gap",
                            select {
                                id: "printer",
                                class: "field-select",
                                disabled: is_loading_printers,
                                value: "{s.default_printer}",
                                oninput: move |e| { let mut st = settings; st.write().default_printer = e.value(); },
                                option { value: "", "— Pilih printer —" }
                                for p in printers().iter() {
                                    option { key: "{p.name}", value: "{p.name}",
                                        "{p.name}"
                                        if p.is_default { " ✓" }
                                    }
                                }
                            }
                            button {
                                r#type: "button",
                                class: "icon-btn",
                                title: "Refresh",
                                disabled: is_loading_printers,
                                onclick: move |_| load_printers(printers, serial_ports, loading_printers, toasts),
                                span { class: "material-symbols-outlined",
                                    if is_loading_printers { "hourglass_empty" } else { "refresh" }
                                }
                            }
                        }
                        p { class: "field-support", "Printer yang terdeteksi di sistem (USB, Bluetooth, jaringan via CUPS)." }
                    }
                } else if ct == "serial" {
                    div { class: "field",
                        label { r#for: "serial-port", class: "field-label", "Port Serial" }
                        div { class: "row-gap",
                            select {
                                id: "serial-port",
                                class: "field-select",
                                disabled: is_loading_printers,
                                value: "{s.default_printer}",
                                oninput: move |e| { let mut st = settings; st.write().default_printer = e.value(); },
                                option { value: "", "— Pilih port —" }
                                for port in serial_ports().iter() {
                                    option { key: "{port}", value: "{port}", "{port}" }
                                }
                            }
                            button {
                                r#type: "button",
                                class: "icon-btn",
                                title: "Refresh",
                                disabled: is_loading_printers,
                                onclick: move |_| load_printers(printers, serial_ports, loading_printers, toasts),
                                span { class: "material-symbols-outlined",
                                    if is_loading_printers { "hourglass_empty" } else { "refresh" }
                                }
                            }
                        }
                        p { class: "field-support", "Hubungkan printer via kabel USB serial / RS-232. Pastikan kabel terpasang sebelum refresh." }
                    }
                    div { class: "field",
                        label { r#for: "baud", class: "field-label", "Kecepatan (Baud Rate)" }
                        select {
                            id: "baud",
                            class: "field-select field-select-short",
                            value: "{s.serial_baud_rate}",
                            oninput: move |e| { if let Ok(v) = e.value().parse::<u32>() { let mut st = settings; st.write().serial_baud_rate = v; } },
                            option { value: "9600", "9600 — RPP02" }
                            option { value: "19200", "19200 — EPSON TM-U220" }
                            option { value: "38400", "38400" }
                            option { value: "115200", "115200" }
                        }
                    }
                } else {
                    div { class: "field",
                        label { r#for: "net-addr", class: "field-label", "Alamat IP Printer" }
                        input {
                            id: "net-addr",
                            class: "field-input",
                            r#type: "text",
                            placeholder: "Contoh: 192.168.1.100:9100",
                            value: "{s.default_printer}",
                            oninput: move |e| { let mut st = settings; st.write().default_printer = e.value(); },
                        }
                        p { class: "field-support", "Format: IP:port. Port default printer jaringan biasanya 9100." }
                    }
                }

                // Paper size
                div { class: "field",
                    span { class: "field-label", "Ukuran Kertas" }
                    div { class: "chip-group",
                        for size in ["58mm", "75mm", "80mm"] {
                            button {
                                key: "{size}",
                                r#type: "button",
                                class: if s.paper_size == size { "chip chip-selected" } else { "chip" },
                                onclick: move |_| { let mut st = settings; st.write().paper_size = size.to_string(); },
                                "{size}"
                            }
                        }
                    }
                }

                // Toggles
                Toggle {
                    on: s.auto_cut,
                    name: "Potong kertas otomatis",
                    desc: "Matikan untuk TM-U220 tanpa cutter.",
                    ontoggle: move |_| { let mut st = settings; let v = !st.peek().auto_cut; st.write().auto_cut = v; },
                }
                Toggle {
                    on: s.bold_items,
                    name: "Tebalkan tulisan item",
                    desc: "Cetak item pesanan dengan huruf tebal (bold).",
                    ontoggle: move |_| { let mut st = settings; let v = !st.peek().bold_items; st.write().bold_items = v; },
                }
                Toggle {
                    on: s.large_font,
                    name: "Perbesar ukuran font item",
                    desc: "Cetak item 2× lebih besar. Kolom per baris jadi setengahnya.",
                    ontoggle: move |_| { let mut st = settings; let v = !st.peek().large_font; st.write().large_font = v; },
                }

                // Extra feeds
                div { class: "field",
                    label { r#for: "extra-feeds", class: "field-label", "Baris Kosong Setelah Cetak" }
                    select {
                        id: "extra-feeds",
                        class: "field-select field-select-short",
                        value: "{s.extra_feeds}",
                        oninput: move |e| { if let Ok(v) = e.value().parse::<u8>() { let mut st = settings; st.write().extra_feeds = v; } },
                        for n in 0..=5u8 {
                            option { key: "{n}", value: "{n}", "{n}" }
                        }
                    }
                    p { class: "field-support", "Tambah jika tulisan terakhir tidak keluar dari kepala cetak." }
                }
            }

            // ── Template section ──
            div { class: "section",
                div { class: "section-header",
                    span { class: "material-symbols-outlined section-icon", "receipt_long" }
                    span { class: "section-title", "Template Cetak" }
                }
                div { class: "field",
                    label { r#for: "store", class: "field-label", "Nama Toko " span { class: "label-optional", "(opsional)" } }
                    input {
                        id: "store",
                        class: "field-input",
                        r#type: "text",
                        placeholder: "Contoh: Toko Makmur Jaya",
                        value: "{s.store_name}",
                        oninput: move |e| { let mut st = settings; st.write().store_name = e.value(); },
                    }
                }
                div { class: "field",
                    label { r#for: "footer", class: "field-label", "Footer " span { class: "label-optional", "(opsional)" } }
                    input {
                        id: "footer",
                        class: "field-input",
                        r#type: "text",
                        placeholder: "Contoh: Terima kasih atas pesanan Anda!",
                        value: "{s.footer_text}",
                        oninput: move |e| { let mut st = settings; st.write().footer_text = e.value(); },
                    }
                }
                div { class: "field",
                    label { r#for: "pcname", class: "field-label", "Nama PC / Kasir" }
                    input {
                        id: "pcname",
                        class: "field-input",
                        r#type: "text",
                        placeholder: "Contoh: Kasir 1",
                        value: "{s.pc_name}",
                        oninput: move |e| { let mut st = settings; st.write().pc_name = e.value(); },
                    }
                    p { class: "field-support", "Muncul di baris bawah struk dan di kolom riwayat pesanan." }
                }
            }

            // ── About & Update ──
            div { class: "section",
                div { class: "section-header",
                    span { class: "material-symbols-outlined section-icon", "system_update" }
                    span { class: "section-title", "Tentang & Update" }
                }
                div { class: "about-row",
                    span { class: "about-label", "Versi saat ini" }
                    span { class: "about-value", "v{version}" }
                }

                if us == UpdateStatus::Available {
                    if let Some((ver, body)) = ui.clone() {
                        div { class: "update-banner",
                            span { class: "material-symbols-outlined update-icon", "new_releases" }
                            div { class: "update-info",
                                span { class: "update-title", "Update tersedia: v{ver}" }
                                if !body.is_empty() {
                                    span { class: "update-notes", "{body}" }
                                }
                            }
                        }
                        button {
                            r#type: "button",
                            class: "btn-update",
                            onclick: move |_| {
                                let mut st = update_status; st.set(UpdateStatus::Installing);
                                spawn(async move {
                                    if let Err(e) = api::install_update().await {
                                        toasts.error(e);
                                        let mut st2 = update_status; st2.set(UpdateStatus::Available);
                                    }
                                });
                            },
                            span { class: "material-symbols-outlined", "download" }
                            "Download & Pasang v{ver}"
                        }
                    }
                } else if us == UpdateStatus::Installing {
                    p { class: "update-msg update-ok",
                        span { class: "material-symbols-outlined", "hourglass_empty" }
                        "Mengunduh & memasang pembaruan..."
                    }
                } else if us == UpdateStatus::UpToDate {
                    p { class: "update-msg update-ok",
                        span { class: "material-symbols-outlined", "check_circle" }
                        "Aplikasi sudah versi terbaru"
                    }
                }

                div { class: "field", style: "margin-top: 8px; margin-bottom: 0",
                    button {
                        r#type: "button",
                        class: "btn-outlined",
                        disabled: us == UpdateStatus::Checking || us == UpdateStatus::Installing,
                        onclick: move |_| {
                            let mut st = update_status; st.set(UpdateStatus::Checking);
                            let mut info = update_data; info.set(None);
                            spawn(async move {
                                match api::check_for_update().await {
                                    Ok(Some(u)) => { info.set(Some((u.version, u.body))); st.set(UpdateStatus::Available); }
                                    _ => st.set(UpdateStatus::UpToDate),
                                }
                            });
                        },
                        span { class: "material-symbols-outlined",
                            if us == UpdateStatus::Checking { "hourglass_empty" } else { "sync" }
                        }
                        if us == UpdateStatus::Checking { "Mengecek..." } else { "Cek Update" }
                    }
                }
            }

            // ── LAN Sync ──
            div { class: "section",
                div { class: "section-header",
                    span { class: "material-symbols-outlined section-icon", "lan" }
                    span { class: "section-title", "Sinkronisasi LAN" }
                }
                p { class: "field-support", style: "margin-bottom: 12px",
                    "Sinkron otomatis antar PC dalam satu jaringan WiFi/LAN. Tidak perlu internet."
                }
                div { class: "field",
                    span { class: "field-label", "Perangkat Terhubung" }
                    if peer_list.is_empty() {
                        p { class: "peer-empty", "Belum ada perangkat lain terdeteksi di jaringan ini." }
                    } else {
                        div { class: "peer-list",
                            for peer in peer_list.iter() {
                                {
                                    let last = format_last_seen(peer.last_seen);
                                    let name = if peer.pc_name.is_empty() { "Tanpa nama".to_string() } else { peer.pc_name.clone() };
                                    rsx! {
                                        div { key: "{peer.device_id}", class: "peer-row",
                                            span { class: "material-symbols-outlined peer-icon", "computer" }
                                            div { class: "peer-info",
                                                span { class: "peer-name", "{name}" }
                                                span { class: "peer-addr", "{peer.addr}" }
                                            }
                                            div { class: "peer-meta",
                                                if peer.orders_synced > 0 {
                                                    span { class: "peer-badge", "{peer.orders_synced} order" }
                                                }
                                                span { class: "peer-seen", "{last}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "field", style: "margin-bottom: 0",
                    button {
                        r#type: "button",
                        class: "btn-outlined",
                        disabled: is_syncing,
                        onclick: move |_| {
                            let mut sy = syncing; sy.set(true);
                            spawn(async move {
                                match api::sync_now().await {
                                    Ok(count) => {
                                        toasts.show(if count > 0 { format!("{count} pesanan berhasil disinkron") } else { "Tidak ada pesanan baru".to_string() });
                                        if let Ok(p) = api::get_peers().await { let mut pr = peers; pr.set(p); }
                                    }
                                    Err(e) => toasts.error(e),
                                }
                                let mut sy2 = syncing; sy2.set(false);
                            });
                        },
                        span { class: "material-symbols-outlined",
                            if is_syncing { "hourglass_empty" } else { "sync" }
                        }
                        if is_syncing { "Menyinkron..." } else { "Sync Sekarang" }
                    }
                }
            }

            // ── Actions ──
            div { class: "actions",
                button {
                    r#type: "button",
                    class: "btn-filled",
                    disabled: is_saving,
                    onclick: move |_| {
                        let mut sv = saving; sv.set(true);
                        let snapshot = settings.peek().clone();
                        spawn(async move {
                            match api::save_settings(&snapshot).await {
                                Ok(()) => toasts.show("Pengaturan disimpan"),
                                Err(e) => toasts.error(e),
                            }
                            let mut sv2 = saving; sv2.set(false);
                        });
                    },
                    span { class: "material-symbols-outlined", "save" }
                    if is_saving { "Menyimpan..." } else { "Simpan Pengaturan" }
                }
                button {
                    r#type: "button",
                    class: "btn-outlined",
                    disabled: is_testing,
                    onclick: move |_| {
                        let mut t = testing; t.set(true);
                        spawn(async move {
                            match api::test_print().await {
                                Ok(()) => toasts.show("Test print berhasil"),
                                Err(e) => toasts.error(e),
                            }
                            let mut t2 = testing; t2.set(false);
                        });
                    },
                    span { class: "material-symbols-outlined", "print" }
                    if is_testing { "Mencetak..." } else { "Test Print" }
                }
            }
        }
    }
}

#[component]
fn Toggle(
    on: bool,
    #[props(into)] name: String,
    #[props(into)] desc: String,
    ontoggle: EventHandler<MouseEvent>,
) -> Element {
    let switch_class = if on { "switch switch-on" } else { "switch" };
    rsx! {
        div { class: "field",
            label { class: "switch-label", onclick: move |e| ontoggle.call(e),
                div { class: "{switch_class}",
                    span { class: "switch-thumb" }
                }
                div { class: "switch-text",
                    span { class: "switch-name", "{name}" }
                    span { class: "switch-desc", "{desc}" }
                }
            }
        }
    }
}

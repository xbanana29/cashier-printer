import { b as attr_class, e as ensure_array_like, d as escape_html, a as attr } from "../../../chunks/renderer.js";
import "@tauri-apps/api/core";
import "@tauri-apps/plugin-updater";
import "@tauri-apps/plugin-process";
import "@tauri-apps/api/app";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let currentVersion = "";
    let updateStatus = "idle";
    let settings = {
      default_printer: "",
      paper_size: "80mm",
      store_name: "",
      footer_text: "",
      auto_cut: true,
      pc_name: "",
      extra_feeds: 0
    };
    let printers = [];
    let loadingPrinters = false;
    let saving = false;
    let testing = false;
    let connType = "os";
    let peers = [];
    let syncing = false;
    function formatLastSeen(ts) {
      if (!ts) return "—";
      const diff = Math.floor(Date.now() / 1e3) - ts;
      if (diff < 60) return `${diff}d lalu`;
      if (diff < 3600) return `${Math.floor(diff / 60)}m lalu`;
      return `${Math.floor(diff / 3600)}j lalu`;
    }
    $$renderer2.push(`<div class="page svelte-1i19ct2"><h2 class="svelte-1i19ct2">Pengaturan</h2> <form><div class="section svelte-1i19ct2"><div class="section-header svelte-1i19ct2"><span class="material-symbols-outlined section-icon svelte-1i19ct2">print</span> <span class="section-title svelte-1i19ct2">Printer</span></div> <div class="field svelte-1i19ct2"><span class="field-label svelte-1i19ct2">Jenis Koneksi</span> <div class="conn-tabs svelte-1i19ct2"><button type="button"${attr_class("conn-tab svelte-1i19ct2", void 0, { "conn-tab-active": connType === "os" })}><span class="material-symbols-outlined svelte-1i19ct2">usb</span> USB / CUPS</button> <button type="button"${attr_class("conn-tab svelte-1i19ct2", void 0, { "conn-tab-active": connType === "serial" })}><span class="material-symbols-outlined svelte-1i19ct2">cable</span> Serial / COM</button> <button type="button"${attr_class("conn-tab svelte-1i19ct2", void 0, { "conn-tab-active": connType === "network" })}><span class="material-symbols-outlined svelte-1i19ct2">wifi</span> Jaringan (LAN)</button></div></div> `);
    {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div class="field svelte-1i19ct2"><label for="printer" class="field-label svelte-1i19ct2">Pilih Printer</label> <div class="row-gap svelte-1i19ct2">`);
      $$renderer2.select(
        {
          id: "printer",
          class: "field-select",
          value: settings.default_printer,
          disabled: loadingPrinters
        },
        ($$renderer3) => {
          $$renderer3.option({ value: "" }, ($$renderer4) => {
            $$renderer4.push(`— Pilih printer —`);
          });
          $$renderer3.push(`<!--[-->`);
          const each_array = ensure_array_like(printers);
          for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
            let p = each_array[$$index];
            $$renderer3.option({ value: p.name }, ($$renderer4) => {
              $$renderer4.push(`${escape_html(p.name)}${escape_html(p.is_default ? " ✓" : "")}`);
            });
          }
          $$renderer3.push(`<!--]-->`);
        },
        "svelte-1i19ct2"
      );
      $$renderer2.push(` <button type="button" class="icon-btn svelte-1i19ct2" title="Refresh"${attr("disabled", loadingPrinters, true)}><span class="material-symbols-outlined svelte-1i19ct2">${escape_html("refresh")}</span></button></div> <p class="field-support svelte-1i19ct2">Printer yang terdeteksi di sistem (USB, Bluetooth, jaringan via CUPS).</p></div>`);
    }
    $$renderer2.push(`<!--]--> <div class="field svelte-1i19ct2"><span class="field-label svelte-1i19ct2">Ukuran Kertas</span> <div class="chip-group svelte-1i19ct2"><!--[-->`);
    const each_array_2 = ensure_array_like(["58mm", "75mm", "80mm"]);
    for (let $$index_2 = 0, $$length = each_array_2.length; $$index_2 < $$length; $$index_2++) {
      let size = each_array_2[$$index_2];
      $$renderer2.push(`<label${attr_class("chip svelte-1i19ct2", void 0, { "chip-selected": settings.paper_size === size })}><input type="radio"${attr("checked", settings.paper_size === size, true)}${attr("value", size)} class="svelte-1i19ct2"/> ${escape_html(size)}</label>`);
    }
    $$renderer2.push(`<!--]--></div></div> <div class="field svelte-1i19ct2"><label class="switch-label svelte-1i19ct2"><div${attr_class("switch svelte-1i19ct2", void 0, { "switch-on": settings.auto_cut })}><input type="checkbox"${attr("checked", settings.auto_cut, true)} class="svelte-1i19ct2"/> <span class="switch-thumb svelte-1i19ct2"></span></div> <div class="switch-text svelte-1i19ct2"><span class="switch-name svelte-1i19ct2">Potong kertas otomatis</span> <span class="switch-desc svelte-1i19ct2">Matikan untuk TM-U220 tanpa cutter.</span></div></label></div> <div class="field svelte-1i19ct2"><label for="extra-feeds" class="field-label svelte-1i19ct2">Baris Kosong Setelah Cetak</label> `);
    $$renderer2.select(
      {
        id: "extra-feeds",
        class: "field-select field-select-short",
        value: settings.extra_feeds
      },
      ($$renderer3) => {
        $$renderer3.option({ value: 0 }, ($$renderer4) => {
          $$renderer4.push(`0`);
        });
        $$renderer3.option({ value: 1 }, ($$renderer4) => {
          $$renderer4.push(`1`);
        });
        $$renderer3.option({ value: 2 }, ($$renderer4) => {
          $$renderer4.push(`2`);
        });
        $$renderer3.option({ value: 3 }, ($$renderer4) => {
          $$renderer4.push(`3`);
        });
        $$renderer3.option({ value: 4 }, ($$renderer4) => {
          $$renderer4.push(`4`);
        });
        $$renderer3.option({ value: 5 }, ($$renderer4) => {
          $$renderer4.push(`5`);
        });
      },
      "svelte-1i19ct2"
    );
    $$renderer2.push(` <p class="field-support svelte-1i19ct2">Tambah jika tulisan terakhir tidak keluar dari kepala cetak.</p></div></div> <div class="section svelte-1i19ct2"><div class="section-header svelte-1i19ct2"><span class="material-symbols-outlined section-icon svelte-1i19ct2">receipt_long</span> <span class="section-title svelte-1i19ct2">Template Cetak</span></div> <div class="field svelte-1i19ct2"><label for="store" class="field-label svelte-1i19ct2">Nama Toko <span class="label-optional svelte-1i19ct2">(opsional)</span></label> <input id="store" class="field-input svelte-1i19ct2" type="text" placeholder="Contoh: Toko Makmur Jaya"${attr("value", settings.store_name)}/></div> <div class="field svelte-1i19ct2"><label for="footer" class="field-label svelte-1i19ct2">Footer <span class="label-optional svelte-1i19ct2">(opsional)</span></label> <input id="footer" class="field-input svelte-1i19ct2" type="text" placeholder="Contoh: Terima kasih atas pesanan Anda!"${attr("value", settings.footer_text)}/></div> <div class="field svelte-1i19ct2"><label for="pcname" class="field-label svelte-1i19ct2">Nama PC / Kasir</label> <input id="pcname" class="field-input svelte-1i19ct2" type="text" placeholder="Contoh: Kasir 1"${attr("value", settings.pc_name)}/> <p class="field-support svelte-1i19ct2">Muncul di baris bawah struk dan di kolom riwayat pesanan.</p></div></div> <div class="section svelte-1i19ct2"><div class="section-header svelte-1i19ct2"><span class="material-symbols-outlined section-icon svelte-1i19ct2">system_update</span> <span class="section-title svelte-1i19ct2">Tentang &amp; Update</span></div> <div class="about-row svelte-1i19ct2"><span class="about-label svelte-1i19ct2">Versi saat ini</span> <span class="about-value svelte-1i19ct2">v${escape_html(currentVersion)}</span></div> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <div class="field svelte-1i19ct2" style="margin-top: 8px; margin-bottom: 0"><button type="button" class="btn-outlined svelte-1i19ct2"${attr("disabled", updateStatus === "downloading", true)}><span class="material-symbols-outlined svelte-1i19ct2">${escape_html("sync")}</span> ${escape_html("Cek Update")}</button></div></div> <div class="section svelte-1i19ct2"><div class="section-header svelte-1i19ct2"><span class="material-symbols-outlined section-icon svelte-1i19ct2">lan</span> <span class="section-title svelte-1i19ct2">Sinkronisasi LAN</span></div> <p class="field-support svelte-1i19ct2" style="margin-bottom: 12px">Sinkron otomatis antar PC dalam satu jaringan WiFi/LAN. Tidak perlu internet.</p> <div class="field svelte-1i19ct2"><span class="field-label svelte-1i19ct2">Perangkat Terhubung</span> `);
    if (peers.length === 0) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<p class="peer-empty svelte-1i19ct2">Belum ada perangkat lain terdeteksi di jaringan ini.</p>`);
    } else {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`<div class="peer-list svelte-1i19ct2"><!--[-->`);
      const each_array_3 = ensure_array_like(peers);
      for (let $$index_3 = 0, $$length = each_array_3.length; $$index_3 < $$length; $$index_3++) {
        let peer = each_array_3[$$index_3];
        $$renderer2.push(`<div class="peer-row svelte-1i19ct2"><span class="material-symbols-outlined peer-icon svelte-1i19ct2">computer</span> <div class="peer-info svelte-1i19ct2"><span class="peer-name svelte-1i19ct2">${escape_html(peer.pc_name || "Tanpa nama")}</span> <span class="peer-addr svelte-1i19ct2">${escape_html(peer.addr)}</span></div> <div class="peer-meta svelte-1i19ct2">`);
        if (peer.orders_synced > 0) {
          $$renderer2.push("<!--[0-->");
          $$renderer2.push(`<span class="peer-badge svelte-1i19ct2">${escape_html(peer.orders_synced)} order</span>`);
        } else {
          $$renderer2.push("<!--[-1-->");
        }
        $$renderer2.push(`<!--]--> <span class="peer-seen svelte-1i19ct2">${escape_html(formatLastSeen(peer.last_seen))}</span></div></div>`);
      }
      $$renderer2.push(`<!--]--></div>`);
    }
    $$renderer2.push(`<!--]--></div> <div class="field svelte-1i19ct2" style="margin-bottom: 0"><button type="button" class="btn-outlined svelte-1i19ct2"${attr("disabled", syncing, true)}><span class="material-symbols-outlined svelte-1i19ct2">${escape_html("sync")}</span> ${escape_html("Sync Sekarang")}</button></div></div> <div class="actions svelte-1i19ct2"><button type="submit" class="btn-filled svelte-1i19ct2"${attr("disabled", saving, true)}><span class="material-symbols-outlined svelte-1i19ct2">save</span> ${escape_html("Simpan Pengaturan")}</button> <button type="button" class="btn-outlined svelte-1i19ct2"${attr("disabled", testing, true)}><span class="material-symbols-outlined svelte-1i19ct2">print</span> ${escape_html("Test Print")}</button></div></form></div>`);
  });
}
export {
  _page as default
};

import { a as attr, c as attr_style, d as escape_html, ac as bind_props, f as stringify, b as attr_class, ab as derived } from "../../../chunks/renderer.js";
import { invoke } from "@tauri-apps/api/core";
import { s as showToast, a as showError } from "../../../chunks/stores.svelte.js";
/* empty css                                                           */
const api = {
  // Orders
  createOrder: (customerName, content, orderType) => invoke("create_order", { customerName, content, orderType }),
  getOrders: (orderType) => invoke("get_orders", { orderType }),
  getOrder: (id) => invoke("get_order", { id }),
  updateOrder: (id, customerName, content) => invoke("update_order", { id, customerName, content }),
  deleteOrder: (id) => invoke("delete_order", { id }),
  purgeOldOrders: () => invoke("purge_old_orders"),
  // Print
  listPrinters: () => invoke("list_printers"),
  listSerialPorts: () => invoke("list_serial_ports"),
  printOrder: (orderId) => invoke("print_order", { orderId }),
  previewReceipt: (orderId) => invoke("preview_receipt", { orderId }),
  reprintOrder: (orderId) => invoke("reprint_order", { orderId }),
  testPrint: () => invoke("test_print"),
  // Settings
  getSettings: () => invoke("get_settings"),
  saveSettings: (settings) => invoke("save_settings", { settings }),
  // LAN Sync
  getPeers: () => invoke("get_peers"),
  syncNow: () => invoke("sync_now")
};
function GuidedTextarea($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let {
      id,
      value = void 0,
      charWidth,
      placeholder = "",
      rows = 8,
      disabled = false,
      onkeydown,
      oninput
    } = $$props;
    $$renderer2.push(`<textarea${attr("id", id)}${attr("placeholder", placeholder)}${attr("rows", rows)}${attr("disabled", disabled, true)} class="guided-textarea svelte-1odbc31"${attr_style(`--guide-col: ${stringify(charWidth)}`)}>`);
    const $$body = escape_html(value);
    if ($$body) {
      $$renderer2.push(`${$$body}`);
    }
    $$renderer2.push(`</textarea>`);
    bind_props($$props, { value });
  });
}
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    const RECEIPT_TEMPLATE = "Jenis : \nGudang : ";
    let orderType = "order";
    let customerName = "";
    let content = "";
    let isSubmitting = false;
    let charWidth = 48;
    const nameLabel = derived(() => "Nama Pelanggan");
    const namePlaceholder = derived(() => "Contoh: Pak Budi");
    const submitLabel = derived(() => "Cetak Pesanan");
    const emptyNameMsg = derived(() => "Nama pelanggan wajib diisi");
    const emptyContentMsg = derived(() => "Isi pesanan wajib diisi");
    const successMsg = derived(() => "Pesanan berhasil dicetak");
    async function submit() {
      const name = customerName.trim();
      const body = content.trim();
      if (!name) {
        showToast(emptyNameMsg(), "error");
        return;
      }
      if (!body) {
        showToast(emptyContentMsg(), "error");
        return;
      }
      isSubmitting = true;
      try {
        const id = await api.createOrder(name, body, orderType);
        await api.printOrder(id);
        showToast(successMsg());
        customerName = "";
        content = orderType === "receipt" ? RECEIPT_TEMPLATE : "";
      } catch (err) {
        showError(err);
      } finally {
        isSubmitting = false;
      }
    }
    function handleKeydown(e) {
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
        e.preventDefault();
        submit();
      }
    }
    let $$settled = true;
    let $$inner_renderer;
    function $$render_inner($$renderer3) {
      $$renderer3.push(`<div class="page svelte-1ax4549"><h2 class="svelte-1ax4549">${escape_html("Pesanan Baru")}</h2> <div class="type-chips svelte-1ax4549"><button type="button"${attr_class("chip svelte-1ax4549", void 0, { "chip-selected": orderType === "order" })}><span class="material-symbols-outlined svelte-1ax4549">receipt</span> Pesanan</button> <button type="button"${attr_class("chip svelte-1ax4549", void 0, { "chip-selected": orderType === "receipt" })}><span class="material-symbols-outlined svelte-1ax4549">assignment</span> Tanda Terima</button></div> <form><div class="field svelte-1ax4549"><label for="customer" class="field-label svelte-1ax4549">${escape_html(nameLabel())}</label> <input id="customer" class="field-input svelte-1ax4549" type="text"${attr("placeholder", namePlaceholder())}${attr("value", customerName)}${attr("disabled", isSubmitting, true)} autocomplete="off"/></div> <div class="field svelte-1ax4549"><label for="content" class="field-label svelte-1ax4549">${escape_html("Isi Pesanan")} <span class="label-hint svelte-1ax4549">— garis biru = batas ${escape_html(charWidth)} kolom</span></label> `);
      GuidedTextarea($$renderer3, {
        id: "content",
        charWidth,
        placeholder: "Tulis atau paste daftar pesanan di sini...",
        rows: 10,
        disabled: isSubmitting,
        onkeydown: handleKeydown,
        get value() {
          return content;
        },
        set value($$value) {
          content = $$value;
          $$settled = false;
        }
      });
      $$renderer3.push(`<!----> <span class="field-support svelte-1ax4549">Ctrl+Enter untuk cetak langsung</span></div> <button type="submit" class="btn-filled svelte-1ax4549"${attr("disabled", isSubmitting, true)}><span class="material-symbols-outlined svelte-1ax4549">print</span> ${escape_html(isSubmitting ? "Mencetak..." : submitLabel())}</button></form></div>`);
    }
    do {
      $$settled = true;
      $$inner_renderer = $$renderer2.copy();
      $$render_inner($$inner_renderer);
    } while (!$$settled);
    $$renderer2.subsume($$inner_renderer);
  });
}
export {
  _page as default
};

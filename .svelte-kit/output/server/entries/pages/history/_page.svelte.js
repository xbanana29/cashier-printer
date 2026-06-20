import { b as attr_class, a as attr } from "../../../chunks/renderer.js";
import "@tauri-apps/api/core";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let activeTab = "order";
    let search = "";
    $$renderer2.push(`<div class="page svelte-1xl2tfr"><div class="page-header svelte-1xl2tfr"><h2 class="svelte-1xl2tfr">Riwayat</h2> <button class="btn-icon-sm svelte-1xl2tfr" title="Muat ulang"><span class="material-symbols-outlined svelte-1xl2tfr">refresh</span></button></div> <div class="tabs svelte-1xl2tfr"><button${attr_class("tab svelte-1xl2tfr", void 0, { "tab-active": activeTab === "order" })}><span class="material-symbols-outlined svelte-1xl2tfr">receipt</span> Pesanan</button> <button${attr_class("tab svelte-1xl2tfr", void 0, { "tab-active": activeTab === "receipt" })}><span class="material-symbols-outlined svelte-1xl2tfr">assignment</span> Tanda Terima</button></div> <div class="search-bar svelte-1xl2tfr"><span class="material-symbols-outlined search-icon svelte-1xl2tfr">search</span> <input class="search-input svelte-1xl2tfr" type="search" placeholder="Cari nama..."${attr("value", search)}/> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--></div> `);
    {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<p class="state-msg svelte-1xl2tfr">Memuat...</p>`);
    }
    $$renderer2.push(`<!--]--></div> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]-->`);
  });
}
export {
  _page as default
};

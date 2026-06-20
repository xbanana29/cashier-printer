import { s as store_get, u as unsubscribe_stores, d as escape_html } from "../../../../chunks/renderer.js";
import "@sveltejs/kit/internal";
import "../../../../chunks/exports.js";
import "../../../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../../../chunks/root.js";
import "../../../../chunks/state.svelte.js";
import { p as page } from "../../../../chunks/stores.js";
import "@tauri-apps/api/core";
/* empty css                                                              */
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    Number(store_get($$store_subs ??= {}, "$page", page).params.id);
    let $$settled = true;
    let $$inner_renderer;
    function $$render_inner($$renderer3) {
      $$renderer3.push(`<div class="page svelte-1ul2axt"><div class="page-header svelte-1ul2axt"><h2 class="svelte-1ul2axt">${escape_html("Edit Pesanan")}</h2> <a href="/history" class="btn-text svelte-1ul2axt"><span class="material-symbols-outlined svelte-1ul2axt">arrow_back</span> Kembali</a></div> `);
      {
        $$renderer3.push("<!--[0-->");
        $$renderer3.push(`<p class="state-msg svelte-1ul2axt">Memuat...</p>`);
      }
      $$renderer3.push(`<!--]--></div>`);
    }
    do {
      $$settled = true;
      $$inner_renderer = $$renderer2.copy();
      $$render_inner($$inner_renderer);
    } while (!$$settled);
    $$renderer2.subsume($$inner_renderer);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
export {
  _page as default
};

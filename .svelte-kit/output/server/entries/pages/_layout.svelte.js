import { e as ensure_array_like, s as store_get, a as attr, b as attr_class, c as attr_style, d as escape_html, u as unsubscribe_stores, f as stringify } from "../../chunks/renderer.js";
import { p as page } from "../../chunks/stores.js";
import "@tauri-apps/api/core";
import { t as toast } from "../../chunks/stores.svelte.js";
import "@tauri-apps/plugin-opener";
function _layout($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let { children } = $$props;
    const navLinks = [
      { href: "/new", label: "Baru", icon: "receipt" },
      { href: "/history", label: "Riwayat", icon: "history" },
      { href: "/settings", label: "Setelan", icon: "settings" }
    ];
    $$renderer2.push(`<div class="app svelte-12qhfyh"><nav class="nav-rail svelte-12qhfyh"><div class="rail-brand svelte-12qhfyh"><span class="brand-logo svelte-12qhfyh">PPO</span> <span class="brand-name svelte-12qhfyh">Print Paste<br/>Order</span></div> <div class="rail-items svelte-12qhfyh"><!--[-->`);
    const each_array = ensure_array_like(navLinks);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let link = each_array[$$index];
      const active = store_get($$store_subs ??= {}, "$page", page).url.pathname === link.href || store_get($$store_subs ??= {}, "$page", page).url.pathname.startsWith("/edit") && link.href === "/history";
      $$renderer2.push(`<a${attr("href", link.href)}${attr_class("rail-item svelte-12qhfyh", void 0, { "active": active })}><div class="rail-indicator svelte-12qhfyh"><span class="material-symbols-outlined svelte-12qhfyh"${attr_style("", {
        "font-variation-settings": active ? "'FILL' 1,'wght' 500,'GRAD' 0,'opsz' 24" : "'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24"
      })}>${escape_html(link.icon)}</span></div> <span class="rail-label svelte-12qhfyh">${escape_html(link.label)}</span></a>`);
    }
    $$renderer2.push(`<!--]--></div></nav> <div class="main-area svelte-12qhfyh"><main class="content svelte-12qhfyh">`);
    children($$renderer2);
    $$renderer2.push(`<!----></main> <footer class="watermark svelte-12qhfyh"><button class="watermark-link svelte-12qhfyh">CV REJEKI AMERTA JAYA  ·  © 2026</button></footer></div></div> `);
    if (toast.message) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div${attr_class(`toast toast-${stringify(toast.message.type)}`, "svelte-12qhfyh")} role="alert">${escape_html(toast.message.text)}</div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]-->`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
export {
  _layout as default
};

<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { showToast, showError } from '$lib/stores.svelte';
  import type { Order } from '$lib/types';

  let orders: Order[] = $state([]);
  let search = $state('');
  let loading = $state(true);
  let deletingId: number | null = $state(null);
  let confirmDeleteId: number | null = $state(null);

  const PAGE_SIZE = 25;
  let page = $state(1);

  const filtered = $derived(
    search.trim() === ''
      ? orders
      : orders.filter(o => o.customer_name.toLowerCase().includes(search.trim().toLowerCase()))
  );

  const totalPages = $derived(Math.max(1, Math.ceil(filtered.length / PAGE_SIZE)));
  const paginated = $derived(filtered.slice((page - 1) * PAGE_SIZE, page * PAGE_SIZE));

  $effect(() => { filtered; page = 1; });

  let previewOrderId: number | null = $state(null);
  let previewText: string = $state('');
  let previewLoading = $state(false);

  // Convert plain-text preview lines into HTML divs so CSS can accurately
  // mirror what the ESC/POS printer produces:
  //   - line 0 (customer name)  → .preview-cname  (2× height, bold, centred)
  //   - lines with [ ] (items)  → .preview-item    (font from content_font_size)
  //   - all other lines         → .line            (normal)
  const previewHtml = $derived(
    previewText.split('\n').map((line, i) => {
      const esc = line
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');
      const safe = esc || '&nbsp;';
      if (i === 0) return `<div class="line preview-cname">${safe}</div>`;
      if (esc.includes('[ ]')) return `<div class="line preview-item">${esc}</div>`;
      return `<div class="line">${safe}</div>`;
    }).join('')
  );

  onMount(async () => {
    await loadOrders();
    try {
      const s = await api.getSettings();
      paperWidth = CHAR_WIDTH[s.paper_size] ?? 48;
      pcName = s.pc_name;
      contentFontSize = s.content_font_size ?? 'normal';
    } catch { /* ignore */ }
  });

  async function loadOrders() {
    loading = true;
    try {
      await api.purgeOldOrders();
      orders = await api.getOrders();
    } catch (err) {
      showError(err);
    } finally {
      loading = false;
    }
  }

  async function openPreview(order: Order) {
    previewOrderId = order.id;
    previewText = '';
    previewLoading = true;
    try {
      previewText = await api.previewReceipt(order.id);
    } catch (err) {
      showError(err);
      previewOrderId = null;
    } finally {
      previewLoading = false;
    }
  }

  function closePreview() {
    previewOrderId = null;
    previewText = '';
  }

  async function reprint(id: number) {
    try {
      await api.reprintOrder(id);
      showToast('Pesanan berhasil dicetak ulang');
    } catch (err) {
      showError(err);
    }
  }

  async function deleteOrder(id: number) {
    deletingId = id;
    confirmDeleteId = null;
    try {
      await api.deleteOrder(id);
      orders = orders.filter(o => o.id !== id);
      showToast('Pesanan dihapus');
    } catch (err) {
      showError(err);
    } finally {
      deletingId = null;
    }
  }

  function formatDate(dt: string): string {
    try {
      return new Date(dt.replace(' ', 'T')).toLocaleString('id-ID', {
        day: '2-digit', month: 'short', year: 'numeric',
        hour: '2-digit', minute: '2-digit',
      });
    } catch { return dt; }
  }

  function contentPreview(content: string): string {
    const first = content.split('\n')[0];
    return first.length > 60 ? first.slice(0, 60) + '…' : first;
  }

  const CHAR_WIDTH: Record<string, number> = { '58mm': 32, '75mm': 42, '80mm': 48 };
  let paperWidth = $state(48);
  let pcName = $state('');
  let contentFontSize = $state('normal');

  // CSS font-size for item lines — wide/large are 2× width, so double the visual size.
  const itemFontSize = $derived(
    (contentFontSize === 'wide' || contentFontSize === 'large') ? '1.56rem' : '0.78rem'
  );
  // CSS line-height for item lines — tall/large are 2× height.
  const itemLineHeight = $derived(
    (contentFontSize === 'tall' || contentFontSize === 'large') ? '3.1' : '1.55'
  );

</script>

<div class="page">
  <div class="page-header">
    <h2>Riwayat Pesanan</h2>
    <button class="btn-icon-sm" onclick={loadOrders} title="Muat ulang">
      <span class="material-symbols-outlined">refresh</span>
    </button>
  </div>

  <!-- MD3 Search bar -->
  <div class="search-bar">
    <span class="material-symbols-outlined search-icon">search</span>
    <input
      class="search-input"
      type="search"
      placeholder="Cari nama pelanggan..."
      bind:value={search}
    />
    {#if search}
      <span class="search-count">{filtered.length} hasil</span>
    {/if}
  </div>

  {#if loading}
    <p class="state-msg">Memuat...</p>
  {:else if orders.length === 0}
    <p class="state-msg">Belum ada pesanan.</p>
  {:else if filtered.length === 0}
    <p class="state-msg">Tidak ada pesanan untuk "<strong>{search}</strong>".</p>
  {:else}
    <div class="list">
      {#each paginated as order (order.id)}
        <div class="card">
          <div class="card-body">
            <div class="card-name">{order.customer_name}</div>
            <div class="card-preview">{contentPreview(order.content)}</div>
            <div class="card-meta">
              <span class="card-date">{formatDate(order.created_at)}</span>
              {#if pcName}
                <span class="card-pc">{pcName}</span>
              {/if}
            </div>
          </div>
          <div class="card-actions">
            {#if confirmDeleteId === order.id}
              <span class="confirm-label">Hapus?</span>
              <button
                class="icon-btn icon-btn-danger"
                onclick={() => deleteOrder(order.id)}
                disabled={deletingId === order.id}
                title="Ya, hapus"
              >
                <span class="material-symbols-outlined">check</span>
              </button>
              <button class="icon-btn" onclick={() => confirmDeleteId = null} title="Batal">
                <span class="material-symbols-outlined">close</span>
              </button>
            {:else}
              <button class="icon-btn" onclick={() => openPreview(order)} title="Preview struk">
                <span class="material-symbols-outlined">receipt_long</span>
              </button>
              <a href="/edit/{order.id}" class="icon-btn" title="Edit">
                <span class="material-symbols-outlined">edit</span>
              </a>
              <button class="icon-btn" onclick={() => reprint(order.id)} title="Cetak ulang">
                <span class="material-symbols-outlined">print</span>
              </button>
              <button
                class="icon-btn icon-btn-danger"
                onclick={() => confirmDeleteId = order.id}
                title="Hapus"
              >
                <span class="material-symbols-outlined">delete</span>
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>

    {#if totalPages > 1}
      <div class="pagination">
        <button class="pg-btn" onclick={() => page--} disabled={page === 1}>
          <span class="material-symbols-outlined">chevron_left</span>
        </button>
        {#each Array.from({ length: totalPages }, (_, i) => i + 1) as p}
          <button class="pg-btn" class:pg-active={p === page} onclick={() => page = p}>{p}</button>
        {/each}
        <button class="pg-btn" onclick={() => page++} disabled={page === totalPages}>
          <span class="material-symbols-outlined">chevron_right</span>
        </button>
        <span class="pg-info">
          {(page - 1) * PAGE_SIZE + 1}–{Math.min(page * PAGE_SIZE, filtered.length)} / {filtered.length}
        </span>
      </div>
    {/if}
  {/if}
</div>

<!-- Receipt preview modal -->
{#if previewOrderId !== null}
  <div
    class="overlay"
    onclick={closePreview}
    role="button"
    tabindex="0"
    onkeydown={(e) => (e.key === 'Escape' || e.key === 'Enter') && closePreview()}
  >
    <div
      class="modal"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onkeydown={(e) => e.key === 'Escape' && closePreview()}
    >
      <div class="modal-header">
        <span class="modal-title">Preview Struk</span>
        <div class="modal-header-actions">
          <button
            class="btn-filled-sm"
            onclick={() => { reprint(previewOrderId!); closePreview(); }}
          >
            <span class="material-symbols-outlined">print</span>
            Cetak Ulang
          </button>
          <button class="modal-close" onclick={closePreview}>
            <span class="material-symbols-outlined">close</span>
          </button>
        </div>
      </div>

      <div class="receipt-wrapper">
        {#if previewLoading}
          <p class="receipt-loading">Memuat preview...</p>
        {:else}
          <div class="paper" style="--cols: {paperWidth}; --item-fs: {itemFontSize}; --item-lh: {itemLineHeight}">
            <!-- {@html} is safe here: content is escaped above -->
            <div class="receipt-text">{@html previewHtml}</div>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .page { max-width: 720px; margin: 0 auto; }

  /* ── Header ── */
  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1.25rem;
  }

  h2 {
    font-size: 1.375rem;
    font-weight: 500;
    color: var(--md-on-surface);
    letter-spacing: .01em;
  }

  .btn-icon-sm {
    width: 40px; height: 40px;
    border-radius: 20px;
    border: none;
    background: transparent;
    display: flex; align-items: center; justify-content: center;
    cursor: pointer;
    color: var(--md-on-surface-variant);
    transition: background .15s;
  }
  .btn-icon-sm:hover { background: var(--md-surface-variant); }
  .btn-icon-sm .material-symbols-outlined { font-size: 20px; }

  /* ── Search bar ── */
  .search-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--md-surface-container);
    border-radius: 28px;
    padding: 0 16px;
    height: 48px;
    margin-bottom: 1rem;
    transition: box-shadow .15s;
  }
  .search-bar:focus-within { box-shadow: var(--md-elev-1); }

  .search-icon {
    font-size: 20px;
    color: var(--md-on-surface-variant);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    border: none;
    background: transparent;
    font-size: .9375rem;
    font-family: 'Roboto', sans-serif;
    color: var(--md-on-surface);
    outline: none;
  }
  .search-input::placeholder { color: var(--md-on-surface-variant); }

  .search-count {
    font-size: .75rem;
    color: var(--md-on-surface-variant);
    white-space: nowrap;
  }

  /* ── State messages ── */
  .state-msg { color: var(--md-on-surface-variant); font-size: .9rem; padding: 1rem 0; }
  .state-msg strong { color: var(--md-on-surface); }

  /* ── Cards ── */
  .list { display: flex; flex-direction: column; gap: 6px; }

  .card {
    display: flex;
    align-items: center;
    gap: 12px;
    background: #fff;
    border-radius: 12px;
    padding: 12px 8px 12px 16px;
    box-shadow: var(--md-elev-1);
    transition: box-shadow .15s;
  }
  .card:hover { box-shadow: var(--md-elev-2); }

  .card-body { flex: 1; min-width: 0; }
  .card-name {
    font-weight: 500;
    font-size: .9375rem;
    color: var(--md-on-surface);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .card-preview {
    font-size: .8125rem;
    color: var(--md-on-surface-variant);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    margin-top: 2px;
  }
  .card-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
    flex-wrap: wrap;
  }
  .card-date {
    font-size: .75rem;
    color: var(--md-on-surface-variant);
  }
  .card-pc {
    font-size: .68rem;
    font-weight: 500;
    color: var(--md-on-primary);
    background: var(--md-primary);
    border-radius: 4px;
    padding: 1px 6px;
    letter-spacing: .02em;
  }

  /* ── Icon buttons ── */
  .card-actions { display: flex; gap: 2px; flex-shrink: 0; }

  .icon-btn {
    width: 40px; height: 40px;
    border-radius: 20px;
    border: none;
    background: transparent;
    display: flex; align-items: center; justify-content: center;
    cursor: pointer;
    color: var(--md-on-surface-variant);
    text-decoration: none;
    transition: background .15s, color .15s;
  }
  .icon-btn:hover { background: var(--md-surface-variant); color: var(--md-on-surface); }
  .icon-btn:disabled { opacity: .38; cursor: not-allowed; }
  .icon-btn .material-symbols-outlined { font-size: 20px; }

  .icon-btn-danger:hover {
    background: var(--md-error-container);
    color: var(--md-error);
  }

  .confirm-label {
    font-size: .75rem;
    font-weight: 500;
    color: var(--md-error);
    align-self: center;
    white-space: nowrap;
    padding: 0 4px;
  }

  /* ── Pagination ── */
  .pagination {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-top: 1rem;
    flex-wrap: wrap;
  }

  .pg-btn {
    min-width: 36px; height: 36px;
    padding: 0 6px;
    border: 1px solid var(--md-outline-variant);
    border-radius: 18px;
    background: transparent;
    font-size: .875rem;
    font-family: 'Roboto', sans-serif;
    cursor: pointer;
    color: var(--md-on-surface-variant);
    display: flex; align-items: center; justify-content: center;
    transition: background .12s;
  }
  .pg-btn .material-symbols-outlined { font-size: 18px; }
  .pg-btn:hover:not(:disabled) { background: var(--md-surface-variant); }
  .pg-btn:disabled { opacity: .35; cursor: not-allowed; }
  .pg-btn.pg-active {
    background: var(--md-primary);
    color: var(--md-on-primary);
    border-color: var(--md-primary);
    font-weight: 700;
  }

  .pg-info { margin-left: 6px; font-size: .75rem; color: var(--md-on-surface-variant); }

  /* ── Modal ── */
  .overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,.5);
    display: flex; align-items: center; justify-content: center;
    z-index: 100;
  }

  .modal {
    background: #f0ede8;
    border-radius: 16px;
    max-height: 88vh;
    display: flex; flex-direction: column;
    box-shadow: var(--md-elev-3);
    max-width: min(92vw, 640px);
    width: fit-content;
    min-width: 280px;
  }

  .modal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 16px;
    border-bottom: 1px solid rgba(0,0,0,.1);
    background: #e8e4de;
    border-radius: 16px 16px 0 0;
    gap: 12px;
  }

  .modal-title { font-weight: 500; font-size: .9375rem; color: #444; }
  .modal-header-actions { display: flex; align-items: center; gap: 8px; }

  .btn-filled-sm {
    display: flex; align-items: center; gap: 6px;
    height: 36px; padding: 0 16px;
    background: var(--md-primary); color: var(--md-on-primary);
    border: none; border-radius: 18px;
    font-size: .8125rem; font-weight: 500;
    font-family: 'Roboto', sans-serif;
    cursor: pointer;
    transition: box-shadow .15s;
  }
  .btn-filled-sm .material-symbols-outlined { font-size: 16px; }
  .btn-filled-sm:hover { box-shadow: var(--md-elev-1); }

  .modal-close {
    width: 36px; height: 36px;
    border-radius: 18px;
    border: none; background: transparent;
    display: flex; align-items: center; justify-content: center;
    cursor: pointer; color: #666;
    transition: background .15s;
  }
  .modal-close:hover { background: rgba(0,0,0,.08); }
  .modal-close .material-symbols-outlined { font-size: 20px; }

  .receipt-wrapper {
    overflow-y: auto;
    padding: 1.25rem 1rem;
    display: flex; justify-content: center;
  }

  .receipt-loading { font-size: .85rem; color: #888; padding: 2rem; }

  .paper {
    width: calc(var(--cols) * 1ch + 2rem);
    background: #fffdf8;
    box-shadow: 0 2px 8px rgba(0,0,0,.15), 0 0 0 1px rgba(0,0,0,.06);
    border-radius: 2px;
    padding: 1rem;
    position: relative;
  }

  .paper::after {
    content: '';
    display: block;
    height: .5rem;
    background: repeating-linear-gradient(90deg, #fffdf8 0 6px, transparent 6px 10px);
    margin-top: .5rem;
    border-top: 1px dashed #ccc;
  }

  .receipt-text {
    font-family: 'Courier New', Courier, monospace;
    font-size: .78rem;
    line-height: 1.55;
    color: #111;
    letter-spacing: 0;
  }

  /* Each line is a div; white-space:pre preserves spaces within the line */
  .receipt-text :global(.line) {
    white-space: pre;
    display: block;
  }

  /* Customer name: mirrors ESC/POS justify-CENTER + GS!0x10 (double-height) + bold */
  .receipt-text :global(.preview-cname) {
    font-size: 1.56rem; /* 2× base .78rem */
    font-weight: 700;
    line-height: 1.3;
    text-align: center;
  }

  /* Content items: font-size and line-height driven by CSS vars set from content_font_size */
  .receipt-text :global(.preview-item) {
    font-size: var(--item-fs, .78rem);
    line-height: var(--item-lh, 1.55);
  }
</style>

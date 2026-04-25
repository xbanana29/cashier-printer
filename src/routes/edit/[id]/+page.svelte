<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { api } from '$lib/api';
  import { showToast, showError } from '$lib/stores.svelte';
  import GuidedTextarea from '$lib/GuidedTextarea.svelte';

  const CHAR_WIDTH: Record<string, number> = { '58mm': 32, '75mm': 42, '80mm': 48 };

  let customerName = $state('');
  let content = $state('');
  let createdAt = $state('');
  let orderType = $state('order');
  let loading = $state(true);
  let saving = $state(false);
  let charWidth = $state(48);

  const id = Number($page.params.id);

  onMount(async () => {
    try {
      const [order, settings] = await Promise.all([
        api.getOrder(id),
        api.getSettings(),
      ]);
      customerName = order.customer_name;
      content = order.content;
      createdAt = order.created_at;
      orderType = order.order_type;
      charWidth = CHAR_WIDTH[settings.paper_size] ?? 48;
    } catch (err) {
      showError(err);
      goto('/history');
    } finally {
      loading = false;
    }
  });

  async function save() {
    const name = customerName.trim();
    const body = content.trim();
    if (!name) { showToast('Nama pelanggan wajib diisi', 'error'); return; }
    if (!body) { showToast('Isi pesanan wajib diisi', 'error'); return; }

    saving = true;
    try {
      await api.updateOrder(id, name, body);
      showToast('Pesanan berhasil disimpan');
      goto('/history');
    } catch (err) {
      showError(err);
    } finally {
      saving = false;
    }
  }

  async function saveAndReprint() {
    const name = customerName.trim();
    const body = content.trim();
    if (!name) { showToast('Nama pelanggan wajib diisi', 'error'); return; }
    if (!body) { showToast('Isi pesanan wajib diisi', 'error'); return; }

    saving = true;
    try {
      await api.updateOrder(id, name, body);
      await api.reprintOrder(id);
      showToast('Pesanan disimpan dan dicetak ulang');
      goto('/history');
    } catch (err) {
      showError(err);
    } finally {
      saving = false;
    }
  }
</script>

<div class="page">
  <div class="page-header">
    <h2>{orderType === 'receipt' ? 'Edit Tanda Terima' : 'Edit Pesanan'}</h2>
    <a href="/history" class="btn-text">
      <span class="material-symbols-outlined">arrow_back</span>
      Kembali
    </a>
  </div>

  {#if loading}
    <p class="state-msg">Memuat...</p>
  {:else}
    <form onsubmit={(e) => { e.preventDefault(); save(); }}>
      <div class="field">
        <label for="created" class="field-label">Tanggal Dibuat</label>
        <input id="created" class="field-input" type="text" value={createdAt} disabled />
      </div>

      <div class="field">
        <label for="customer" class="field-label">{orderType === 'receipt' ? 'Diterima dari' : 'Nama Pelanggan'}</label>
        <input
          id="customer"
          class="field-input"
          type="text"
          bind:value={customerName}
          disabled={saving}
          autocomplete="off"
        />
      </div>

      <div class="field">
        <label for="content" class="field-label">
          {orderType === 'receipt' ? 'Isi Tanda Terima' : 'Isi Pesanan'}
          <span class="label-hint">— garis biru = batas {charWidth} kolom</span>
        </label>
        <GuidedTextarea id="content"
          bind:value={content}
          {charWidth}
          rows={10}
          disabled={saving}
        />
      </div>

      <div class="actions">
        <button type="submit" class="btn-filled" disabled={saving}>
          <span class="material-symbols-outlined">save</span>
          {saving ? 'Menyimpan...' : 'Simpan'}
        </button>
        <button type="button" class="btn-outlined" onclick={saveAndReprint} disabled={saving}>
          <span class="material-symbols-outlined">print</span>
          Simpan & Cetak Ulang
        </button>
      </div>
    </form>
  {/if}
</div>

<style>
  .page { max-width: 580px; margin: 0 auto; }

  .page-header {
    display: flex; align-items: center; justify-content: space-between;
    margin-bottom: 1.5rem;
  }

  h2 {
    font-size: 1.375rem;
    font-weight: 500;
    color: var(--md-on-surface);
    letter-spacing: .01em;
  }

  .btn-text {
    display: flex; align-items: center; gap: 4px;
    font-size: .875rem; font-weight: 500;
    color: var(--md-secondary);
    text-decoration: none;
    padding: 0 12px; height: 36px;
    border-radius: 18px;
    transition: background .15s;
    font-family: 'Roboto', sans-serif;
  }
  .btn-text:hover { background: var(--md-primary-container); }
  .btn-text .material-symbols-outlined { font-size: 18px; }

  .state-msg { color: var(--md-on-surface-variant); font-size: .9rem; }

  .field {
    display: flex; flex-direction: column; gap: 6px;
    margin-bottom: 1.25rem;
  }

  .field-label {
    font-size: .75rem; font-weight: 500;
    color: var(--md-on-surface-variant);
    letter-spacing: .05em; text-transform: uppercase;
  }

  .label-hint {
    font-weight: 400; text-transform: none; letter-spacing: 0;
    color: var(--md-outline); font-size: .72rem;
  }

  .field-input {
    height: 48px; padding: 0 16px;
    border: 1px solid var(--md-outline-variant);
    border-radius: 4px;
    font-size: .9375rem;
    font-family: 'Roboto', sans-serif;
    color: var(--md-on-surface);
    background: #fff;
    transition: border .15s;
    outline: none; width: 100%;
  }
  .field-input:focus {
    border: 2px solid var(--md-primary);
    padding: 0 15px;
  }
  .field-input:disabled {
    background: var(--md-surface-variant);
    color: var(--md-on-surface-variant);
    cursor: not-allowed;
  }

  .actions { display: flex; gap: 10px; margin-top: .5rem; }

  .btn-filled {
    flex: 1; display: flex; align-items: center; justify-content: center; gap: 8px;
    height: 40px; padding: 0 24px;
    background: var(--md-primary); color: var(--md-on-primary);
    border: none; border-radius: 20px;
    font-size: .875rem; font-weight: 500;
    font-family: 'Roboto', sans-serif;
    cursor: pointer; transition: box-shadow .15s, opacity .15s;
  }
  .btn-filled .material-symbols-outlined { font-size: 18px; }
  .btn-filled:hover:not(:disabled) { box-shadow: var(--md-elev-1); }
  .btn-filled:disabled { opacity: .38; cursor: not-allowed; }

  .btn-outlined {
    flex: 1; display: flex; align-items: center; justify-content: center; gap: 8px;
    height: 40px; padding: 0 24px;
    background: transparent; color: var(--md-primary);
    border: 1px solid var(--md-outline); border-radius: 20px;
    font-size: .875rem; font-weight: 500;
    font-family: 'Roboto', sans-serif;
    cursor: pointer; transition: background .15s;
  }
  .btn-outlined .material-symbols-outlined { font-size: 18px; }
  .btn-outlined:hover:not(:disabled) { background: var(--md-primary-container); }
  .btn-outlined:disabled { opacity: .38; cursor: not-allowed; }
</style>

<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { showToast, showError } from '$lib/stores.svelte';
  import GuidedTextarea from '$lib/GuidedTextarea.svelte';

  const CHAR_WIDTH: Record<string, number> = { '58mm': 32, '75mm': 42, '80mm': 48 };
  const RECEIPT_TEMPLATE = 'Jenis : \nGudang : ';

  type OrderType = 'order' | 'receipt';

  let orderType: OrderType = $state('order');
  let customerName = $state('');
  let content = $state('');
  let isSubmitting = $state(false);
  let charWidth = $state(48);

  onMount(async () => {
    try {
      const s = await api.getSettings();
      charWidth = CHAR_WIDTH[s.paper_size] ?? 48;
    } catch { /* use default */ }
  });

  function selectType(type: OrderType) {
    if (type === orderType) return;
    const wasReceipt = orderType === 'receipt';
    orderType = type;
    if (type === 'receipt' && content.trim() === '') {
      content = RECEIPT_TEMPLATE;
    } else if (type === 'order' && content === RECEIPT_TEMPLATE) {
      content = '';
    }
    // clear name when switching
    if (wasReceipt !== (type === 'receipt')) {
      customerName = '';
    }
  }

  const nameLabel = $derived(orderType === 'receipt' ? 'Diterima dari' : 'Nama Pelanggan');
  const namePlaceholder = $derived(orderType === 'receipt' ? 'Contoh: Toko Maju' : 'Contoh: Pak Budi');
  const submitLabel = $derived(orderType === 'receipt' ? 'Cetak Tanda Terima' : 'Cetak Pesanan');
  const emptyNameMsg = $derived(orderType === 'receipt' ? 'Nama penerima wajib diisi' : 'Nama pelanggan wajib diisi');
  const emptyContentMsg = $derived(orderType === 'receipt' ? 'Isi tanda terima wajib diisi' : 'Isi pesanan wajib diisi');
  const successMsg = $derived(orderType === 'receipt' ? 'Tanda terima berhasil dicetak' : 'Pesanan berhasil dicetak');

  async function submit() {
    const name = customerName.trim();
    const body = content.trim();
    if (!name) { showToast(emptyNameMsg, 'error'); return; }
    if (!body)  { showToast(emptyContentMsg, 'error'); return; }

    isSubmitting = true;
    try {
      const id = await api.createOrder(name, body, orderType);
      await api.printOrder(id);
      showToast(successMsg);
      customerName = '';
      content = orderType === 'receipt' ? RECEIPT_TEMPLATE : '';
    } catch (err) {
      showError(err);
    } finally {
      isSubmitting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      submit();
    }
  }
</script>

<div class="page">
  <h2>{orderType === 'receipt' ? 'Tanda Terima' : 'Pesanan Baru'}</h2>

  <!-- Type selector -->
  <div class="type-chips">
    <button
      type="button"
      class="chip"
      class:chip-selected={orderType === 'order'}
      onclick={() => selectType('order')}
    >
      <span class="material-symbols-outlined">receipt</span>
      Pesanan
    </button>
    <button
      type="button"
      class="chip"
      class:chip-selected={orderType === 'receipt'}
      onclick={() => selectType('receipt')}
    >
      <span class="material-symbols-outlined">assignment</span>
      Tanda Terima
    </button>
  </div>

  <form onsubmit={(e) => { e.preventDefault(); submit(); }}>
    <div class="field">
      <label for="customer" class="field-label">{nameLabel}</label>
      <input
        id="customer"
        class="field-input"
        type="text"
        placeholder={namePlaceholder}
        bind:value={customerName}
        disabled={isSubmitting}
        autocomplete="off"
      />
    </div>

    <div class="field">
      <label for="content" class="field-label">
        {orderType === 'receipt' ? 'Isi Tanda Terima' : 'Isi Pesanan'}
        <span class="label-hint">— garis biru = batas {charWidth} kolom</span>
      </label>
      <GuidedTextarea
        id="content"
        bind:value={content}
        {charWidth}
        placeholder={orderType === 'receipt' ? 'Jenis : ...\nGudang : ...' : 'Tulis atau paste daftar pesanan di sini...'}
        rows={10}
        disabled={isSubmitting}
        onkeydown={handleKeydown}
      />
      <span class="field-support">Ctrl+Enter untuk cetak langsung</span>
    </div>

    <button type="submit" class="btn-filled" disabled={isSubmitting}>
      <span class="material-symbols-outlined">print</span>
      {isSubmitting ? 'Mencetak...' : submitLabel}
    </button>
  </form>
</div>

<style>
  .page { max-width: 580px; margin: 0 auto; }

  h2 {
    font-size: 1.375rem;
    font-weight: 500;
    color: var(--md-on-surface);
    margin-bottom: 1rem;
    letter-spacing: .01em;
  }

  /* ── Type chips ── */
  .type-chips {
    display: flex;
    gap: 8px;
    margin-bottom: 1.25rem;
  }

  .chip {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 36px;
    padding: 0 16px;
    border: 1px solid var(--md-outline-variant);
    border-radius: 18px;
    font-size: .875rem;
    font-weight: 500;
    font-family: 'Roboto', sans-serif;
    color: var(--md-on-surface-variant);
    background: transparent;
    cursor: pointer;
    transition: background .15s, border-color .15s, color .15s;
    user-select: none;
  }
  .chip .material-symbols-outlined { font-size: 18px; }
  .chip:hover { background: var(--md-surface-variant); }
  .chip.chip-selected {
    background: var(--md-primary-container);
    border-color: var(--md-secondary);
    color: var(--md-primary);
  }

  /* ── Fields ── */
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 1.25rem;
  }

  .field-label {
    font-size: .75rem;
    font-weight: 500;
    color: var(--md-on-surface-variant);
    letter-spacing: .05em;
    text-transform: uppercase;
  }

  .label-hint {
    font-weight: 400;
    text-transform: none;
    letter-spacing: 0;
    color: var(--md-outline);
    font-size: .72rem;
  }

  .field-input {
    height: 48px;
    padding: 0 16px;
    border: 1px solid var(--md-outline-variant);
    border-radius: 4px;
    font-size: .9375rem;
    font-family: 'Roboto', sans-serif;
    color: var(--md-on-surface);
    background: #fff;
    transition: border .15s;
    outline: none;
    width: 100%;
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

  .field-support {
    font-size: .72rem;
    color: var(--md-on-surface-variant);
    text-align: right;
  }

  .btn-filled {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: 100%;
    height: 40px;
    padding: 0 24px;
    background: var(--md-primary);
    color: var(--md-on-primary);
    border: none;
    border-radius: 20px;
    font-size: .875rem;
    font-weight: 500;
    font-family: 'Roboto', sans-serif;
    letter-spacing: .01em;
    cursor: pointer;
    transition: box-shadow .15s, opacity .15s;
    margin-top: .5rem;
  }
  .btn-filled .material-symbols-outlined { font-size: 18px; }
  .btn-filled:hover:not(:disabled) { box-shadow: var(--md-elev-1); }
  .btn-filled:active:not(:disabled) { box-shadow: none; opacity: .92; }
  .btn-filled:disabled { opacity: .38; cursor: not-allowed; }
</style>

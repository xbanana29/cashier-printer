<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { showToast, showError } from '$lib/stores.svelte';
  import type { AppSettings, PrinterInfo } from '$lib/types';
  import { check } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { getVersion } from '@tauri-apps/api/app';

  type UpdateStatus = 'idle' | 'checking' | 'up-to-date' | 'available' | 'downloading' | 'error';

  let currentVersion = $state('');
  let updateStatus: UpdateStatus = $state('idle');
  let updateInfo: { version: string; body: string } | null = $state(null);
  let downloadProgress = $state(0);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let pendingUpdate: any = null;

  let settings: AppSettings = $state({
    default_printer: '',
    paper_size: '80mm',
    store_name: '',
    footer_text: '',
    serial_baud_rate: 9600,
    auto_cut: true,
    pc_name: '',
    content_font_size: 'normal',
    extra_feeds: 0,
  });

  let printers: PrinterInfo[] = $state([]);
  let loadingPrinters = $state(false);
  let saving = $state(false);
  let testing = $state(false);

  onMount(async () => {
    currentVersion = await getVersion().catch(() => '—');
    await loadSettings();
    await loadPrinters();
  });

  async function loadSettings() {
    try {
      settings = await api.getSettings();
    } catch (err) {
      showError(err);
    }
  }

  async function loadPrinters() {
    loadingPrinters = true;
    try {
      printers = await api.listPrinters();
    } catch (err) {
      showError(err);
    } finally {
      loadingPrinters = false;
    }
  }

  async function save() {
    saving = true;
    try {
      await api.saveSettings(settings);
      showToast('Pengaturan disimpan');
    } catch (err) {
      showError(err);
    } finally {
      saving = false;
    }
  }

  async function checkUpdate() {
    updateStatus = 'checking';
    updateInfo = null;
    pendingUpdate = null;
    try {
      const update = await check();
      if (update) {
        updateStatus = 'available';
        updateInfo = { version: update.version, body: update.body ?? '' };
        pendingUpdate = update;
      } else {
        updateStatus = 'up-to-date';
      }
    } catch {
      updateStatus = 'error';
    }
  }

  async function installUpdate() {
    if (!pendingUpdate) return;
    updateStatus = 'downloading';
    downloadProgress = 0;
    try {
      let downloaded = 0;
      let total = 0;
      await pendingUpdate.downloadAndInstall((event: { event: string; data?: { contentLength?: number; chunkLength?: number } }) => {
        if (event.event === 'Started') {
          total = event.data?.contentLength ?? 0;
        } else if (event.event === 'Progress') {
          downloaded += event.data?.chunkLength ?? 0;
          if (total > 0) downloadProgress = Math.round((downloaded / total) * 100);
        }
      });
      showToast('Update selesai, aplikasi akan restart...');
      await relaunch();
    } catch (err) {
      showError(err);
      updateStatus = 'available';
    }
  }

  async function testPrint() {
    testing = true;
    try {
      await api.testPrint();
      showToast('Test print berhasil');
    } catch (err) {
      showError(err);
    } finally {
      testing = false;
    }
  }
</script>

<div class="page">
  <h2>Pengaturan</h2>

  <form onsubmit={(e) => { e.preventDefault(); save(); }}>

    <!-- ── Printer section ───────────────────────── -->
    <div class="section">
      <div class="section-header">
        <span class="material-symbols-outlined section-icon">print</span>
        <span class="section-title">Printer</span>
      </div>

      <div class="field">
        <label for="printer" class="field-label">Printer Default</label>
        <div class="row-gap">
          <select id="printer" class="field-select" bind:value={settings.default_printer} disabled={loadingPrinters}>
            <option value="">— Pilih printer —</option>
            {#each printers as p}
              <option value={p.name}>{p.name}{p.is_default ? ' (default sistem)' : ''}</option>
            {/each}
          </select>
          <button type="button" class="icon-btn" onclick={loadPrinters} title="Refresh daftar printer" disabled={loadingPrinters}>
            <span class="material-symbols-outlined">{loadingPrinters ? 'hourglass_empty' : 'refresh'}</span>
          </button>
        </div>
        <p class="field-support">
          Ketik manual: serial <code>/dev/tty.usbserial-1234</code> atau jaringan <code>192.168.1.100:9100</code>
        </p>
        <input
          class="field-input"
          type="text"
          placeholder="Atau ketik nama / alamat manual..."
          bind:value={settings.default_printer}
        />
      </div>

      <div class="field">
        <span class="field-label">Ukuran Kertas</span>
        <div class="chip-group">
          {#each ['58mm', '75mm', '80mm'] as size}
            <label class="chip" class:chip-selected={settings.paper_size === size}>
              <input type="radio" bind:group={settings.paper_size} value={size} />
              {size}
            </label>
          {/each}
        </div>
      </div>

      <div class="field">
        <label for="baud" class="field-label">
          Baud Rate Serial
          <span class="label-optional">— hanya untuk koneksi COM/serial</span>
        </label>
        <select id="baud" class="field-select" bind:value={settings.serial_baud_rate}>
          <option value={9600}>9600 — RPP02 default</option>
          <option value={19200}>19200 — EPSON TM-U220 serial</option>
          <option value={38400}>38400</option>
          <option value={115200}>115200</option>
        </select>
        <p class="field-support">Tidak berpengaruh jika printer terhubung via USB/CUPS atau jaringan.</p>
      </div>

      <div class="field">
        <label class="switch-label">
          <div class="switch" class:switch-on={settings.auto_cut}>
            <input type="checkbox" bind:checked={settings.auto_cut} />
            <span class="switch-thumb"></span>
          </div>
          <div class="switch-text">
            <span class="switch-name">Pemotong kertas otomatis (auto-cut)</span>
            <span class="switch-desc">
              Nonaktifkan untuk TM-U220 <strong>tanpa</strong> cutter. TM-U220 dengan cutter &amp; TM-T82X: aktifkan.
            </span>
          </div>
        </label>
      </div>

      <div class="field">
        <label for="extra-feeds" class="field-label">
          Baris Tambahan Setelah Cetak
          <span class="label-optional">— untuk mendorong kertas keluar</span>
        </label>
        <select id="extra-feeds" class="field-select field-select-short" bind:value={settings.extra_feeds}>
          <option value={0}>0 — tidak ada tambahan</option>
          <option value={1}>1 baris</option>
          <option value={2}>2 baris</option>
          <option value={3}>3 baris</option>
          <option value={4}>4 baris</option>
          <option value={5}>5 baris</option>
        </select>
        <p class="field-support">
          Tambahkan baris kosong agar tulisan terakhir keluar dari kepala cetak.
          Berguna untuk TM-U220 dan printer non-standar yang tidak otomatis mengeluarkan kertas.
        </p>
      </div>
    </div>

    <!-- ── Template section ──────────────────────── -->
    <div class="section">
      <div class="section-header">
        <span class="material-symbols-outlined section-icon">receipt_long</span>
        <span class="section-title">Template Cetak</span>
      </div>

      <div class="field">
        <span class="field-label">Ukuran Font Isi Pesanan</span>
        <div class="chip-group">
          {#each [
            { value: 'normal', label: 'Normal',    desc: '1×' },
            { value: 'tall',   label: 'Tinggi 2×', desc: 'tinggi' },
            { value: 'wide',   label: 'Lebar 2×',  desc: 'lebar' },
            { value: 'large',  label: 'Besar 2×2', desc: 'lebar+tinggi' },
          ] as opt}
            <label class="chip" class:chip-selected={settings.content_font_size === opt.value} title={opt.desc}>
              <input type="radio" bind:group={settings.content_font_size} value={opt.value} />
              {opt.label}
            </label>
          {/each}
        </div>
        <p class="field-support">
          Mengatur ukuran font baris item pesanan. Garis putus-putus menyesuaikan otomatis.<br>
          <strong>Lebar 2×</strong> / <strong>Besar 2×2</strong>: lebar karakter ganda — kolom teks berkurang separuh.
        </p>
      </div>

      <div class="field">
        <label for="store" class="field-label">Nama Toko <span class="label-optional">(opsional)</span></label>
        <input id="store" class="field-input" type="text" placeholder="Contoh: Toko Makmur Jaya" bind:value={settings.store_name} />
      </div>

      <div class="field">
        <label for="footer" class="field-label">Footer <span class="label-optional">(opsional)</span></label>
        <input id="footer" class="field-input" type="text" placeholder="Contoh: Terima kasih atas pesanan Anda!" bind:value={settings.footer_text} />
      </div>

      <div class="field">
        <label for="pcname" class="field-label">Nama PC / Kasir</label>
        <input id="pcname" class="field-input" type="text" placeholder="Contoh: Kasir 1" bind:value={settings.pc_name} />
        <p class="field-support">Muncul di baris bawah struk dan di kolom riwayat pesanan.</p>
      </div>
    </div>

    <!-- ── Tentang & Update ─────────────────────── -->
    <div class="section">
      <div class="section-header">
        <span class="material-symbols-outlined section-icon">system_update</span>
        <span class="section-title">Tentang & Update</span>
      </div>

      <div class="about-row">
        <span class="about-label">Versi saat ini</span>
        <span class="about-value">v{currentVersion}</span>
      </div>

      {#if updateStatus === 'available' && updateInfo}
        <div class="update-banner">
          <span class="material-symbols-outlined update-icon">new_releases</span>
          <div class="update-info">
            <span class="update-title">Update tersedia: v{updateInfo.version}</span>
            {#if updateInfo.body}
              <span class="update-notes">{updateInfo.body}</span>
            {/if}
          </div>
        </div>
        <button type="button" class="btn-update" onclick={installUpdate} disabled={updateStatus === 'downloading'}>
          <span class="material-symbols-outlined">download</span>
          Download & Pasang v{updateInfo.version}
        </button>
      {:else if updateStatus === 'downloading'}
        <div class="progress-row">
          <div class="progress-bar">
            <div class="progress-fill" style="width: {downloadProgress}%"></div>
          </div>
          <span class="progress-label">{downloadProgress}%</span>
        </div>
      {:else if updateStatus === 'up-to-date'}
        <p class="update-msg update-ok">
          <span class="material-symbols-outlined">check_circle</span>
          Aplikasi sudah versi terbaru
        </p>
      {:else if updateStatus === 'error'}
        <p class="update-msg update-err">
          <span class="material-symbols-outlined">error</span>
          Gagal cek update. Pastikan koneksi internet aktif.
        </p>
      {/if}

      <div class="field" style="margin-top: 8px; margin-bottom: 0">
        <button
          type="button"
          class="btn-outlined"
          onclick={checkUpdate}
          disabled={updateStatus === 'checking' || updateStatus === 'downloading'}
        >
          <span class="material-symbols-outlined">
            {updateStatus === 'checking' ? 'hourglass_empty' : 'sync'}
          </span>
          {updateStatus === 'checking' ? 'Mengecek...' : 'Cek Update'}
        </button>
      </div>
    </div>

    <!-- ── Actions ───────────────────────────────── -->
    <div class="actions">
      <button type="submit" class="btn-filled" disabled={saving}>
        <span class="material-symbols-outlined">save</span>
        {saving ? 'Menyimpan...' : 'Simpan Pengaturan'}
      </button>
      <button type="button" class="btn-outlined" onclick={testPrint} disabled={testing}>
        <span class="material-symbols-outlined">print</span>
        {testing ? 'Mencetak...' : 'Test Print'}
      </button>
    </div>

  </form>
</div>

<style>
  .page { max-width: 560px; margin: 0 auto; }

  h2 {
    font-size: 1.375rem; font-weight: 500;
    color: var(--md-on-surface);
    letter-spacing: .01em; margin-bottom: 1.5rem;
  }

  /* ── Section card ── */
  .section {
    background: #fff;
    border-radius: 12px;
    padding: 4px 16px 12px;
    margin-bottom: 12px;
    box-shadow: var(--md-elev-1);
  }

  .section-header {
    display: flex; align-items: center; gap: 10px;
    padding: 12px 0 10px;
    border-bottom: 1px solid var(--md-outline-variant);
    margin-bottom: 12px;
  }
  .section-icon { font-size: 20px; color: var(--md-secondary); }
  .section-title {
    font-size: .8125rem; font-weight: 600;
    color: var(--md-on-surface-variant);
    text-transform: uppercase; letter-spacing: .07em;
  }

  /* ── Fields ── */
  .field { margin-bottom: 14px; display: flex; flex-direction: column; gap: 6px; }

  .field-label {
    font-size: .75rem; font-weight: 500;
    color: var(--md-on-surface-variant);
    text-transform: uppercase; letter-spacing: .05em;
  }
  .label-optional {
    font-weight: 400; text-transform: none; letter-spacing: 0;
    color: var(--md-outline); font-size: .72rem;
  }

  .field-input {
    height: 48px; padding: 0 16px;
    border: 1px solid var(--md-outline-variant); border-radius: 4px;
    font-size: .9375rem; font-family: 'Roboto', sans-serif;
    color: var(--md-on-surface); background: #fff;
    transition: border .15s; outline: none; width: 100%;
  }
  .field-input:focus { border: 2px solid var(--md-primary); padding: 0 15px; }
  .field-input:disabled {
    background: var(--md-surface-variant);
    color: var(--md-on-surface-variant); cursor: not-allowed;
  }

  .field-select {
    height: 48px; padding: 0 12px;
    border: 1px solid var(--md-outline-variant); border-radius: 4px;
    font-size: .9375rem; font-family: 'Roboto', sans-serif;
    color: var(--md-on-surface); background: #fff;
    transition: border .15s; outline: none; width: 100%; cursor: pointer;
  }
  .field-select:focus { border: 2px solid var(--md-primary); }
  .field-select:disabled { background: var(--md-surface-variant); cursor: not-allowed; }
  .field-select-short { max-width: 240px; }

  .field-support { font-size: .72rem; color: var(--md-on-surface-variant); line-height: 1.5; }
  code { background: var(--md-surface-variant); padding: .1em .3em; border-radius: 3px; font-size: .82em; }

  .row-gap { display: flex; gap: 8px; }
  .row-gap .field-select { flex: 1; }

  /* ── Icon button ── */
  .icon-btn {
    width: 48px; height: 48px; border-radius: 4px;
    border: 1px solid var(--md-outline-variant);
    background: #fff; display: flex; align-items: center; justify-content: center;
    cursor: pointer; color: var(--md-on-surface-variant);
    flex-shrink: 0; transition: background .15s;
  }
  .icon-btn:hover:not(:disabled) { background: var(--md-surface-variant); }
  .icon-btn:disabled { opacity: .38; cursor: not-allowed; }
  .icon-btn .material-symbols-outlined { font-size: 20px; }

  /* ── Filter chips for paper size ── */
  .chip-group { display: flex; gap: 8px; flex-wrap: wrap; }

  .chip {
    display: flex; align-items: center;
    height: 32px; padding: 0 16px;
    border: 1px solid var(--md-outline-variant);
    border-radius: 8px;
    font-size: .875rem; font-weight: 500;
    color: var(--md-on-surface-variant);
    cursor: pointer;
    transition: background .15s, border-color .15s, color .15s;
    user-select: none;
  }
  .chip input[type="radio"] { display: none; }
  .chip:hover { background: var(--md-surface-variant); }
  .chip.chip-selected {
    background: var(--md-primary-container);
    border-color: var(--md-secondary);
    color: var(--md-primary);
    font-weight: 600;
  }

  /* ── MD3 Switch ── */
  .switch-label {
    display: flex; align-items: flex-start; gap: 12px;
    cursor: pointer; padding: 4px 0;
  }

  .switch {
    position: relative; flex-shrink: 0;
    width: 52px; height: 32px;
    border-radius: 16px;
    border: 2px solid var(--md-outline);
    background: transparent;
    transition: background .2s, border-color .2s;
    cursor: pointer;
  }
  .switch input { display: none; }
  .switch-on {
    background: var(--md-primary);
    border-color: var(--md-primary);
  }

  .switch-thumb {
    position: absolute; top: 3px; left: 3px;
    width: 22px; height: 22px;
    border-radius: 11px;
    background: var(--md-outline);
    transition: transform .2s, background .2s;
  }
  .switch-on .switch-thumb {
    transform: translateX(20px);
    background: var(--md-on-primary);
  }

  .switch-text { display: flex; flex-direction: column; gap: 2px; }
  .switch-name { font-size: .9375rem; font-weight: 500; color: var(--md-on-surface); }
  .switch-desc { font-size: .75rem; color: var(--md-on-surface-variant); line-height: 1.5; }
  .switch-desc strong { color: var(--md-on-surface); }

  /* ── About / Update ── */
  .about-row {
    display: flex; align-items: center; justify-content: space-between;
    padding: 6px 0 10px; margin-bottom: 8px;
    border-bottom: 1px solid var(--md-outline-variant);
  }
  .about-label { font-size: .875rem; color: var(--md-on-surface-variant); }
  .about-value { font-size: .875rem; font-weight: 600; color: var(--md-on-surface); }

  .update-banner {
    display: flex; align-items: flex-start; gap: 10px;
    background: var(--md-primary-container);
    border-radius: 8px; padding: 10px 12px; margin-bottom: 10px;
  }
  .update-icon { font-size: 20px; color: var(--md-primary); flex-shrink: 0; }
  .update-info { display: flex; flex-direction: column; gap: 2px; }
  .update-title { font-size: .875rem; font-weight: 600; color: var(--md-primary); }
  .update-notes { font-size: .75rem; color: var(--md-on-surface-variant); white-space: pre-wrap; }

  .update-msg {
    display: flex; align-items: center; gap: 6px;
    font-size: .875rem; padding: 6px 0; margin-bottom: 4px;
  }
  .update-msg .material-symbols-outlined { font-size: 18px; }
  .update-ok { color: #1a7a4a; }
  .update-err { color: var(--md-error, #b00020); }

  .btn-update {
    display: flex; align-items: center; justify-content: center; gap: 8px;
    width: 100%; height: 40px; padding: 0 24px; margin-bottom: 10px;
    background: var(--md-primary); color: var(--md-on-primary);
    border: none; border-radius: 20px;
    font-size: .875rem; font-weight: 500; font-family: 'Roboto', sans-serif;
    cursor: pointer; transition: box-shadow .15s;
  }
  .btn-update .material-symbols-outlined { font-size: 18px; }
  .btn-update:hover:not(:disabled) { box-shadow: var(--md-elev-1); }
  .btn-update:disabled { opacity: .38; cursor: not-allowed; }

  .progress-row { display: flex; align-items: center; gap: 10px; margin-bottom: 10px; }
  .progress-bar { flex: 1; height: 6px; border-radius: 3px; background: var(--md-outline-variant); overflow: hidden; }
  .progress-fill { height: 100%; background: var(--md-primary); border-radius: 3px; transition: width .2s; }
  .progress-label { font-size: .75rem; color: var(--md-on-surface-variant); width: 32px; text-align: right; }

  /* ── Actions ── */
  .actions { display: flex; gap: 10px; margin-top: 4px; }

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

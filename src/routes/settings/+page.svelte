<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { showToast, showError } from '$lib/stores.svelte';
  import type { AppSettings, PrinterInfo } from '$lib/types';
  import { check } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { getVersion } from '@tauri-apps/api/app';
  import type { PeerInfo } from '$lib/types';

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
  let serialPorts: string[] = $state([]);
  let loadingPrinters = $state(false);
  let saving = $state(false);
  let testing = $state(false);

  type ConnType = 'os' | 'serial' | 'network';

  // Derive connection type from stored printer value
  function detectConnType(val: string): ConnType {
    if (/^(\/dev\/tty|\/dev\/cu\.|COM\d)/i.test(val)) return 'serial';
    if (/^\d{1,3}(\.\d{1,3}){3}(:\d+)?$/.test(val)) return 'network';
    return 'os';
  }

  let connType: ConnType = $state('os');

  // LAN sync
  let peers: PeerInfo[] = $state([]);
  let syncing = $state(false);
  let deviceId = $state('');

  onMount(async () => {
    currentVersion = await getVersion().catch(() => '—');
    await loadSettings();
    await loadPrinters();
    await refreshPeers();
    // Poll peers every 5s while settings page is open
    const peerTimer = setInterval(refreshPeers, 5000);
    return () => clearInterval(peerTimer);
  });

  async function loadSettings() {
    try {
      settings = await api.getSettings();
      connType = detectConnType(settings.default_printer);
    } catch (err) {
      showError(err);
    }
  }

  async function refreshPeers() {
    try {
      peers = await api.getPeers();
    } catch { /* ignore — sync may not be ready yet */ }
    // Grab device_id from settings DB via a simple getSettings call (reuse existing data)
    if (!deviceId && settings.pc_name) {
      deviceId = settings.pc_name; // fallback display until we expose device_id
    }
  }

  async function syncNow() {
    syncing = true;
    try {
      const count = await api.syncNow();
      showToast(count > 0 ? `${count} pesanan berhasil disinkron` : 'Tidak ada pesanan baru');
      await refreshPeers();
    } catch (err) {
      showError(err);
    } finally {
      syncing = false;
    }
  }

  function formatLastSeen(ts: number): string {
    if (!ts) return '—';
    const diff = Math.floor(Date.now() / 1000) - ts;
    if (diff < 60) return `${diff}d lalu`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m lalu`;
    return `${Math.floor(diff / 3600)}j lalu`;
  }

  async function loadPrinters() {
    loadingPrinters = true;
    try {
      [printers, serialPorts] = await Promise.all([
        api.listPrinters(),
        api.listSerialPorts(),
      ]);
    } catch (err) {
      showError(err);
    } finally {
      loadingPrinters = false;
    }
  }

  function switchConnType(type: ConnType) {
    connType = type;
    // Reset default_printer when switching type to avoid stale values
    settings.default_printer = '';
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
      // Endpoint belum ada (belum ada release) atau jaringan bermasalah —
      // tampilkan "sudah terbaru" agar tidak membingungkan pengguna.
      updateStatus = 'up-to-date';
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

      <!-- Connection type selector -->
      <div class="field">
        <span class="field-label">Jenis Koneksi</span>
        <div class="conn-tabs">
          <button
            type="button"
            class="conn-tab"
            class:conn-tab-active={connType === 'os'}
            onclick={() => switchConnType('os')}
          >
            <span class="material-symbols-outlined">usb</span>
            USB / CUPS
          </button>
          <button
            type="button"
            class="conn-tab"
            class:conn-tab-active={connType === 'serial'}
            onclick={() => switchConnType('serial')}
          >
            <span class="material-symbols-outlined">cable</span>
            Serial / COM
          </button>
          <button
            type="button"
            class="conn-tab"
            class:conn-tab-active={connType === 'network'}
            onclick={() => switchConnType('network')}
          >
            <span class="material-symbols-outlined">wifi</span>
            Jaringan (LAN)
          </button>
        </div>
      </div>

      <!-- OS/USB -->
      {#if connType === 'os'}
        <div class="field">
          <label for="printer" class="field-label">Pilih Printer</label>
          <div class="row-gap">
            <select id="printer" class="field-select" bind:value={settings.default_printer} disabled={loadingPrinters}>
              <option value="">— Pilih printer —</option>
              {#each printers as p}
                <option value={p.name}>{p.name}{p.is_default ? ' ✓' : ''}</option>
              {/each}
            </select>
            <button type="button" class="icon-btn" onclick={loadPrinters} title="Refresh" disabled={loadingPrinters}>
              <span class="material-symbols-outlined">{loadingPrinters ? 'hourglass_empty' : 'refresh'}</span>
            </button>
          </div>
          <p class="field-support">Printer yang terdeteksi di sistem (USB, Bluetooth, jaringan via CUPS).</p>
        </div>

      <!-- Serial/COM -->
      {:else if connType === 'serial'}
        <div class="field">
          <label for="serial-port" class="field-label">Port Serial</label>
          <div class="row-gap">
            <select id="serial-port" class="field-select" bind:value={settings.default_printer} disabled={loadingPrinters}>
              <option value="">— Pilih port —</option>
              {#each serialPorts as port}
                <option value={port}>{port}</option>
              {/each}
            </select>
            <button type="button" class="icon-btn" onclick={loadPrinters} title="Refresh" disabled={loadingPrinters}>
              <span class="material-symbols-outlined">{loadingPrinters ? 'hourglass_empty' : 'refresh'}</span>
            </button>
          </div>
          <p class="field-support">
            Hubungkan printer via kabel USB serial / RS-232. Pastikan kabel terpasang sebelum refresh.
          </p>
        </div>
        <div class="field">
          <label for="baud" class="field-label">Kecepatan (Baud Rate)</label>
          <select id="baud" class="field-select field-select-short" bind:value={settings.serial_baud_rate}>
            <option value={9600}>9600 — RPP02</option>
            <option value={19200}>19200 — EPSON TM-U220</option>
            <option value={38400}>38400</option>
            <option value={115200}>115200</option>
          </select>
        </div>

      <!-- Network/LAN -->
      {:else}
        <div class="field">
          <label for="net-addr" class="field-label">Alamat IP Printer</label>
          <input
            id="net-addr"
            class="field-input"
            type="text"
            placeholder="Contoh: 192.168.1.100:9100"
            bind:value={settings.default_printer}
          />
          <p class="field-support">Format: <code>IP:port</code>. Port default printer jaringan biasanya <code>9100</code>.</p>
        </div>
      {/if}

      <!-- Paper size — always visible -->
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

      <!-- Auto-cut -->
      <div class="field">
        <label class="switch-label">
          <div class="switch" class:switch-on={settings.auto_cut}>
            <input type="checkbox" bind:checked={settings.auto_cut} />
            <span class="switch-thumb"></span>
          </div>
          <div class="switch-text">
            <span class="switch-name">Potong kertas otomatis</span>
            <span class="switch-desc">Matikan untuk TM-U220 tanpa cutter.</span>
          </div>
        </label>
      </div>

      <!-- Extra feeds -->
      <div class="field">
        <label for="extra-feeds" class="field-label">Baris Kosong Setelah Cetak</label>
        <select id="extra-feeds" class="field-select field-select-short" bind:value={settings.extra_feeds}>
          <option value={0}>0</option>
          <option value={1}>1</option>
          <option value={2}>2</option>
          <option value={3}>3</option>
          <option value={4}>4</option>
          <option value={5}>5</option>
        </select>
        <p class="field-support">Tambah jika tulisan terakhir tidak keluar dari kepala cetak.</p>
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
            { value: 'normal', label: 'Normal',      sub: '1× — standar' },
            { value: 'tall',   label: 'Tinggi',      sub: '2× tinggi' },
            { value: 'wide',   label: 'Lebar',       sub: '2× lebar, ½ kolom' },
            { value: 'large',  label: 'Besar',       sub: '2× tinggi + lebar' },
          ] as opt}
            <label class="chip font-chip" class:chip-selected={settings.content_font_size === opt.value}>
              <input type="radio" bind:group={settings.content_font_size} value={opt.value} />
              <span class="chip-label">{opt.label}</span>
              <span class="chip-sub">{opt.sub}</span>
            </label>
          {/each}
        </div>
        <p class="field-support">
          Printer ESC/POS raw hanya mendukung 4 ukuran fisik ini.
          <strong>Lebar</strong> dan <strong>Besar</strong> mengurangi kolom teks separuh — baris item lebih sedikit per baris.
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

    <!-- ── Sinkronisasi LAN ──────────────────────── -->
    <div class="section">
      <div class="section-header">
        <span class="material-symbols-outlined section-icon">lan</span>
        <span class="section-title">Sinkronisasi LAN</span>
      </div>

      <p class="field-support" style="margin-bottom: 12px">
        Sinkron otomatis antar PC dalam satu jaringan WiFi/LAN. Tidak perlu internet.
      </p>

      <div class="field">
        <span class="field-label">Perangkat Terhubung</span>
        {#if peers.length === 0}
          <p class="peer-empty">Belum ada perangkat lain terdeteksi di jaringan ini.</p>
        {:else}
          <div class="peer-list">
            {#each peers as peer (peer.device_id)}
              <div class="peer-row">
                <span class="material-symbols-outlined peer-icon">computer</span>
                <div class="peer-info">
                  <span class="peer-name">{peer.pc_name || 'Tanpa nama'}</span>
                  <span class="peer-addr">{peer.addr}</span>
                </div>
                <div class="peer-meta">
                  {#if peer.orders_synced > 0}
                    <span class="peer-badge">{peer.orders_synced} order</span>
                  {/if}
                  <span class="peer-seen">{formatLastSeen(peer.last_seen)}</span>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div class="field" style="margin-bottom: 0">
        <button type="button" class="btn-outlined" onclick={syncNow} disabled={syncing}>
          <span class="material-symbols-outlined">{syncing ? 'hourglass_empty' : 'sync'}</span>
          {syncing ? 'Menyinkron...' : 'Sync Sekarang'}
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

  /* ── Connection type tabs ── */
  .conn-tabs {
    display: flex; gap: 0;
    border: 1px solid var(--md-outline-variant);
    border-radius: 8px; overflow: hidden;
  }

  .conn-tab {
    flex: 1; display: flex; align-items: center; justify-content: center; gap: 6px;
    height: 44px; padding: 0 12px;
    background: #fff; color: var(--md-on-surface-variant);
    border: none; border-right: 1px solid var(--md-outline-variant);
    font-size: .8125rem; font-weight: 500; font-family: 'Roboto', sans-serif;
    cursor: pointer; transition: background .15s, color .15s;
  }
  .conn-tab:last-child { border-right: none; }
  .conn-tab .material-symbols-outlined { font-size: 18px; }
  .conn-tab:hover:not(.conn-tab-active) { background: var(--md-surface-variant); }

  .conn-tab-active {
    background: var(--md-primary-container);
    color: var(--md-primary);
    font-weight: 600;
  }

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

  /* ── Font chip sub-label ── */
  .font-chip { flex-direction: column; height: auto; padding: 6px 14px; gap: 2px; align-items: center; }
  .chip-label { font-size: .875rem; font-weight: 500; }
  .chip-sub   { font-size: .65rem; color: var(--md-on-surface-variant); font-weight: 400; }
  .font-chip.chip-selected .chip-sub { color: var(--md-primary); opacity: .85; }

  /* ── LAN Sync peers ── */
  .peer-empty {
    font-size: .8125rem;
    color: var(--md-on-surface-variant);
    font-style: italic;
    padding: 4px 0;
  }

  .peer-list { display: flex; flex-direction: column; gap: 6px; }

  .peer-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    background: var(--md-surface-container);
    border-radius: 8px;
  }

  .peer-icon { font-size: 20px; color: var(--md-secondary); flex-shrink: 0; }

  .peer-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
  .peer-name { font-size: .875rem; font-weight: 500; color: var(--md-on-surface); }
  .peer-addr { font-size: .72rem; color: var(--md-on-surface-variant); font-family: monospace; }

  .peer-meta { display: flex; flex-direction: column; align-items: flex-end; gap: 2px; flex-shrink: 0; }
  .peer-badge {
    font-size: .68rem; font-weight: 600;
    color: var(--md-on-primary); background: var(--md-primary);
    border-radius: 4px; padding: 1px 6px;
  }
  .peer-seen { font-size: .68rem; color: var(--md-on-surface-variant); }

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

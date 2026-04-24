# Print Paste Order (PPO)

Aplikasi desktop ringan untuk kasir toko kecil — input pesanan, cetak struk termal, simpan riwayat.  
Dibuat dengan **Tauri 2 · Svelte 5 · Rust · SQLite** — berjalan 100% offline, tanpa server, tanpa internet.

---

## Download

Unduh versi terbaru untuk platform Anda di halaman [**Releases**](https://github.com/nikokevin29/cashier-printer/releases/latest):

| Platform | File | Keterangan |
|---|---|---|
| Windows 64-bit | `*_x64-setup.exe` | Setup installer NSIS, tidak perlu hak admin |
| macOS Intel | `*_x64.dmg` | macOS 10.15 (Catalina) ke atas |
| macOS Apple Silicon | `*_aarch64.dmg` | M1 / M2 / M3 |
| Linux 64-bit | `*.AppImage` | Portable — tidak perlu install |

### Cara Install

**Windows**
```
Jalankan .exe → Next → Next → Finish
```

**macOS**
```
Buka .dmg → Drag "Print Paste Order" ke folder Applications
```

**Linux**
```bash
chmod +x Print.Paste.Order_*.AppImage
./Print.Paste.Order_*.AppImage
```

---

## Fitur

| Fitur | Keterangan |
|---|---|
| Pesanan baru | Input nama pelanggan + isi pesanan, cetak dengan satu klik atau Ctrl+Enter |
| Riwayat pesanan | Cari, lihat preview, edit, cetak ulang, hapus — paginasi 25 per halaman |
| Edit pesanan | Ubah nama atau isi, simpan saja atau simpan & cetak ulang |
| Pengaturan | Pilih printer, ukuran kertas, nama toko, footer, nama kasir, auto-cut |
| Test print | Cetak halaman uji coba langsung dari menu pengaturan |
| Retensi data | Riwayat disimpan selama 1 tahun, dihapus otomatis saat aplikasi dibuka |

---

## Printer yang Didukung

| Printer | Ukuran | Koneksi |
|---|---|---|
| RPP02 | 58 mm | USB / Serial (9600 baud default) |
| EPSON TM-U220 | 75 mm | USB / Serial (19200 baud) — ada yang tanpa cutter |
| EPSON TM-T82X | 80 mm | USB / LAN (IP:9100) |
| Printer ESC/POS lainnya | 58 / 75 / 80 mm | USB (CUPS) / Serial / Jaringan |

### Konfigurasi Koneksi

Isi kolom **Printer Default** di halaman Pengaturan sesuai jenis koneksi:

| Jenis | Contoh nilai |
|---|---|
| USB via CUPS (macOS/Linux) | `EPSON_TM-T82X` *(nama dari daftar printer)* |
| USB via WinSpool (Windows) | `EPSON TM-T82X` *(nama dari daftar printer)* |
| Serial / COM (Windows) | `COM3` |
| Serial / tty (macOS/Linux) | `/dev/tty.usbserial-1234` |
| Jaringan (LAN) | `192.168.1.100:9100` |

> **TM-U220 tanpa cutter** — matikan toggle **Auto-cut** di Pengaturan.

---

## Tampilan

```
  DK PASAR                     ← nama pelanggan (cetak tebal, tinggi 2×)
  Tanggal  : 2026-04-24 10:32

2 sak beras........................ [ ]
1 sak terigu....................... [ ]
40 kg minyak goreng kemasan besar
ekonomis........................... [ ]
5 karton teh botol sosro........... [ ]

          Terima kasih
     CV REJEKI AMERTA JAYA
PC: Kasir 1
```

- Titik-titik (dot leaders) memudahkan pencoretan manual
- Kotak `[ ]` untuk centang saat barang disiapkan
- Nama pelanggan dicetak dua kali lebih besar untuk mudah dibaca

---

## Pengaturan Detail

### Ukuran Kertas

| Pilihan | Lebar cetak | Printer |
|---|---|---|
| 58 mm | 32 karakter | RPP02 |
| 75 mm | 42 karakter | EPSON TM-U220 |
| 80 mm | 48 karakter | EPSON TM-T82X *(default)* |

### Template Cetak

| Field | Keterangan |
|---|---|
| Nama Toko | Muncul di bawah daftar item, teks terpusat |
| Footer | Baris tepat di atas nama toko (misal: "Terima kasih atas pesanan Anda!") |
| Nama PC / Kasir | Muncul di baris paling bawah struk dan di kolom riwayat |

---

## Build dari Source

### Prasyarat

- [Bun](https://bun.sh) ≥ 1.0
- [Rust](https://rustup.rs) stable
- Tauri v2 system dependencies — lihat [Prerequisites Tauri](https://v2.tauri.app/start/prerequisites/)

### Langkah

```bash
# Clone repositori
git clone https://github.com/nikokevin29/cashier-printer.git
cd cashier-printer

# Install dependensi frontend
bun install

# Jalankan mode development
bun run tauri dev

# Build production (platform saat ini)
bun run tauri build
```

Output binary ada di `src-tauri/target/release/bundle/`.

### Menjalankan Unit Test

```bash
cd src-tauri
cargo test
```

49 unit test mencakup: CRUD database, pengaturan, builder ESC/POS, dan dispatch printer.

---

## Struktur Proyek

```
cashier-printer/
├── src/
│   ├── lib/
│   │   ├── api.ts               # Wrapper typed untuk invoke() Tauri
│   │   ├── stores.svelte.ts     # Toast & state global (Svelte 5 runes)
│   │   ├── types.ts             # Order, AppSettings, PrinterInfo
│   │   └── GuidedTextarea.svelte # Textarea dengan garis batas kolom
│   └── routes/
│       ├── +layout.svelte       # Navigation Rail MD3 + toast overlay
│       ├── new/                 # Form pesanan baru
│       ├── history/             # Riwayat + search + pagination
│       ├── edit/[id]/           # Edit pesanan
│       └── settings/            # Pengaturan printer & template
└── src-tauri/src/
    ├── db/
    │   ├── mod.rs               # Inisialisasi SQLite + schema
    │   ├── orders.rs            # CRUD pesanan
    │   └── settings.rs          # Key-value settings
    ├── print/
    │   ├── builder.rs           # VecDriver + layout struk ESC/POS
    │   └── driver.rs            # Dispatch: CUPS → Serial → Jaringan
    ├── commands/                # Tauri commands (order, print, settings)
    ├── error.rs                 # AppError dengan serde + thiserror
    └── lib.rs                   # Tauri builder + state init
```

---

## CI / CD

Setiap tag `v*` yang di-push ke `main` akan otomatis men-trigger GitHub Actions untuk:

1. Build untuk 4 platform (macOS Intel, macOS ARM, Windows x64, Linux x64)
2. Membuat GitHub Release dengan file siap unduh

Lihat [`.github/workflows/release.yml`](.github/workflows/release.yml).

---

## Lisensi

Copyright © 2026 [CV Rejeki Amerta Jaya](https://rejekiamerta.com). All rights reserved.

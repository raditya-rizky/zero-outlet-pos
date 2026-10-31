# Kasir Zero Outlet

Aplikasi kasir (Point of Sale) berbasis web untuk UMKM pakaian, dengan studi kasus Zero Outlet, Seririt, Buleleng.
Capstone Project STSI4440 Kelompok D, Universitas Terbuka.

- **Backend:** Rust + Axum (REST API), SQLx, PostgreSQL
- **Frontend:** Vue 3 + Vite (SPA), Pinia, Vue Router, Tailwind CSS v4
- **Basis data:** PostgreSQL 17 (lewat Docker)

Backend juga menyajikan hasil build frontend, sehingga di komputer toko hanya perlu satu program server dan satu basis data.

## Status: siklus prototipe 1

| Kebutuhan | Fitur | Status |
| --- | --- | --- |
| KF-01 | Login/logout, sesi berakhir otomatis bila tidak aktif | Selesai |
| KF-02 | Kelola akun dan peran | Selesai |
| KF-03, KF-04 | Produk, varian ukuran-warna, SKU/barcode, stok per varian | Selesai |
| KF-05 | Pencarian dan pemindaian barcode | Selesai |
| KF-06, KF-08 | Keranjang dengan validasi stok, pembayaran tunai/transfer/QRIS | Selesai |
| KF-09 | Struk 58 mm/PDF, riwayat, cetak ulang | Selesai |
| KF-19 | Sesi kasir dan rekonsiliasi kas | Selesai |
| KF-20 | Log aktivitas (pencatatan dan tampilan) | Selesai |
| KF-07, KF-10–KF-14 | Diskon, retur, barang masuk, stok opname, kartu stok, peringatan | Siklus 2 (tabel sudah disiapkan) |
| KF-15–KF-18 | Laporan penjualan, ekspor, biaya, laba rugi | Siklus 3 (tabel sudah disiapkan) |

## Menjalankan untuk pengembangan

Prasyarat: Docker, Rust (stable), Node.js 20.19 atau 22.12 ke atas, dan pnpm 10.

```bash
# 1. Basis data
docker compose up -d

# 2. Konfigurasi backend
cp .env.example backend/.env

# 3. Data contoh (25 produk, 183 varian, akun admin dan kasir)
cd backend && cargo run --bin seed

# 4. Server API di port 3000 (migrasi berjalan otomatis)
cargo run

# 5. Di terminal lain: frontend dengan hot reload di port 5173
cd frontend && pnpm install && pnpm dev
```

Buka http://localhost:5173. Akun bawaan dari data contoh:

| Peran | Nama pengguna | Kata sandi |
| --- | --- | --- |
| Pemilik/Admin | `admin` | `admin12345` |
| Kasir | `kasir1`, `kasir2` | `kasir12345` |

Ganti kata sandi ini sebelum dipakai di toko.

## Pengujian

```bash
cd backend
cargo test                       # uji unit + uji integrasi (butuh Postgres dari docker compose)
cargo clippy --all-targets -- -D warnings
cd ../frontend && pnpm typecheck
```

Uji integrasi di `backend/tests/api.rs` mengikuti skenario uji Tabel 25:
- UJ-01: login dengan kata sandi salah
- UJ-02: kasir membuka fitur admin
- UJ-03: transaksi tanpa sesi aktif
- UJ-04: stok tidak cukup, dan seluruh transaksi dibatalkan
- UJ-06: cetak ulang struk
- UJ-09: selisih kas wajib disertai alasan
- UJ-10: perubahan harga tercatat di log

Ada juga uji dua kasir yang menjual unit terakhir secara bersamaan. Setiap uji memakai basis data sementara yang dibuat otomatis oleh `#[sqlx::test]`.

## Pemasangan di komputer toko (mode lokal)

1. Pasang Docker, lalu jalankan `docker compose up -d`.
2. Build frontend: `cd frontend && pnpm install --frozen-lockfile && pnpm build`.
3. Build backend: `cd backend && cargo build --release`.
4. Salin `.env.example` ke `backend/.env`. Ubah `ADMIN_PASSWORD`, dan pastikan `FRONTEND_DIST=../frontend/dist`.
5. Jalankan `./target/release/zero-outlet-pos` dari folder `backend`.
6. Perangkat lain di jaringan toko membuka `http://<alamat-IP-komputer-toko>:3000`.

Untuk mode cloud (HTTPS), set `COOKIE_SECURE=true`.

Bila skema basis data berubah, perbarui cache kueri SQLx agar build tetap bisa berjalan tanpa basis data (`SQLX_OFFLINE=true`):

```bash
cd backend && cargo sqlx prepare -- --all-targets
```

## Struktur

```
backend/
  migrations/      skrip SQL per modul (berjalan otomatis saat server start)
  src/routes/      endpoint: auth, pengguna, produk, sesi, transaksi, log
  src/services/    hitung (perhitungan uang), stok (mutasi + kartu stok), log, sandi
  src/middleware/  autentikasi cookie dan otorisasi peran
  src/bin/seed.rs  data contoh sintetis
  tests/           uji integrasi API
frontend/src/
  views/           halaman (Kasir, Struk, Riwayat, Sesi, Produk, Pengguna, Log)
  stores/          auth, sesi kasir, keranjang
  components/      label harga, dialog, kertas struk
  components/ui/   komponen dasar Tailwind (tombol, isian, pesan, lencana, tabel, kerangka halaman)
  styles.css       tema Tailwind (warna toko, font, radius) dan gaya dasar elemen
```

## Catatan rancangan untuk Bab IV

- **Tambahan tabel di luar Tabel 20:**
  - `detail_retur`: satu retur dapat mencakup beberapa item.
  - `pengaturan`: identitas toko pada struk dan batas diskon kasir.
  - `sesi_login`: sesi login sisi server. Hanya hash token yang disimpan.

  Totalnya kini 17 tabel.
- **`detail_transaksi.nama_barang`** menyalin nama dan varian saat penjualan, sehingga cetak ulang struk sama dengan aslinya walaupun data produk diubah.
- **Log aktivitas** bersifat *append-only*. Aturan ini dijaga oleh trigger basis data, tidak hanya oleh aplikasi.
- **Penyimpanan transaksi** mengikuti rancangan Bab III.C.4:
  - Pengurangan stok bersyarat (`stok + jumlah >= 0`) dalam satu transaksi basis data.
  - Baris varian dikunci dengan urutan id yang konsisten untuk mencegah *deadlock*.
- **Harga dan total** selalu dihitung ulang di server dari basis data, bukan dari isian antarmuka.
- **Kasir tidak menerima harga beli** dalam respons API (Tabel 2).

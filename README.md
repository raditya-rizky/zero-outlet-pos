# Kasir Zero Outlet

Aplikasi kasir (Point of Sale) berbasis web untuk UMKM pakaian, dengan studi kasus Zero Outlet, Seririt, Buleleng.
Capstone Project STSI4440 Kelompok D, Universitas Terbuka.

- **Backend:** Rust + Axum (REST API), SQLx, PostgreSQL
- **Frontend:** Vue 3 + Vite (SPA), Pinia, Vue Router, Tailwind CSS v4
- **Basis data:** PostgreSQL 17 (lewat Docker)

Backend juga menyajikan hasil build frontend, sehingga di komputer toko hanya perlu satu program server dan satu basis data.

## Status: siklus prototipe 3 (semua KF-01 sampai KF-20)

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
| KF-07 | Diskon per baris dan per transaksi (Rp/%), batas diskon kasir | Selesai (siklus 2) |
| KF-10 | Pembatalan, retur, tukar ukuran dengan persetujuan pemilik dari ponsel | Selesai (siklus 2) |
| KF-11 | Barang masuk dari pemasok, harga beli mengikuti harga terbaru | Selesai (siklus 2) |
| KF-12 | Stok opname dengan alasan wajib dan ringkasan akurasi | Selesai (siklus 2) |
| KF-13 | Kartu stok per varian | Selesai (siklus 2) |
| KF-14 | Peringatan stok menipis dan daftar lambat terjual, lencana menu | Selesai (siklus 2) |
| KF-15 | Laporan penjualan harian/mingguan/bulanan, rincian per produk, kategori, ukuran, kasir, dan metode bayar, terlaris, grafik tren | Selesai (siklus 3) |
| KF-16 | Ekspor CSV (penjualan, laba rugi, stok) dan cetak/simpan PDF (A4) | Selesai (siklus 3) |
| KF-17 | Biaya operasional per tanggal dan kategori | Selesai (siklus 3) |
| KF-18 | Laba rugi sederhana dengan pembanding periode sebelumnya | Selesai (siklus 3) |
| Tabel 21 | Dasbor pemilik sebagai halaman awal admin | Selesai (siklus 3) |

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

Uji integrasi siklus 2 ada di `backend/tests/siklus2.rs`:
- UJ-05: diskon transaksi 10% dengan pembayaran tunai
- batas diskon kasir, dan admin tidak dibatasi
- UJ-07: retur tanpa persetujuan pemilik belum mengubah stok dan kas; setelah disetujui, stok kembali dan refund tunai mengurangi kas seharusnya sekali saja
- pembatalan (hanya di sesi yang sama), tukar ukuran (hanya produk yang sama), dan batas sisa retur
- penutupan sesi terhalang pengajuan yang menunggu
- UJ-08: stok opname wajib beralasan, beserta hasil akurasinya
- barang masuk dan perubahan harga beli, kartu stok, serta daftar lambat terjual
- polling tidak memperpanjang sesi login

Uji integrasi siklus 3 ada di `backend/tests/siklus3.rs`:
- satu skenario laba rugi Oktober-November 2025 yang setiap angkanya dihitung manual, mencakup diskon baris dan diskon transaksi, transaksi batal, retur yang disetujui pada bulan berikutnya, tukar ukuran, dan biaya
- rincian per produk berjumlah tepat dan sama dengan dasar nilai retur
- kasir hanya melihat penjualannya sendiri
- format CSV, termasuk penetralan rumus lembar kerja
- validasi biaya dan dasbor

Ada juga uji dua kasir yang menjual unit terakhir secara bersamaan. Setiap uji memakai basis data sementara yang dibuat otomatis oleh `#[sqlx::test]`.

### Uji kinerja laporan (Tabel 1: laporan bulanan ≤ 5 detik)

`seed --riwayat <hari>` membuat riwayat penjualan **sintetis** (sekitar 40 transaksi per hari) dan hanya mau berjalan pada basis data yang namanya mengandung `_uji`. Data ini memakai kasir "Data Contoh (sintetis)" dan faktur berawalan `UJI-`. Jangan pernah dilaporkan sebagai data nyata Zero Outlet.

```bash
docker exec zero-outlet-pos-db-1 createdb -U zero zero_outlet_pos_uji
cd backend
DATABASE_URL=postgres://zero:zero@localhost:5432/zero_outlet_pos_uji cargo run --bin seed
DATABASE_URL=postgres://zero:zero@localhost:5432/zero_outlet_pos_uji cargo run --bin seed -- --riwayat 90
DATABASE_URL=postgres://zero:zero@localhost:5432/zero_outlet_pos_uji BIND_ADDR=127.0.0.1:3001 cargo run
```

Lalu login ke `http://127.0.0.1:3001` dan buka laporan satu bulan. Pengukuran pengembang pada 3.603 transaksi sintetis (90 hari, build debug, laptop pengembang): laporan penjualan bulanan (1.201 transaksi) 0,02–0,04 detik, laba rugi bulanan 0,01 detik. Angka resmi untuk Bab IV tetap diukur ulang di komputer toko dengan data nyata.

## Pemasangan di komputer toko (mode lokal)

1. Pasang Docker, lalu jalankan `docker compose up -d`.
2. Build frontend: `cd frontend && pnpm install --frozen-lockfile && pnpm build`.
3. Build backend: `cd backend && cargo build --release`.
4. Salin `.env.example` ke `backend/.env`. Ubah `ADMIN_PASSWORD`, dan pastikan `FRONTEND_DIST=../frontend/dist`.
5. Jalankan `./target/release/zero-outlet-pos` dari folder `backend`.
6. Perangkat lain di jaringan toko membuka `http://<alamat-IP-komputer-toko>:3000`.

Untuk mode cloud (HTTPS), set `COOKIE_SECURE=true`.

**Persetujuan dari ponsel pemilik (KF-10).** Pemilik menyetujui pembatalan, retur, dan tukar ukuran di halaman *Persetujuan* lewat peramban ponsel. Pada mode lokal, ponsel harus terhubung ke Wi-Fi toko yang sama dengan komputer server. Agar pemilik dapat menyetujui dari luar toko, gunakan mode cloud (atau akses jaringan pribadi seperti VPN ke komputer toko). Jika pemilik tidak dapat dihubungi, kasir dapat menarik pengajuannya agar sesi kasir tetap bisa ditutup.

Bila skema basis data berubah, perbarui cache kueri SQLx agar build tetap bisa berjalan tanpa basis data (`SQLX_OFFLINE=true`):

```bash
cd backend && cargo sqlx prepare -- --all-targets
```

## Struktur

```
backend/
  migrations/      skrip SQL per modul (berjalan otomatis saat server start)
  src/routes/      endpoint: auth, pengguna, produk, sesi, transaksi, pengajuan, barang_masuk, stok, laporan, biaya, pengaturan, log
  src/services/    hitung (uang, diskon, nilai retur), periode (periode pembanding, margin), stok, log, pengaturan, sandi
  src/middleware/  autentikasi cookie dan otorisasi peran
  src/bin/seed.rs  data contoh sintetis; --riwayat untuk uji kinerja (basis data _uji saja)
  tests/           uji integrasi API
frontend/src/
  views/           halaman (Dasbor, Kasir, Struk, Riwayat, Pengajuan, Sesi, Persetujuan, Produk, Stok, Laporan, Laba rugi, Biaya, Pengguna, Log, Pengaturan)
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
- **Siklus 2, tabel `retur` menjadi tabel pengajuan** (migrasi 0009):
  - `jenis`: `batal` | `retur` | `tukar`.
  - Status: `menunggu` → `disetujui` | `ditolak` | `dibatalkan` (ditarik kasir).
  - Nilai pengembalian dihitung saat pengajuan dibuat. Diskon transaksi dibagi proporsional per baris, dan retur sebagian memakai pembulatan kumulatif sehingga total tidak pernah melebihi yang dibayar.
- **Pembatalan hanya di sesi yang sama.** Pembatalan dibatasi pada sesi kasir yang masih dibuka, tempat transaksi terjadi. Transaksi lain memakai retur. Dengan begitu rekap sesi yang sudah ditutup tidak berubah, dan pembatalan tidak dihitung dua kali pada kas seharusnya.
- **Kasir boleh mencari transaksi kasir lain** dengan nomor faktur lengkap untuk melayani retur. Riwayat dan struk kasir lain tetap tertutup.
- **Polling tidak memperpanjang sesi.** Permintaan berkala dari antarmuka (status pengajuan dan lencana menu) memakai header `X-Latar`, sehingga tidak memperpanjang sesi login dan keluar otomatis (KF-01) tetap berjalan.
- **Batas diskon kasir** dihitung dengan bilangan bulat terhadap subtotal kotor: `diskon × 100 > kotor × batas`.
- **Siklus 3, aturan angka laporan:**
  - Penjualan hanya dari transaksi berstatus `selesai`.
  - Penjualan bersih = kotor − diskon − retur, sehingga sama dengan total transaksi dikurangi retur.
  - **Retur dihitung pada tanggal disetujui**, bukan tanggal penjualan asal. Laporan periode yang sudah lewat tidak berubah, dan angkanya cocok dengan rekap kas sesi. Hanya pembatalan yang bisa mengubah hari sebelumnya, dan itu pun hanya bila sesi kasir melewati tengah malam.
  - Tukar ukuran tidak memengaruhi penjualan maupun HPP, karena ditukar dalam produk yang sama dengan harga yang sama.
  - HPP = harga pokok saat terjual dikurangi harga pokok barang yang diretur.
  - Margin disimpan dalam basis poin (bilangan bulat) dan kosong bila penjualan bersih nol atau minus.
- **Rincian per produk/kategori/ukuran membagi diskon transaksi** dengan cara yang sama persis dengan perhitungan nilai retur (bagian proporsional, sisa ke baris bernilai terbesar). Jumlah rincian sama tepat dengan total penjualan.
- **Retur pada laporan per kasir** mengurangi angka kasir yang menjual transaksi asalnya, bukan kasir yang memproses retur.
- **Periode pembanding laba rugi:**
  - Bulan penuh dibandingkan bulan penuh sebelumnya.
  - "Bulan ini sejauh ini" dibandingkan tanggal yang sama pada bulan lalu.
  - Rentang lain dibandingkan jumlah hari yang sama tepat sebelumnya.
- **CSV** memakai UTF-8 dengan BOM, pemisah `;`, dan angka tanpa titik ribuan, agar langsung terbaca Excel berbahasa Indonesia. Teks yang diawali `= + - @` diberi tanda kutip agar tidak dijalankan sebagai rumus.
- **Perbaikan cetak struk:** aturan lama `@page { size: 58mm auto }` tidak sah sehingga diabaikan peramban, dan struk tercetak pada ukuran kertas bawaan (Letter/A4). Sekarang laporan memakai A4, sedangkan halaman struk menetapkan ukuran 58 mm × tinggi struk saat dibuka. Hasilnya diuji lewat PDF (58 × 84 mm untuk struk contoh).

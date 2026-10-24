-- Sesi kasir dan rekonsiliasi kas (KF-19).
CREATE TABLE sesi_kasir (
    id_sesi         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    id_pengguna     BIGINT      NOT NULL REFERENCES pengguna (id_pengguna),
    waktu_buka      TIMESTAMPTZ NOT NULL DEFAULT now(),
    modal_awal      BIGINT      NOT NULL CHECK (modal_awal >= 0),
    waktu_tutup     TIMESTAMPTZ,
    kas_fisik       BIGINT      CHECK (kas_fisik >= 0),
    kas_seharusnya  BIGINT,
    selisih         BIGINT,
    alasan_selisih  TEXT,
    -- Selisih selain nol wajib disertai alasan.
    CHECK (selisih IS NULL OR selisih = 0 OR coalesce(trim(alasan_selisih), '') <> '')
);
-- Satu kasir hanya boleh memiliki satu sesi aktif.
CREATE UNIQUE INDEX sesi_kasir_satu_aktif ON sesi_kasir (id_pengguna) WHERE waktu_tutup IS NULL;

CREATE SEQUENCE no_faktur_seq;

-- Header transaksi penjualan (KF-06, KF-08, KF-09).
CREATE TABLE transaksi (
    id_transaksi  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    no_faktur     TEXT        NOT NULL UNIQUE,
    id_sesi       BIGINT      NOT NULL REFERENCES sesi_kasir (id_sesi),
    id_pengguna   BIGINT      NOT NULL REFERENCES pengguna (id_pengguna),
    tanggal_waktu TIMESTAMPTZ NOT NULL DEFAULT now(),
    subtotal      BIGINT      NOT NULL CHECK (subtotal >= 0),
    diskon        BIGINT      NOT NULL DEFAULT 0 CHECK (diskon >= 0),
    total         BIGINT      NOT NULL CHECK (total >= 0),
    metode_bayar  TEXT        NOT NULL CHECK (metode_bayar IN ('tunai', 'transfer', 'qris')),
    bayar         BIGINT      NOT NULL,
    kembalian     BIGINT      NOT NULL CHECK (kembalian >= 0),
    status        TEXT        NOT NULL DEFAULT 'selesai' CHECK (status IN ('selesai', 'batal')),
    jumlah_cetak  INTEGER     NOT NULL DEFAULT 1,
    CHECK (total = subtotal - diskon),
    CHECK (kembalian = bayar - total)
);
CREATE INDEX transaksi_sesi_idx ON transaksi (id_sesi);
CREATE INDEX transaksi_tanggal_idx ON transaksi (tanggal_waktu);
CREATE INDEX transaksi_pengguna_idx ON transaksi (id_pengguna);

-- Harga dan nama barang disalin saat transaksi agar laporan dan cetak ulang
-- struk tidak berubah ketika data produk diperbarui.
CREATE TABLE detail_transaksi (
    id_detail     BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    id_transaksi  BIGINT  NOT NULL REFERENCES transaksi (id_transaksi) ON DELETE CASCADE,
    id_varian     BIGINT  NOT NULL REFERENCES varian (id_varian),
    nama_barang   TEXT    NOT NULL,
    jumlah        INTEGER NOT NULL CHECK (jumlah > 0),
    harga_satuan  BIGINT  NOT NULL CHECK (harga_satuan >= 0),
    harga_pokok   BIGINT  NOT NULL CHECK (harga_pokok >= 0),
    diskon_item   BIGINT  NOT NULL DEFAULT 0 CHECK (diskon_item >= 0),
    subtotal      BIGINT  NOT NULL,
    CHECK (subtotal = jumlah * harga_satuan - diskon_item)
);
CREATE INDEX detail_transaksi_transaksi_idx ON detail_transaksi (id_transaksi);
CREATE INDEX detail_transaksi_varian_idx ON detail_transaksi (id_varian);

-- Retur dan tukar ukuran (KF-10, siklus 2). Satu retur dapat mencakup
-- beberapa item dari satu transaksi, sehingga rinciannya disimpan di detail_retur.
CREATE TABLE retur (
    id_retur            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    id_transaksi        BIGINT      NOT NULL REFERENCES transaksi (id_transaksi),
    tanggal_waktu       TIMESTAMPTZ NOT NULL DEFAULT now(),
    jenis               TEXT        NOT NULL CHECK (jenis IN ('retur', 'tukar')),
    alasan              TEXT        NOT NULL,
    total_pengembalian  BIGINT      NOT NULL DEFAULT 0 CHECK (total_pengembalian >= 0),
    metode_pengembalian TEXT        CHECK (metode_pengembalian IN ('tunai', 'transfer', 'qris')),
    id_sesi             BIGINT      REFERENCES sesi_kasir (id_sesi), -- sesi tempat refund tunai dikeluarkan
    id_pengguna         BIGINT      NOT NULL REFERENCES pengguna (id_pengguna),
    id_otorisator       BIGINT      NOT NULL REFERENCES pengguna (id_pengguna)
);
CREATE INDEX retur_transaksi_idx ON retur (id_transaksi);

CREATE TABLE detail_retur (
    id_detail_retur     BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    id_retur            BIGINT  NOT NULL REFERENCES retur (id_retur) ON DELETE CASCADE,
    id_detail           BIGINT  NOT NULL REFERENCES detail_transaksi (id_detail),
    jumlah              INTEGER NOT NULL CHECK (jumlah > 0),
    nilai               BIGINT  NOT NULL CHECK (nilai >= 0),
    id_varian_pengganti BIGINT  REFERENCES varian (id_varian) -- diisi bila tukar ukuran
);

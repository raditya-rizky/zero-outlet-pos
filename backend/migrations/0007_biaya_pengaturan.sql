-- Biaya operasional untuk laporan laba rugi (KF-17, siklus 3).
CREATE TABLE biaya_operasional (
    id_biaya     BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tanggal      DATE        NOT NULL,
    kategori     TEXT        NOT NULL,
    nominal      BIGINT      NOT NULL CHECK (nominal > 0),
    keterangan   TEXT,
    id_pengguna  BIGINT      NOT NULL REFERENCES pengguna (id_pengguna),
    dibuat_pada  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX biaya_operasional_tanggal_idx ON biaya_operasional (tanggal);

-- Pengaturan toko dalam bentuk kunci-nilai.
CREATE TABLE pengaturan (
    kunci       TEXT PRIMARY KEY,
    nilai       TEXT NOT NULL,
    keterangan  TEXT
);

INSERT INTO pengaturan (kunci, nilai, keterangan) VALUES
    ('nama_toko', 'Zero Outlet', 'Ditampilkan pada struk'),
    ('alamat_toko', 'Desa Seririt, Kec. Seririt, Buleleng, Bali', 'Ditampilkan pada struk'),
    ('catatan_struk', 'Terima kasih atas kunjungan Anda. Barang dapat ditukar maks. 3 hari dengan struk.', 'Catatan di bagian bawah struk'),
    ('batas_diskon_kasir_persen', '10', 'Batas maksimum diskon yang dapat diberikan kasir tanpa admin (KF-07)');

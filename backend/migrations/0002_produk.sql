-- Data induk produk dan varian ukuran-warna (KF-03, KF-04).
CREATE TABLE kategori (
    id_kategori   BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nama_kategori TEXT NOT NULL UNIQUE
);

CREATE TABLE produk (
    id_produk    BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    kode         TEXT        NOT NULL UNIQUE,
    nama         TEXT        NOT NULL,
    id_kategori  BIGINT      NOT NULL REFERENCES kategori (id_kategori),
    harga_beli   BIGINT      NOT NULL CHECK (harga_beli >= 0), -- rupiah
    harga_jual   BIGINT      NOT NULL CHECK (harga_jual >= 0), -- rupiah
    aktif        BOOLEAN     NOT NULL DEFAULT TRUE,
    dibuat_pada  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX produk_kategori_idx ON produk (id_kategori);

-- Varian adalah satuan stok terkecil.
CREATE TABLE varian (
    id_varian     BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    id_produk     BIGINT  NOT NULL REFERENCES produk (id_produk) ON DELETE CASCADE,
    sku           TEXT    NOT NULL UNIQUE,
    ukuran        TEXT    NOT NULL,
    warna         TEXT    NOT NULL,
    stok          INTEGER NOT NULL DEFAULT 0 CHECK (stok >= 0),
    stok_minimum  INTEGER NOT NULL DEFAULT 0 CHECK (stok_minimum >= 0),
    aktif         BOOLEAN NOT NULL DEFAULT TRUE,
    UNIQUE (id_produk, ukuran, warna)
);
CREATE INDEX varian_produk_idx ON varian (id_produk);

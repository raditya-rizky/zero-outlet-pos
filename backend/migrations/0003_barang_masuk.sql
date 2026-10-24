-- Penerimaan barang dari pemasok (KF-11, siklus 2).
CREATE TABLE pemasok (
    id_pemasok  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nama        TEXT NOT NULL,
    kontak      TEXT,
    alamat      TEXT
);

CREATE TABLE barang_masuk (
    id_barang_masuk BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    no_referensi    TEXT,
    tanggal         DATE        NOT NULL DEFAULT CURRENT_DATE,
    id_pemasok      BIGINT      REFERENCES pemasok (id_pemasok),
    id_pengguna     BIGINT      NOT NULL REFERENCES pengguna (id_pengguna),
    keterangan      TEXT,
    dibuat_pada     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE detail_barang_masuk (
    id_detail       BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    id_barang_masuk BIGINT  NOT NULL REFERENCES barang_masuk (id_barang_masuk) ON DELETE CASCADE,
    id_varian       BIGINT  NOT NULL REFERENCES varian (id_varian),
    jumlah          INTEGER NOT NULL CHECK (jumlah > 0),
    harga_beli      BIGINT  NOT NULL CHECK (harga_beli >= 0)
);

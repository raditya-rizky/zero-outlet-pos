-- Kartu stok per varian (KF-13). Jumlah bertanda: positif = masuk, negatif = keluar.
CREATE TABLE mutasi_stok (
    id_mutasi     BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    id_varian     BIGINT      NOT NULL REFERENCES varian (id_varian),
    tanggal       TIMESTAMPTZ NOT NULL DEFAULT now(),
    jenis         TEXT        NOT NULL CHECK (jenis IN ('masuk', 'keluar', 'retur', 'penyesuaian')),
    jumlah        INTEGER     NOT NULL CHECK (jumlah <> 0),
    stok_sesudah  INTEGER     NOT NULL CHECK (stok_sesudah >= 0),
    referensi     TEXT,
    id_pengguna   BIGINT      NOT NULL REFERENCES pengguna (id_pengguna),
    keterangan    TEXT
);
CREATE INDEX mutasi_stok_varian_idx ON mutasi_stok (id_varian, tanggal);

-- Akun pengguna (KF-01, KF-02). Akun tidak dihapus, hanya dinonaktifkan.
CREATE TABLE pengguna (
    id_pengguna  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nama         TEXT        NOT NULL,
    username     TEXT        NOT NULL UNIQUE,
    kata_sandi   TEXT        NOT NULL, -- hash Argon2 (format PHC)
    peran        TEXT        NOT NULL CHECK (peran IN ('kasir', 'admin')),
    aktif        BOOLEAN     NOT NULL DEFAULT TRUE,
    dibuat_pada  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Sesi login sisi server. Hanya hash SHA-256 dari token yang disimpan.
CREATE TABLE sesi_login (
    id_sesi_login  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    token_hash     TEXT        NOT NULL UNIQUE,
    id_pengguna    BIGINT      NOT NULL REFERENCES pengguna (id_pengguna),
    dibuat_pada    TIMESTAMPTZ NOT NULL DEFAULT now(),
    terakhir_aktif TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX sesi_login_pengguna_idx ON sesi_login (id_pengguna);

-- Jejak audit (KF-20). Hanya dapat ditambah; perubahan dan penghapusan ditolak
-- oleh trigger sehingga log tidak dapat diubah melalui aplikasi.
CREATE TABLE log_aktivitas (
    id_log       BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    id_pengguna  BIGINT      REFERENCES pengguna (id_pengguna), -- kosong untuk login gagal dengan username tak dikenal
    waktu        TIMESTAMPTZ NOT NULL DEFAULT now(),
    aksi         TEXT        NOT NULL,
    objek        TEXT        NOT NULL,
    id_objek     TEXT,
    nilai_lama   JSONB,
    nilai_baru   JSONB,
    keterangan   TEXT
);
CREATE INDEX log_aktivitas_waktu_idx ON log_aktivitas (waktu);
CREATE INDEX log_aktivitas_pengguna_idx ON log_aktivitas (id_pengguna);

CREATE FUNCTION tolak_ubah_log() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'log_aktivitas bersifat append-only dan tidak dapat diubah atau dihapus';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER log_aktivitas_append_only
    BEFORE UPDATE OR DELETE ON log_aktivitas
    FOR EACH ROW EXECUTE FUNCTION tolak_ubah_log();

CREATE TRIGGER log_aktivitas_tanpa_truncate
    BEFORE TRUNCATE ON log_aktivitas
    FOR EACH STATEMENT EXECUTE FUNCTION tolak_ubah_log();

-- Pengajuan pembatalan, retur, dan tukar ukuran dengan persetujuan pemilik (KF-10).
-- Tabel retur dipakai untuk ketiga jenis pengajuan. Kasir mengajukan (menunggu),
-- pemilik menyetujui atau menolak, dan kasir dapat menarik pengajuannya sendiri.
ALTER TABLE retur
    DROP CONSTRAINT retur_jenis_check,
    ADD CONSTRAINT retur_jenis_check CHECK (jenis IN ('batal', 'retur', 'tukar')),
    ALTER COLUMN id_otorisator DROP NOT NULL,
    ALTER COLUMN id_sesi SET NOT NULL,
    ADD COLUMN status TEXT NOT NULL DEFAULT 'menunggu'
        CHECK (status IN ('menunggu', 'disetujui', 'ditolak', 'dibatalkan')),
    ADD COLUMN waktu_keputusan TIMESTAMPTZ,
    ADD COLUMN alasan_tolak TEXT,
    -- Disetujui/ditolak wajib punya pemberi keputusan dan waktunya; ditarik kasir cukup waktunya.
    ADD CONSTRAINT retur_keputusan_check CHECK (
        (status = 'menunggu' AND waktu_keputusan IS NULL AND id_otorisator IS NULL)
        OR (status = 'dibatalkan' AND waktu_keputusan IS NOT NULL AND id_otorisator IS NULL)
        OR (status IN ('disetujui', 'ditolak') AND waktu_keputusan IS NOT NULL AND id_otorisator IS NOT NULL)
    ),
    ADD CONSTRAINT retur_alasan_tolak_check CHECK (status <> 'ditolak' OR coalesce(trim(alasan_tolak), '') <> ''),
    ADD CONSTRAINT retur_metode_check CHECK (jenis = 'tukar' OR metode_pengembalian IS NOT NULL);

CREATE INDEX retur_status_idx ON retur (status);
CREATE INDEX retur_sesi_idx ON retur (id_sesi);
CREATE INDEX detail_retur_retur_idx ON detail_retur (id_retur);
CREATE INDEX detail_retur_detail_idx ON detail_retur (id_detail);

INSERT INTO pengaturan (kunci, nilai, keterangan) VALUES
    ('batas_hari_retur', '3', 'Batas hari retur/tukar; lewat batas tetap boleh diajukan dengan peringatan'),
    ('hari_lambat_terjual', '30', 'Varian tanpa penjualan selama N hari dianggap lambat terjual (KF-14)');

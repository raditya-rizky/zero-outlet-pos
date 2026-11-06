-- Laporan dan biaya operasional (KF-15 sampai KF-18).
-- Kategori biaya mengikuti contoh pada proposal; "Lain-lain" wajib berketerangan.
ALTER TABLE biaya_operasional
    ADD CONSTRAINT biaya_operasional_kategori_check CHECK (
        kategori IN ('Sewa tempat', 'Listrik dan air', 'Upah karyawan', 'Pengemasan', 'Promosi', 'Lain-lain')
    ),
    ADD CONSTRAINT biaya_operasional_keterangan_check CHECK (
        kategori <> 'Lain-lain' OR coalesce(trim(keterangan), '') <> ''
    );

-- Laporan memfilter transaksi selesai dan retur disetujui berdasarkan waktunya.
CREATE INDEX transaksi_status_tanggal_idx ON transaksi (status, tanggal_waktu);
CREATE INDEX retur_status_keputusan_idx ON retur (status, waktu_keputusan);

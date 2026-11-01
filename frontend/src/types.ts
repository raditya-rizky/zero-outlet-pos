// Bentuk data dari REST API backend.

export type Peran = 'kasir' | 'admin'
export type MetodeBayar = 'tunai' | 'transfer' | 'qris'

export interface PenggunaLogin {
  id_pengguna: number
  nama: string
  username: string
  peran: Peran
}

export interface Pengguna extends PenggunaLogin {
  aktif: boolean
  dibuat_pada: string
}

export interface Kategori {
  id_kategori: number
  nama_kategori: string
}

export interface Varian {
  id_varian: number
  id_produk: number
  sku: string
  ukuran: string
  warna: string
  stok: number
  stok_minimum: number
  aktif: boolean
}

export interface Produk {
  id_produk: number
  kode: string
  nama: string
  id_kategori: number
  nama_kategori: string
  harga_beli?: number
  harga_jual: number
  aktif: boolean
  varian: Varian[]
}

export interface Sesi {
  id_sesi: number
  id_pengguna: number
  nama_kasir: string
  waktu_buka: string
  modal_awal: number
  waktu_tutup: string | null
  kas_fisik: number | null
  kas_seharusnya: number | null
  selisih: number | null
  alasan_selisih: string | null
}

export interface Rekap {
  sesi: Sesi
  jumlah_transaksi: number
  penjualan_tunai: number
  penjualan_transfer: number
  penjualan_qris: number
  total_penjualan: number
  refund_tunai: number
  kas_seharusnya: number
}

export interface ItemStruk {
  id_detail: number
  id_varian: number
  nama_barang: string
  jumlah: number
  harga_satuan: number
  diskon_item: number
  subtotal: number
  /** Unit yang sudah diretur lewat pengajuan yang disetujui. */
  jumlah_diretur: number
}

export interface Struk {
  toko: { nama: string; alamat: string; catatan: string }
  id_transaksi: number
  no_faktur: string
  tanggal_waktu: string
  id_sesi: number
  id_pengguna: number
  nama_kasir: string
  subtotal: number
  diskon: number
  total: number
  metode_bayar: MetodeBayar
  bayar: number
  kembalian: number
  status: 'selesai' | 'batal'
  jumlah_cetak: number
  cetak_ulang: boolean
  items: ItemStruk[]
  pengajuan: RingkasPengajuan[]
}

export interface RingkasTransaksi {
  id_transaksi: number
  no_faktur: string
  tanggal_waktu: string
  id_pengguna: number
  nama_kasir: string
  total: number
  metode_bayar: MetodeBayar
  status: 'selesai' | 'batal'
  jumlah_barang: number
}

export interface LogAktivitas {
  id_log: number
  waktu: string
  id_pengguna: number | null
  nama_pengguna: string | null
  aksi: string
  objek: string
  id_objek: string | null
  nilai_lama: unknown
  nilai_baru: unknown
  keterangan: string | null
}

export interface HalamanLog {
  data: LogAktivitas[]
  total: number
  halaman: number
  per_halaman: number
}

// ---------- Siklus 2 ----------

export interface Pengaturan {
  nama_toko: string
  alamat_toko: string
  catatan_struk: string
  batas_diskon_kasir_persen: number
  batas_hari_retur: number
  hari_lambat_terjual: number
}

export type JenisPengajuan = 'batal' | 'retur' | 'tukar'
export type StatusPengajuan = 'menunggu' | 'disetujui' | 'ditolak' | 'dibatalkan'

export interface RingkasPengajuan {
  id_retur: number
  jenis: JenisPengajuan
  status: StatusPengajuan
  total_pengembalian: number
  tanggal_waktu: string
}

export interface ItemPengajuan {
  id_detail_retur: number
  id_detail: number
  id_varian: number
  nama_barang: string
  jumlah: number
  nilai: number
  id_varian_pengganti: number | null
  nama_pengganti: string | null
}

export interface Pengajuan {
  id_retur: number
  id_transaksi: number
  no_faktur: string
  tanggal_transaksi: string
  jenis: JenisPengajuan
  status: StatusPengajuan
  alasan: string
  alasan_tolak: string | null
  total_pengembalian: number
  metode_pengembalian: MetodeBayar | null
  id_sesi: number
  id_pengguna: number
  nama_pengaju: string
  nama_otorisator: string | null
  tanggal_waktu: string
  waktu_keputusan: string | null
  lewat_batas_hari: boolean
  items: ItemPengajuan[]
}

export interface VarianPengganti {
  id_varian: number
  ukuran: string
  warna: string
  stok: number
}

export interface BarisFaktur {
  id_detail: number
  id_varian: number
  nama_barang: string
  jumlah: number
  subtotal: number
  dibayar: number
  sisa: number
  pengganti: VarianPengganti[]
}

export interface FakturPengajuan {
  id_transaksi: number
  no_faktur: string
  tanggal_waktu: string
  nama_kasir: string
  status: 'selesai' | 'batal'
  metode_bayar: MetodeBayar
  subtotal: number
  diskon: number
  total: number
  lewat_batas_hari: boolean
  bisa_dibatalkan: boolean
  items: BarisFaktur[]
}

export interface Pemasok {
  id_pemasok: number
  nama: string
  kontak: string | null
  alamat: string | null
}

export interface ItemBarangMasuk {
  id_detail: number
  id_varian: number
  nama_barang: string
  sku: string
  jumlah: number
  harga_beli: number
}

export interface BarangMasuk {
  id_barang_masuk: number
  tanggal: string
  no_referensi: string | null
  id_pemasok: number | null
  nama_pemasok: string | null
  nama_pengguna: string
  keterangan: string | null
  jumlah_unit: number
  total_nilai: number
  dibuat_pada: string
  items: ItemBarangMasuk[]
}

export interface SelisihOpname {
  id_varian: number
  nama_barang: string
  sku: string
  stok_sistem: number
  stok_fisik: number
  selisih: number
}

export interface HasilOpname {
  referensi: string
  dihitung: number
  sesuai: number
  akurasi_persen: number
  selisih: SelisihOpname[]
}

export interface MutasiKartu {
  id_mutasi: number
  tanggal: string
  jenis: 'masuk' | 'keluar' | 'retur' | 'penyesuaian'
  jumlah: number
  stok_sesudah: number
  referensi: string | null
  keterangan: string | null
  nama_pengguna: string
}

export interface KartuStok {
  id_varian: number
  id_produk: number
  nama_produk: string
  sku: string
  ukuran: string
  warna: string
  stok: number
  stok_minimum: number
  saldo_awal: number
  mutasi: MutasiKartu[]
}

export interface VarianStok {
  id_varian: number
  id_produk: number
  nama_produk: string
  kode: string
  sku: string
  ukuran: string
  warna: string
  stok: number
  stok_minimum: number
  terakhir_terjual: string | null
}

export interface PeringatanStok {
  hari_lambat_terjual: number
  menipis: VarianStok[]
  lambat_terjual: VarianStok[]
}

export interface Lencana {
  pengajuan_menunggu: number
  stok_menipis: number
}

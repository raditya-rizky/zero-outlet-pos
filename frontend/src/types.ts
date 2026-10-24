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

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import type { Produk, Varian } from '@/types'

/** Diskon disimpan sebagai masukan kasir (Rp atau %) dan dihitung ulang saat jumlah berubah. */
export interface Diskon {
  mode: 'rp' | 'persen'
  nilai: number
}

export interface ItemKeranjang {
  id_varian: number
  nama: string
  ukuran: string
  warna: string
  sku: string
  harga: number
  stok: number
  jumlah: number
  diskon: Diskon
}

const tanpaDiskon = (): Diskon => ({ mode: 'rp', nilai: 0 })

/** Rupiah utuh dari masukan diskon terhadap suatu dasar; persen dibulatkan ke bawah. */
export function nilaiDiskon(d: Diskon, dasar: number): number {
  const n = d.mode === 'persen' ? Math.floor((dasar * Math.min(d.nilai, 100)) / 100) : d.nilai
  return Math.max(0, Math.min(Math.floor(n) || 0, dasar))
}

/**
 * Keranjang belanja. Total di sini hanya untuk tampilan; backend menghitung
 * ulang dari harga di basis data dan memeriksa batas diskon saat transaksi disimpan.
 */
export const useKeranjang = defineStore('keranjang', () => {
  const items = ref<ItemKeranjang[]>([])
  const diskonTransaksi = ref<Diskon>(tanpaDiskon())
  const pesan = ref<string | null>(null)

  const jumlahBarang = computed(() => items.value.reduce((n, i) => n + i.jumlah, 0))
  /** Subtotal kotor sebelum diskon; dasar batas diskon kasir (KF-07). */
  const kotor = computed(() => items.value.reduce((n, i) => n + i.jumlah * i.harga, 0))
  const diskonBaris = (i: ItemKeranjang) => nilaiDiskon(i.diskon, i.jumlah * i.harga)
  const subtotalBaris = (i: ItemKeranjang) => i.jumlah * i.harga - diskonBaris(i)
  const subtotal = computed(() => items.value.reduce((n, i) => n + subtotalBaris(i), 0))
  const diskonTrx = computed(() => nilaiDiskon(diskonTransaksi.value, subtotal.value))
  const diskonTotal = computed(() => kotor.value - subtotal.value + diskonTrx.value)
  const total = computed(() => subtotal.value - diskonTrx.value)

  /** Sama dengan pemeriksaan backend: diskon x 100 > kotor x batas. */
  function melebihiBatas(batasPersen: number) {
    return diskonTotal.value * 100 > kotor.value * batasPersen
  }

  function cari(id_varian: number) {
    return items.value.find((i) => i.id_varian === id_varian)
  }

  /** Menambah satu unit; ditolak bila melebihi stok varian (KF-06). */
  function tambah(produk: Produk, varian: Varian): boolean {
    pesan.value = null
    const ada = cari(varian.id_varian)
    const jumlahBaru = (ada?.jumlah ?? 0) + 1
    if (jumlahBaru > varian.stok) {
      pesan.value = `Stok ${produk.nama} ${varian.ukuran}/${varian.warna} tinggal ${varian.stok}`
      return false
    }
    if (ada) {
      ada.jumlah = jumlahBaru
      ada.stok = varian.stok
    } else {
      items.value.push({
        id_varian: varian.id_varian,
        nama: produk.nama,
        ukuran: varian.ukuran,
        warna: varian.warna,
        sku: varian.sku,
        harga: produk.harga_jual,
        stok: varian.stok,
        jumlah: 1,
        diskon: tanpaDiskon(),
      })
    }
    return true
  }

  function ubahJumlah(id_varian: number, jumlah: number) {
    pesan.value = null
    const item = cari(id_varian)
    if (!item) return
    if (jumlah <= 0) return hapus(id_varian)
    if (jumlah > item.stok) {
      pesan.value = `Stok ${item.nama} ${item.ukuran}/${item.warna} tinggal ${item.stok}`
      item.jumlah = item.stok
      return
    }
    item.jumlah = jumlah
  }

  function hapus(id_varian: number) {
    items.value = items.value.filter((i) => i.id_varian !== id_varian)
  }

  function kosongkan() {
    items.value = []
    diskonTransaksi.value = tanpaDiskon()
    pesan.value = null
  }

  /** Isi permintaan POST /transaksi; diskon_item adalah rupiah untuk seluruh baris. */
  function muatan() {
    return {
      items: items.value.map((i) => ({ id_varian: i.id_varian, jumlah: i.jumlah, diskon_item: diskonBaris(i) })),
      diskon_transaksi: diskonTrx.value,
    }
  }

  return {
    items,
    diskonTransaksi,
    pesan,
    jumlahBarang,
    kotor,
    subtotal,
    diskonTrx,
    diskonTotal,
    total,
    diskonBaris,
    subtotalBaris,
    melebihiBatas,
    tambah,
    ubahJumlah,
    hapus,
    kosongkan,
    muatan,
  }
})

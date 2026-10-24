import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import type { Produk, Varian } from '@/types'

export interface ItemKeranjang {
  id_varian: number
  nama: string
  ukuran: string
  warna: string
  sku: string
  harga: number
  stok: number
  jumlah: number
}

/**
 * Keranjang belanja. Total di sini hanya untuk tampilan; backend menghitung
 * ulang dari harga di basis data saat transaksi disimpan.
 */
export const useKeranjang = defineStore('keranjang', () => {
  const items = ref<ItemKeranjang[]>([])
  const pesan = ref<string | null>(null)

  const jumlahBarang = computed(() => items.value.reduce((n, i) => n + i.jumlah, 0))
  const total = computed(() => items.value.reduce((n, i) => n + i.jumlah * i.harga, 0))

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
    pesan.value = null
  }

  return { items, pesan, jumlahBarang, total, tambah, ubahJumlah, hapus, kosongkan }
})

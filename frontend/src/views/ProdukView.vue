<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { api, pesanError } from '@/api'
import { rupiah } from '@/format'
import type { Kategori, Produk } from '@/types'

const route = useRoute()
const cari = ref('')
const idKategori = ref('')
const status = ref('aktif')
const kategori = ref<Kategori[]>([])
const data = ref<Produk[]>([])
const galat = ref<string | null>(null)
const memuat = ref(false)
const pesan = ref(typeof route.query.pesan === 'string' ? route.query.pesan : null)

async function muat() {
  memuat.value = true
  galat.value = null
  try {
    data.value = await api.get<Produk[]>('/produk', { cari: cari.value, id_kategori: idKategori.value, status: status.value })
  } catch (e) {
    galat.value = pesanError(e)
  } finally {
    memuat.value = false
  }
}

let jeda: ReturnType<typeof setTimeout> | undefined
watch(cari, () => {
  clearTimeout(jeda)
  jeda = setTimeout(muat, 300)
})
watch([idKategori, status], muat)

const totalStok = (p: Produk) => p.varian.filter((v) => v.aktif).reduce((n, v) => n + v.stok, 0)
const menipis = (p: Produk) => p.varian.filter((v) => v.aktif && v.stok <= v.stok_minimum).length

onMounted(async () => {
  muat()
  kategori.value = await api.get<Kategori[]>('/kategori').catch(() => [])
})
</script>

<template>
  <div class="halaman">
    <div class="kepala-halaman">
      <div>
        <h1>Produk</h1>
        <p class="redup">Model pakaian beserta varian ukuran dan warnanya. Stok dicatat per varian.</p>
      </div>
      <RouterLink to="/admin/produk/baru" class="tombol">Tambah produk</RouterLink>
    </div>

    <p v-if="pesan" class="pesan berhasil">{{ pesan }}</p>

    <div class="saring">
      <label class="isian">Cari <input v-model="cari" type="search" placeholder="Nama, kode, SKU, warna" /></label>
      <label class="isian">
        Kategori
        <select v-model="idKategori">
          <option value="">Semua kategori</option>
          <option v-for="k in kategori" :key="k.id_kategori" :value="k.id_kategori">{{ k.nama_kategori }}</option>
        </select>
      </label>
      <label class="isian">
        Status
        <select v-model="status">
          <option value="aktif">Dijual</option>
          <option value="nonaktif">Nonaktif</option>
          <option value="semua">Semua</option>
        </select>
      </label>
    </div>

    <p v-if="galat" class="pesan galat">{{ galat }}</p>

    <div class="tabel-bungkus">
      <table>
        <thead>
          <tr>
            <th>Produk</th>
            <th>Kategori</th>
            <th class="kanan">Harga beli</th>
            <th class="kanan">Harga jual</th>
            <th class="kanan">Varian</th>
            <th class="kanan">Stok</th>
            <th>Keterangan</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="p in data" :key="p.id_produk">
            <td>
              <strong>{{ p.nama }}</strong><br />
              <span class="kecil redup">{{ p.kode }}</span>
            </td>
            <td>{{ p.nama_kategori }}</td>
            <td class="kanan angka">{{ rupiah(p.harga_beli ?? 0) }}</td>
            <td class="kanan angka">{{ rupiah(p.harga_jual) }}</td>
            <td class="kanan angka">{{ p.varian.filter((v) => v.aktif).length }}</td>
            <td class="kanan angka">{{ totalStok(p) }}</td>
            <td>
              <span v-if="!p.aktif" class="lencana netral">Nonaktif</span>
              <span v-else-if="menipis(p)" class="lencana waspada">{{ menipis(p) }} varian menipis</span>
            </td>
            <td class="kanan"><RouterLink :to="`/admin/produk/${p.id_produk}`" class="tombol polos">Ubah</RouterLink></td>
          </tr>
        </tbody>
      </table>
      <p v-if="!memuat && !data.length" class="kosong">Belum ada produk yang cocok. Tambahkan produk baru atau ubah saringan.</p>
    </div>
  </div>
</template>

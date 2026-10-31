<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { api, pesanError } from '@/api'
import { rupiah } from '@/format'
import type { Kategori, Produk } from '@/types'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiLencana from '@/components/ui/UiLencana.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTabel from '@/components/ui/UiTabel.vue'
import UiTombol from '@/components/ui/UiTombol.vue'

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
  <UiHalaman>
    <UiKepalaHalaman judul="Produk">
      <template #deskripsi>Model pakaian beserta varian ukuran dan warnanya. Stok dicatat per varian.</template>
      <template #aksi><UiTombol to="/admin/produk/baru">Tambah produk</UiTombol></template>
    </UiKepalaHalaman>

    <UiPesan v-if="pesan" jenis="berhasil">{{ pesan }}</UiPesan>

    <div class="flex flex-wrap items-end gap-3 [&>label]:flex-[1_1_160px]">
      <UiIsian label="Cari"><input v-model="cari" type="search" placeholder="Nama, kode, SKU, warna" /></UiIsian>
      <UiIsian label="Kategori">
        <select v-model="idKategori">
          <option value="">Semua kategori</option>
          <option v-for="k in kategori" :key="k.id_kategori" :value="k.id_kategori">{{ k.nama_kategori }}</option>
        </select>
      </UiIsian>
      <UiIsian label="Status">
        <select v-model="status">
          <option value="aktif">Dijual</option>
          <option value="nonaktif">Nonaktif</option>
          <option value="semua">Semua</option>
        </select>
      </UiIsian>
    </div>

    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <UiTabel :kosong="!memuat && !data.length ? 'Belum ada produk yang cocok. Tambahkan produk baru atau ubah saringan.' : null">
      <thead>
        <tr>
          <th>Produk</th>
          <th>Kategori</th>
          <th class="text-right">Harga beli</th>
          <th class="text-right">Harga jual</th>
          <th class="text-right">Varian</th>
          <th class="text-right">Stok</th>
          <th>Keterangan</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="p in data" :key="p.id_produk">
          <td>
            <strong>{{ p.nama }}</strong><br />
            <span class="text-sm text-redup">{{ p.kode }}</span>
          </td>
          <td>{{ p.nama_kategori }}</td>
          <td class="text-right tabular-nums">{{ rupiah(p.harga_beli ?? 0) }}</td>
          <td class="text-right tabular-nums">{{ rupiah(p.harga_jual) }}</td>
          <td class="text-right tabular-nums">{{ p.varian.filter((v) => v.aktif).length }}</td>
          <td class="text-right tabular-nums">{{ totalStok(p) }}</td>
          <td>
            <UiLencana v-if="!p.aktif" warna="netral">Nonaktif</UiLencana>
            <UiLencana v-else-if="menipis(p)" warna="waspada">{{ menipis(p) }} varian menipis</UiLencana>
          </td>
          <td class="text-right"><UiTombol :to="`/admin/produk/${p.id_produk}`" varian="polos">Ubah</UiTombol></td>
        </tr>
      </tbody>
    </UiTabel>
  </UiHalaman>
</template>

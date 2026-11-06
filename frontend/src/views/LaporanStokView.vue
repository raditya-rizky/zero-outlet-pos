<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { api, pesanError } from '@/api'
import { ribuan, rupiah } from '@/format'
import NavLaporan from '@/components/NavLaporan.vue'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTabel from '@/components/ui/UiTabel.vue'
import UiTombol from '@/components/ui/UiTombol.vue'
import type { LaporanStok } from '@/types'

// Laporan stok dan nilai persediaan per varian (KF-15d), dengan ekspor (KF-16).
const data = ref<LaporanStok | null>(null)
const galat = ref<string | null>(null)
const cari = ref('')

onMounted(async () => {
  try {
    data.value = await api.get<LaporanStok>('/laporan/stok')
  } catch (e) {
    galat.value = pesanError(e)
  }
})

const baris = computed(() => {
  const kata = cari.value.trim().toLowerCase()
  const semua = data.value?.varian ?? []
  return kata ? semua.filter((v) => `${v.nama_produk} ${v.kode} ${v.kategori} ${v.sku} ${v.warna}`.toLowerCase().includes(kata)) : semua
})

async function unduhCsv() {
  try {
    await api.unduh('/laporan/stok.csv')
  } catch (e) {
    galat.value = pesanError(e)
  }
}
const cetak = () => window.print()
</script>

<template>
  <UiHalaman>
    <UiKepalaHalaman judul="Laporan stok">
      <template #deskripsi>Stok saat ini dan nilai persediaan berdasarkan harga beli.</template>
      <template #aksi>
        <span class="flex flex-wrap gap-2.5 print:hidden">
          <UiTombol varian="kedua" :disabled="!data" @click="unduhCsv">Unduh CSV</UiTombol>
          <UiTombol varian="kedua" :disabled="!data" @click="cetak">Cetak / simpan PDF</UiTombol>
        </span>
      </template>
    </UiKepalaHalaman>
    <NavLaporan />
    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <template v-if="data">
      <dl class="flex flex-wrap gap-x-10 gap-y-2 [&_dd]:text-2xl [&_dd]:font-bold [&_dd]:tabular-nums [&_dt]:text-sm [&_dt]:text-redup">
        <div><dt>Total unit</dt><dd>{{ ribuan(data.total_unit) }}</dd></div>
        <div><dt>Nilai persediaan</dt><dd>{{ rupiah(data.total_nilai) }}</dd></div>
      </dl>
      <UiIsian label="Cari" class="max-w-sm print:hidden"><input v-model="cari" type="search" placeholder="Nama, kode, SKU, warna" /></UiIsian>
      <UiTabel :kosong="baris.length ? null : 'Tidak ada varian yang cocok.'">
        <thead>
          <tr><th>Produk</th><th>Kategori</th><th>Varian</th><th class="text-right">Stok</th><th class="text-right">Harga beli</th><th class="text-right">Nilai</th></tr>
        </thead>
        <tbody>
          <tr v-for="v in baris" :key="v.id_varian">
            <td>{{ v.nama_produk }} <span class="text-sm text-redup">{{ v.kode }}</span></td>
            <td class="text-sm">{{ v.kategori }}</td>
            <td class="text-sm">{{ v.ukuran }} / {{ v.warna }}</td>
            <td class="text-right tabular-nums" :class="{ 'font-semibold text-waspada': v.stok <= v.stok_minimum }">{{ v.stok }}</td>
            <td class="text-right tabular-nums">{{ rupiah(v.harga_beli) }}</td>
            <td class="text-right tabular-nums">{{ rupiah(v.nilai) }}</td>
          </tr>
        </tbody>
      </UiTabel>
    </template>
  </UiHalaman>
</template>

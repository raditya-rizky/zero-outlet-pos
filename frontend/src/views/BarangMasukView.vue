<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { api, pesanError } from '@/api'
import { rupiah } from '@/format'
import NavStok from '@/components/NavStok.vue'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTabel from '@/components/ui/UiTabel.vue'
import UiTombol from '@/components/ui/UiTombol.vue'
import type { BarangMasuk } from '@/types'

// Daftar penerimaan barang dari pemasok (KF-11).
const route = useRoute()
const dari = ref('')
const sampai = ref('')
const data = ref<BarangMasuk[]>([])
const galat = ref<string | null>(null)
const memuat = ref(false)
const terbuka = ref<number | null>(null)
const pesan = typeof route.query.pesan === 'string' ? route.query.pesan : null
const peringatan = typeof route.query.peringatan === 'string' ? route.query.peringatan.split('\n') : []

async function muat() {
  memuat.value = true
  galat.value = null
  try {
    data.value = await api.get<BarangMasuk[]>('/barang-masuk', { dari: dari.value, sampai: sampai.value })
  } catch (e) {
    galat.value = pesanError(e)
  } finally {
    memuat.value = false
  }
}
onMounted(muat)
</script>

<template>
  <UiHalaman>
    <UiKepalaHalaman judul="Stok">
      <template #deskripsi>Catat barang yang datang dari pemasok. Stok bertambah dan harga beli mengikuti harga terbaru.</template>
      <template #aksi><UiTombol to="/admin/stok/masuk/baru">Catat barang masuk</UiTombol></template>
    </UiKepalaHalaman>
    <NavStok />

    <UiPesan v-if="pesan" jenis="berhasil">{{ pesan }}</UiPesan>
    <UiPesan v-for="p in peringatan" :key="p">{{ p }}</UiPesan>

    <form class="flex flex-wrap items-end gap-3 [&>label]:flex-[1_1_160px]" @submit.prevent="muat">
      <UiIsian label="Dari"><input v-model="dari" type="date" /></UiIsian>
      <UiIsian label="Sampai"><input v-model="sampai" type="date" /></UiIsian>
      <UiTombol :disabled="memuat">Tampilkan</UiTombol>
    </form>
    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <UiTabel :kosong="!memuat && !data.length ? 'Belum ada barang masuk yang dicatat.' : null">
      <thead>
        <tr><th>Tanggal</th><th>Referensi</th><th>Pemasok</th><th class="text-right">Unit</th><th class="text-right">Nilai</th><th>Dicatat</th><th></th></tr>
      </thead>
      <tbody>
        <template v-for="b in data" :key="b.id_barang_masuk">
          <tr>
            <td class="whitespace-nowrap">{{ b.tanggal }}</td>
            <td>BM-{{ b.id_barang_masuk }}<span v-if="b.no_referensi" class="text-redup"> / {{ b.no_referensi }}</span></td>
            <td>{{ b.nama_pemasok ?? 'Tanpa pemasok' }}</td>
            <td class="text-right tabular-nums">{{ b.jumlah_unit }}</td>
            <td class="text-right tabular-nums">{{ rupiah(b.total_nilai) }}</td>
            <td class="text-sm">{{ b.nama_pengguna }}</td>
            <td class="text-right">
              <UiTombol varian="polos" ukuran="kecil" @click="terbuka = terbuka === b.id_barang_masuk ? null : b.id_barang_masuk">
                {{ terbuka === b.id_barang_masuk ? 'Tutup' : 'Rincian' }}
              </UiTombol>
            </td>
          </tr>
          <tr v-if="terbuka === b.id_barang_masuk">
            <td colspan="7" class="bg-kertas">
              <ul class="grid gap-1 text-sm">
                <li v-for="i in b.items" :key="i.id_detail" class="flex justify-between gap-3">
                  <span>{{ i.jumlah }} x {{ i.nama_barang }} <span class="text-redup">{{ i.sku }}</span></span>
                  <span class="tabular-nums">@ {{ rupiah(i.harga_beli) }}</span>
                </li>
              </ul>
              <p v-if="b.keterangan" class="mt-2 text-sm text-redup">{{ b.keterangan }}</p>
            </td>
          </tr>
        </template>
      </tbody>
    </UiTabel>
  </UiHalaman>
</template>

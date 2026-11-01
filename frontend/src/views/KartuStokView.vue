<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api, pesanError } from '@/api'
import { waktu } from '@/format'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiLencana from '@/components/ui/UiLencana.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTabel from '@/components/ui/UiTabel.vue'
import UiTombol from '@/components/ui/UiTombol.vue'
import type { KartuStok, MutasiKartu } from '@/types'

// Kartu stok: riwayat mutasi satu varian (KF-13).
const props = defineProps<{ id: string }>()
const dari = ref('')
const sampai = ref('')
const kartu = ref<KartuStok | null>(null)
const galat = ref<string | null>(null)

const namaJenis: Record<MutasiKartu['jenis'], string> = {
  masuk: 'Masuk',
  keluar: 'Keluar',
  retur: 'Retur',
  penyesuaian: 'Penyesuaian',
}
const warnaJenis: Record<MutasiKartu['jenis'], 'aman' | 'denim' | 'waspada' | 'netral'> = {
  masuk: 'aman',
  keluar: 'netral',
  retur: 'denim',
  penyesuaian: 'waspada',
}

async function muat() {
  galat.value = null
  try {
    kartu.value = await api.get<KartuStok>(`/varian/${props.id}/kartu-stok`, { dari: dari.value, sampai: sampai.value })
  } catch (e) {
    galat.value = pesanError(e)
  }
}
onMounted(muat)
</script>

<template>
  <UiHalaman>
    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>
    <template v-if="kartu">
      <UiKepalaHalaman :judul="`Kartu stok ${kartu.nama_produk} ${kartu.ukuran}/${kartu.warna}`">
        <template #atas><RouterLink :to="`/admin/produk/${kartu.id_produk}`" class="text-sm">Ke data produk</RouterLink></template>
        <template #deskripsi>
          {{ kartu.sku }}. Stok sekarang {{ kartu.stok }}, minimum {{ kartu.stok_minimum }}.
        </template>
      </UiKepalaHalaman>

      <form class="flex flex-wrap items-end gap-3 [&>label]:flex-[1_1_160px]" @submit.prevent="muat">
        <UiIsian label="Dari"><input v-model="dari" type="date" /></UiIsian>
        <UiIsian label="Sampai"><input v-model="sampai" type="date" /></UiIsian>
        <UiTombol>Tampilkan</UiTombol>
      </form>

      <UiTabel :kosong="kartu.mutasi.length ? null : 'Belum ada mutasi pada rentang ini.'">
        <thead>
          <tr><th>Waktu</th><th>Jenis</th><th>Referensi</th><th>Keterangan</th><th>Oleh</th><th class="text-right">Jumlah</th><th class="text-right">Stok</th></tr>
        </thead>
        <tbody>
          <tr v-if="dari">
            <td colspan="6" class="text-redup">Saldo sebelum {{ dari }}</td>
            <td class="text-right font-semibold tabular-nums">{{ kartu.saldo_awal }}</td>
          </tr>
          <tr v-for="m in kartu.mutasi" :key="m.id_mutasi">
            <td class="whitespace-nowrap">{{ waktu(m.tanggal) }}</td>
            <td><UiLencana :warna="warnaJenis[m.jenis]">{{ namaJenis[m.jenis] }}</UiLencana></td>
            <td class="text-sm">{{ m.referensi }}</td>
            <td class="text-sm text-redup">{{ m.keterangan }}</td>
            <td class="text-sm">{{ m.nama_pengguna }}</td>
            <td class="text-right tabular-nums" :class="m.jumlah < 0 ? 'text-bahaya' : 'text-aman'">{{ m.jumlah > 0 ? '+' : '' }}{{ m.jumlah }}</td>
            <td class="text-right font-semibold tabular-nums">{{ m.stok_sesudah }}</td>
          </tr>
        </tbody>
      </UiTabel>
    </template>
  </UiHalaman>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api, pesanError } from '@/api'
import { hariIni, rupiah, waktu } from '@/format'
import type { Pengguna, Sesi } from '@/types'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiLencana from '@/components/ui/UiLencana.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTabel from '@/components/ui/UiTabel.vue'
import UiTombol from '@/components/ui/UiTombol.vue'

const d = new Date()
d.setDate(d.getDate() - 6)
const dari = ref(d.toISOString().slice(0, 10))
const sampai = ref(hariIni())
const idKasir = ref('')
const kasir = ref<Pengguna[]>([])
const data = ref<Sesi[]>([])
const galat = ref<string | null>(null)

async function muat() {
  galat.value = null
  try {
    data.value = await api.get<Sesi[]>('/sesi', { dari: dari.value, sampai: sampai.value, id_pengguna: idKasir.value })
  } catch (e) {
    galat.value = pesanError(e)
  }
}

function kelasSelisih(n: number | null): 'netral' | 'aman' | 'bahaya' {
  return n === null ? 'netral' : n === 0 ? 'aman' : 'bahaya'
}

onMounted(async () => {
  muat()
  kasir.value = await api.get<Pengguna[]>('/pengguna').catch(() => [])
})
</script>

<template>
  <UiHalaman>
    <UiKepalaHalaman judul="Sesi kasir">
      <template #deskripsi>Rekonsiliasi kas setiap shift. Tinjau sesi dengan selisih beserta alasannya.</template>
    </UiKepalaHalaman>

    <form class="flex flex-wrap items-end gap-3 [&>label]:flex-[1_1_160px]" @submit.prevent="muat">
      <UiIsian label="Dari"><input v-model="dari" type="date" /></UiIsian>
      <UiIsian label="Sampai"><input v-model="sampai" type="date" /></UiIsian>
      <UiIsian label="Kasir">
        <select v-model="idKasir">
          <option value="">Semua kasir</option>
          <option v-for="k in kasir" :key="k.id_pengguna" :value="k.id_pengguna">{{ k.nama }}</option>
        </select>
      </UiIsian>
      <UiTombol>Tampilkan</UiTombol>
    </form>

    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <UiTabel :kosong="!data.length ? 'Tidak ada sesi kasir pada rentang tanggal ini.' : null">
      <thead>
        <tr>
          <th>Kasir</th>
          <th>Dibuka</th>
          <th>Ditutup</th>
          <th class="text-right">Modal awal</th>
          <th class="text-right">Kas seharusnya</th>
          <th class="text-right">Kas fisik</th>
          <th>Selisih</th>
          <th>Alasan</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="s in data" :key="s.id_sesi">
          <td>{{ s.nama_kasir }}</td>
          <td>{{ waktu(s.waktu_buka) }}</td>
          <td>
            <span v-if="s.waktu_tutup">{{ waktu(s.waktu_tutup) }}</span>
            <UiLencana v-else>Masih dibuka</UiLencana>
          </td>
          <td class="text-right tabular-nums">{{ rupiah(s.modal_awal) }}</td>
          <td class="text-right tabular-nums">{{ s.kas_seharusnya === null ? '' : rupiah(s.kas_seharusnya) }}</td>
          <td class="text-right tabular-nums">{{ s.kas_fisik === null ? '' : rupiah(s.kas_fisik) }}</td>
          <td>
            <UiLencana v-if="s.selisih !== null" :warna="kelasSelisih(s.selisih)" class="tabular-nums">
              {{ s.selisih === 0 ? 'Sesuai' : (s.selisih > 0 ? '+' : '−') + rupiah(Math.abs(s.selisih)) }}
            </UiLencana>
          </td>
          <td>{{ s.alasan_selisih }}</td>
        </tr>
      </tbody>
    </UiTabel>
  </UiHalaman>
</template>

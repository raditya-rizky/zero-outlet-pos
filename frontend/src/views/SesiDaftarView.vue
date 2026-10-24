<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api, pesanError } from '@/api'
import { hariIni, rupiah, waktu } from '@/format'
import type { Pengguna, Sesi } from '@/types'

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

function kelasSelisih(n: number | null) {
  return n === null ? 'netral' : n === 0 ? 'aman' : 'bahaya'
}

onMounted(async () => {
  muat()
  kasir.value = await api.get<Pengguna[]>('/pengguna').catch(() => [])
})
</script>

<template>
  <div class="halaman">
    <div>
      <h1>Sesi kasir</h1>
      <p class="redup">Rekonsiliasi kas setiap shift. Tinjau sesi dengan selisih beserta alasannya.</p>
    </div>

    <form class="saring" @submit.prevent="muat">
      <label class="isian">Dari <input v-model="dari" type="date" /></label>
      <label class="isian">Sampai <input v-model="sampai" type="date" /></label>
      <label class="isian">
        Kasir
        <select v-model="idKasir">
          <option value="">Semua kasir</option>
          <option v-for="k in kasir" :key="k.id_pengguna" :value="k.id_pengguna">{{ k.nama }}</option>
        </select>
      </label>
      <button class="tombol">Tampilkan</button>
    </form>

    <p v-if="galat" class="pesan galat">{{ galat }}</p>

    <div class="tabel-bungkus">
      <table>
        <thead>
          <tr>
            <th>Kasir</th>
            <th>Dibuka</th>
            <th>Ditutup</th>
            <th class="kanan">Modal awal</th>
            <th class="kanan">Kas seharusnya</th>
            <th class="kanan">Kas fisik</th>
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
              <span v-else class="lencana">Masih dibuka</span>
            </td>
            <td class="kanan angka">{{ rupiah(s.modal_awal) }}</td>
            <td class="kanan angka">{{ s.kas_seharusnya === null ? '' : rupiah(s.kas_seharusnya) }}</td>
            <td class="kanan angka">{{ s.kas_fisik === null ? '' : rupiah(s.kas_fisik) }}</td>
            <td>
              <span v-if="s.selisih !== null" class="lencana angka" :class="kelasSelisih(s.selisih)">
                {{ s.selisih === 0 ? 'Sesuai' : (s.selisih > 0 ? '+' : '−') + rupiah(Math.abs(s.selisih)) }}
              </span>
            </td>
            <td>{{ s.alasan_selisih }}</td>
          </tr>
        </tbody>
      </table>
      <p v-if="!data.length" class="kosong">Tidak ada sesi kasir pada rentang tanggal ini.</p>
    </div>
  </div>
</template>

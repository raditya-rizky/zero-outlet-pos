<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { api, pesanError } from '@/api'
import { waktu } from '@/format'
import type { HalamanLog, LogAktivitas, Pengguna } from '@/types'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiLencana from '@/components/ui/UiLencana.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTabel from '@/components/ui/UiTabel.vue'
import UiTombol from '@/components/ui/UiTombol.vue'

const AKSI: Record<string, string> = {
  login: 'Login',
  login_gagal: 'Login gagal',
  logout: 'Logout',
  transaksi: 'Transaksi',
  cetak_ulang: 'Cetak ulang struk',
  buka_sesi: 'Buka sesi',
  tutup_sesi: 'Tutup sesi',
  tambah: 'Tambah data',
  ubah: 'Ubah data',
  hapus: 'Hapus data',
  nonaktifkan: 'Nonaktifkan',
  aktifkan: 'Aktifkan',
}

const idPengguna = ref('')
const aksi = ref('')
const dari = ref('')
const sampai = ref('')
const halaman = ref(1)
const hasil = ref<HalamanLog | null>(null)
const pengguna = ref<Pengguna[]>([])
const galat = ref<string | null>(null)

const jumlahHalaman = computed(() => (hasil.value ? Math.max(1, Math.ceil(hasil.value.total / hasil.value.per_halaman)) : 1))

async function muat(ke = 1) {
  galat.value = null
  halaman.value = ke
  try {
    hasil.value = await api.get<HalamanLog>('/log-aktivitas', {
      id_pengguna: idPengguna.value,
      aksi: aksi.value,
      dari: dari.value,
      sampai: sampai.value,
      halaman: ke,
    })
  } catch (e) {
    galat.value = pesanError(e)
  }
}

type Nilai = Record<string, unknown>
const objek = (v: unknown): v is Nilai => typeof v === 'object' && v !== null && !Array.isArray(v)
const tampil = (v: unknown) => (typeof v === 'string' ? v : JSON.stringify(v))

/** Daftar kolom yang berubah antara nilai lama dan nilai baru. */
function perubahan(l: LogAktivitas): { kolom: string; lama?: string; baru?: string }[] {
  const lama = objek(l.nilai_lama) ? l.nilai_lama : {}
  const baru = objek(l.nilai_baru) ? l.nilai_baru : {}
  const kolom = new Set([...Object.keys(lama), ...Object.keys(baru)])
  return [...kolom]
    .filter((k) => JSON.stringify(lama[k]) !== JSON.stringify(baru[k]))
    .map((k) => ({
      kolom: k,
      lama: k in lama ? tampil(lama[k]) : undefined,
      baru: k in baru ? tampil(baru[k]) : undefined,
    }))
}

onMounted(async () => {
  muat()
  pengguna.value = await api.get<Pengguna[]>('/pengguna').catch(() => [])
})
</script>

<template>
  <UiHalaman>
    <UiKepalaHalaman judul="Log aktivitas">
      <template #deskripsi>Jejak siapa melakukan apa dan kapan. Log hanya dapat dibaca; tidak ada yang dapat mengubah atau menghapusnya.</template>
    </UiKepalaHalaman>

    <form class="flex flex-wrap items-end gap-3 [&>label]:flex-[1_1_160px]" @submit.prevent="muat(1)">
      <UiIsian label="Pengguna">
        <select v-model="idPengguna">
          <option value="">Semua pengguna</option>
          <option v-for="p in pengguna" :key="p.id_pengguna" :value="p.id_pengguna">{{ p.nama }}</option>
        </select>
      </UiIsian>
      <UiIsian label="Aktivitas">
        <select v-model="aksi">
          <option value="">Semua aktivitas</option>
          <option v-for="(label, kunci) in AKSI" :key="kunci" :value="kunci">{{ label }}</option>
        </select>
      </UiIsian>
      <UiIsian label="Dari"><input v-model="dari" type="date" /></UiIsian>
      <UiIsian label="Sampai"><input v-model="sampai" type="date" /></UiIsian>
      <UiTombol>Tampilkan</UiTombol>
    </form>

    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <UiTabel v-if="hasil" :kosong="!hasil.data.length ? 'Tidak ada aktivitas yang cocok dengan saringan.' : null">
      <thead>
        <tr>
          <th>Waktu</th>
          <th>Pengguna</th>
          <th>Aktivitas</th>
          <th>Data</th>
          <th>Rincian</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="l in hasil.data" :key="l.id_log">
          <td class="whitespace-nowrap">{{ waktu(l.waktu) }}</td>
          <td>{{ l.nama_pengguna ?? 'Tidak dikenal' }}</td>
          <td>
            <UiLencana :warna="l.aksi === 'login_gagal' ? 'bahaya' : 'denim'">{{ AKSI[l.aksi] ?? l.aksi }}</UiLencana>
          </td>
          <td class="whitespace-nowrap">{{ l.objek }}<span v-if="l.id_objek" class="text-redup"> #{{ l.id_objek }}</span></td>
          <td class="max-w-[480px] text-[0.9rem]">
            <p v-if="l.keterangan">{{ l.keterangan }}</p>
            <details v-if="perubahan(l).length">
              <summary class="cursor-pointer text-denim">{{ perubahan(l).length }} kolom {{ l.nilai_lama ? 'berubah' : 'tercatat' }}</summary>
              <ul class="mt-1.5 list-disc pl-[18px] break-words">
                <li v-for="c in perubahan(l)" :key="c.kolom">
                  <strong>{{ c.kolom }}</strong>:
                  <template v-if="c.lama !== undefined"><del class="text-bahaya">{{ c.lama }}</del> menjadi </template>
                  <span>{{ c.baru ?? '(dihapus)' }}</span>
                </li>
              </ul>
            </details>
          </td>
        </tr>
      </tbody>
    </UiTabel>

    <nav v-if="hasil && jumlahHalaman > 1" class="flex items-center justify-center gap-4" aria-label="Halaman log">
      <UiTombol varian="kedua" :disabled="halaman <= 1" @click="muat(halaman - 1)">Sebelumnya</UiTombol>
      <span class="text-redup">Halaman {{ halaman }} dari {{ jumlahHalaman }}</span>
      <UiTombol varian="kedua" :disabled="halaman >= jumlahHalaman" @click="muat(halaman + 1)">Berikutnya</UiTombol>
    </nav>
  </UiHalaman>
</template>

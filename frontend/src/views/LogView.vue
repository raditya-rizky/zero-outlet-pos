<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { api, pesanError } from '@/api'
import { waktu } from '@/format'
import type { HalamanLog, LogAktivitas, Pengguna } from '@/types'

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
  <div class="halaman">
    <div>
      <h1>Log aktivitas</h1>
      <p class="redup">Jejak siapa melakukan apa dan kapan. Log hanya dapat dibaca; tidak ada yang dapat mengubah atau menghapusnya.</p>
    </div>

    <form class="saring" @submit.prevent="muat(1)">
      <label class="isian">
        Pengguna
        <select v-model="idPengguna">
          <option value="">Semua pengguna</option>
          <option v-for="p in pengguna" :key="p.id_pengguna" :value="p.id_pengguna">{{ p.nama }}</option>
        </select>
      </label>
      <label class="isian">
        Aktivitas
        <select v-model="aksi">
          <option value="">Semua aktivitas</option>
          <option v-for="(label, kunci) in AKSI" :key="kunci" :value="kunci">{{ label }}</option>
        </select>
      </label>
      <label class="isian">Dari <input v-model="dari" type="date" /></label>
      <label class="isian">Sampai <input v-model="sampai" type="date" /></label>
      <button class="tombol">Tampilkan</button>
    </form>

    <p v-if="galat" class="pesan galat">{{ galat }}</p>

    <div v-if="hasil" class="tabel-bungkus">
      <table>
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
            <td class="nowrap">{{ waktu(l.waktu) }}</td>
            <td>{{ l.nama_pengguna ?? 'Tidak dikenal' }}</td>
            <td>
              <span class="lencana" :class="{ bahaya: l.aksi === 'login_gagal' }">{{ AKSI[l.aksi] ?? l.aksi }}</span>
            </td>
            <td class="nowrap">{{ l.objek }}<span v-if="l.id_objek" class="redup"> #{{ l.id_objek }}</span></td>
            <td class="rincian">
              <p v-if="l.keterangan">{{ l.keterangan }}</p>
              <details v-if="perubahan(l).length">
                <summary>{{ perubahan(l).length }} kolom {{ l.nilai_lama ? 'berubah' : 'tercatat' }}</summary>
                <ul>
                  <li v-for="c in perubahan(l)" :key="c.kolom">
                    <strong>{{ c.kolom }}</strong>:
                    <template v-if="c.lama !== undefined"><del>{{ c.lama }}</del> menjadi </template>
                    <span>{{ c.baru ?? '(dihapus)' }}</span>
                  </li>
                </ul>
              </details>
            </td>
          </tr>
        </tbody>
      </table>
      <p v-if="!hasil.data.length" class="kosong">Tidak ada aktivitas yang cocok dengan saringan.</p>
    </div>

    <nav v-if="hasil && jumlahHalaman > 1" class="halaman-nav" aria-label="Halaman log">
      <button class="tombol kedua" :disabled="halaman <= 1" @click="muat(halaman - 1)">Sebelumnya</button>
      <span class="redup">Halaman {{ halaman }} dari {{ jumlahHalaman }}</span>
      <button class="tombol kedua" :disabled="halaman >= jumlahHalaman" @click="muat(halaman + 1)">Berikutnya</button>
    </nav>
  </div>
</template>

<style scoped>
.nowrap {
  white-space: nowrap;
}
.rincian {
  max-width: 480px;
  font-size: 0.9rem;
}
.rincian ul {
  margin: 6px 0 0;
  padding-left: 18px;
  word-break: break-word;
}
.rincian del {
  color: var(--bahaya);
}
summary {
  cursor: pointer;
  color: var(--denim);
}
.halaman-nav {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
}
</style>

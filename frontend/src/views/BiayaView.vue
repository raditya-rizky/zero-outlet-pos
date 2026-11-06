<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { api, pesanError } from '@/api'
import { hariIni, parseRupiah, ribuan, rupiah, tanggalSaja } from '@/format'
import { rentangCepat, type Periode } from '@/periode'
import DialogModal from '@/components/DialogModal.vue'
import NavLaporan from '@/components/NavLaporan.vue'
import PilihPeriode from '@/components/PilihPeriode.vue'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTabel from '@/components/ui/UiTabel.vue'
import UiTombol from '@/components/ui/UiTombol.vue'
import { KATEGORI_BIAYA, type Biaya } from '@/types'

// Biaya operasional untuk laporan laba rugi (KF-17), tanpa penggajian.
const periode = ref<Periode>(rentangCepat('bulan'))
const data = ref<Biaya[]>([])
const galat = ref<string | null>(null)
const pesan = ref<string | null>(null)

interface Formulir {
  id: number | null
  tanggal: string
  kategori: string
  nominal: string
  keterangan: string
}
const formulir = ref<Formulir | null>(null)
const galatForm = ref<string | null>(null)
const menyimpan = ref(false)

async function muat() {
  galat.value = null
  try {
    data.value = await api.get<Biaya[]>('/biaya', { ...periode.value })
  } catch (e) {
    galat.value = pesanError(e)
  }
}
watch(periode, muat, { immediate: true })

const total = computed(() => data.value.reduce((n, b) => n + b.nominal, 0))
const perKategori = computed(() =>
  KATEGORI_BIAYA.map((k) => ({ kategori: k, nominal: data.value.filter((b) => b.kategori === k).reduce((n, b) => n + b.nominal, 0) })).filter(
    (k) => k.nominal > 0,
  ),
)

function tambah() {
  galatForm.value = null
  formulir.value = { id: null, tanggal: hariIni(), kategori: 'Sewa tempat', nominal: '', keterangan: '' }
}
function ubah(b: Biaya) {
  galatForm.value = null
  formulir.value = { id: b.id_biaya, tanggal: b.tanggal, kategori: b.kategori, nominal: ribuan(b.nominal), keterangan: b.keterangan ?? '' }
}

async function simpan() {
  const f = formulir.value
  if (!f) return
  menyimpan.value = true
  galatForm.value = null
  const body = { tanggal: f.tanggal, kategori: f.kategori, nominal: parseRupiah(f.nominal), keterangan: f.keterangan }
  try {
    if (f.id) await api.put(`/biaya/${f.id}`, body)
    else await api.post('/biaya', body)
    pesan.value = `Biaya ${f.kategori.toLowerCase()} ${rupiah(body.nominal)} disimpan.`
    formulir.value = null
    await muat()
  } catch (e) {
    galatForm.value = pesanError(e)
  } finally {
    menyimpan.value = false
  }
}

async function hapus(b: Biaya) {
  if (!confirm(`Hapus biaya ${b.kategori} ${rupiah(b.nominal)} tanggal ${tanggalSaja(b.tanggal)}?`)) return
  try {
    await api.del(`/biaya/${b.id_biaya}`)
    pesan.value = 'Biaya dihapus.'
    await muat()
  } catch (e) {
    galat.value = pesanError(e)
  }
}
</script>

<template>
  <UiHalaman>
    <UiKepalaHalaman judul="Biaya operasional">
      <template #deskripsi>Sewa, listrik, upah, dan pengeluaran rutin lain. Dipakai pada laporan laba rugi.</template>
      <template #aksi><UiTombol @click="tambah">Catat biaya</UiTombol></template>
    </UiKepalaHalaman>
    <NavLaporan />
    <PilihPeriode v-model="periode" />
    <UiPesan v-if="pesan" jenis="berhasil">{{ pesan }}</UiPesan>
    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <section v-if="perKategori.length" class="grid gap-2">
      <p class="text-sm text-redup">Total periode ini</p>
      <p class="text-2xl font-bold tabular-nums">{{ rupiah(total) }}</p>
      <ul class="flex flex-wrap gap-x-6 gap-y-1 text-sm">
        <li v-for="k in perKategori" :key="k.kategori">{{ k.kategori }} <span class="tabular-nums text-redup">{{ rupiah(k.nominal) }}</span></li>
      </ul>
    </section>

    <UiTabel :kosong="data.length ? null : 'Belum ada biaya pada periode ini.'">
      <thead><tr><th>Tanggal</th><th>Kategori</th><th>Keterangan</th><th class="text-right">Nominal</th><th>Dicatat</th><th></th></tr></thead>
      <tbody>
        <tr v-for="b in data" :key="b.id_biaya">
          <td class="whitespace-nowrap">{{ tanggalSaja(b.tanggal) }}</td>
          <td>{{ b.kategori }}</td>
          <td class="text-sm text-redup">{{ b.keterangan }}</td>
          <td class="text-right tabular-nums">{{ rupiah(b.nominal) }}</td>
          <td class="text-sm">{{ b.nama_pengguna }}</td>
          <td class="py-1 text-right whitespace-nowrap">
            <UiTombol varian="polos" ukuran="kecil" @click="ubah(b)">Ubah</UiTombol>
            <UiTombol varian="polos" ukuran="kecil" @click="hapus(b)">Hapus</UiTombol>
          </td>
        </tr>
      </tbody>
    </UiTabel>

    <DialogModal v-if="formulir" :judul="formulir.id ? 'Ubah biaya' : 'Catat biaya'" @tutup="formulir = null">
      <form class="grid gap-3.5" @submit.prevent="simpan">
        <UiPesan v-if="galatForm" jenis="galat">{{ galatForm }}</UiPesan>
        <UiIsian label="Tanggal"><input v-model="formulir.tanggal" type="date" required /></UiIsian>
        <UiIsian label="Kategori">
          <select v-model="formulir.kategori">
            <option v-for="k in KATEGORI_BIAYA" :key="k" :value="k">{{ k }}</option>
          </select>
        </UiIsian>
        <UiIsian label="Nominal (Rp)">
          <input v-model="formulir.nominal" inputmode="numeric" required class="tabular-nums" @blur="formulir.nominal = formulir.nominal.trim() ? ribuan(parseRupiah(formulir.nominal)) : ''" />
        </UiIsian>
        <UiIsian label="Keterangan" :petunjuk="formulir.kategori === 'Lain-lain' ? 'Wajib untuk kategori Lain-lain.' : 'Opsional.'">
          <input v-model="formulir.keterangan" :required="formulir.kategori === 'Lain-lain'" />
        </UiIsian>
        <UiTombol :disabled="menyimpan">Simpan biaya</UiTombol>
      </form>
    </DialogModal>
  </UiHalaman>
</template>

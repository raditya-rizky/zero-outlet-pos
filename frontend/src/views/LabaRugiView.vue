<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { api, pesanError } from '@/api'
import { perubahan, persenBps, rupiah, tanggalSaja } from '@/format'
import { rentangCepat, type Periode } from '@/periode'
import NavLaporan from '@/components/NavLaporan.vue'
import PilihPeriode from '@/components/PilihPeriode.vue'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTombol from '@/components/ui/UiTombol.vue'
import type { LabaRugi, PeriodeLabaRugi } from '@/types'

// Laporan laba rugi sederhana dengan pembanding periode sebelumnya (KF-18).
const periode = ref<Periode>(rentangCepat('bulan_lalu'))
const data = ref<LabaRugi | null>(null)
const galat = ref<string | null>(null)

async function muat() {
  galat.value = null
  try {
    data.value = await api.get<LabaRugi>('/laporan/laba-rugi', { ...periode.value })
  } catch (e) {
    galat.value = pesanError(e)
  }
}
watch(periode, muat, { immediate: true })

async function unduhCsv() {
  try {
    await api.unduh('/laporan/laba-rugi.csv', { ...periode.value })
  } catch (e) {
    galat.value = pesanError(e)
  }
}
const cetak = () => window.print()

const label = (p: PeriodeLabaRugi) => (p.dari === p.sampai ? tanggalSaja(p.dari) : `${tanggalSaja(p.dari)} sampai ${tanggalSaja(p.sampai)}`)

type Pos = { nama: string; kunci: keyof PeriodeLabaRugi; tanda?: '−' | '='; tebal?: boolean }
const pos: Pos[] = [
  { nama: 'Penjualan kotor', kunci: 'kotor' },
  { nama: 'Diskon', kunci: 'diskon', tanda: '−' },
  { nama: 'Retur', kunci: 'retur', tanda: '−' },
  { nama: 'Penjualan bersih', kunci: 'bersih', tanda: '=', tebal: true },
  { nama: 'Harga pokok penjualan', kunci: 'hpp', tanda: '−' },
  { nama: 'Laba kotor', kunci: 'laba_kotor', tanda: '=', tebal: true },
  { nama: 'Biaya operasional', kunci: 'biaya', tanda: '−' },
  { nama: 'Laba bersih', kunci: 'laba_bersih', tanda: '=', tebal: true },
]
const nilai = (p: PeriodeLabaRugi, k: keyof PeriodeLabaRugi) => p[k] as number
const kategoriBiaya = computed(() => {
  if (!data.value) return []
  const semua = new Set([...data.value.sekarang.biaya_per_kategori, ...data.value.sebelumnya.biaya_per_kategori].map((b) => b.kategori))
  const cari = (p: PeriodeLabaRugi, k: string) => p.biaya_per_kategori.find((b) => b.kategori === k)?.nominal ?? 0
  return [...semua].map((k) => ({ kategori: k, sekarang: cari(data.value!.sekarang, k), sebelumnya: cari(data.value!.sebelumnya, k) }))
})
</script>

<template>
  <UiHalaman>
    <UiKepalaHalaman judul="Laba rugi">
      <template #deskripsi>{{ data ? label(data.sekarang) : '' }}, dibandingkan periode sebelumnya.</template>
      <template #aksi>
        <span class="flex flex-wrap gap-2.5 print:hidden">
          <UiTombol varian="kedua" :disabled="!data" @click="unduhCsv">Unduh CSV</UiTombol>
          <UiTombol varian="kedua" :disabled="!data" @click="cetak">Cetak / simpan PDF</UiTombol>
        </span>
      </template>
    </UiKepalaHalaman>
    <NavLaporan />
    <PilihPeriode v-model="periode" awal="bulan_lalu" />
    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <template v-if="data">
      <div class="overflow-x-auto">
        <table class="w-full border-collapse text-[0.95rem] [&_td:first-child]:pl-0 [&_th:first-child]:pl-0">
          <thead>
            <tr>
              <th>Pos</th>
              <th class="text-right">{{ label(data.sekarang) }}</th>
              <th class="text-right">{{ label(data.sebelumnya) }}</th>
              <th class="text-right">Perubahan</th>
            </tr>
          </thead>
          <tbody>
            <template v-for="p in pos" :key="p.kunci">
              <tr :class="{ 'font-semibold': p.tebal }">
                <td>
                  <span class="inline-block w-4 text-redup" aria-hidden="true">{{ p.tanda }}</span>{{ p.nama }}
                </td>
                <td
                  class="text-right tabular-nums"
                  :class="{ 'text-bahaya': p.kunci.startsWith('laba') && nilai(data.sekarang, p.kunci) < 0 }"
                >
                  {{ rupiah(nilai(data.sekarang, p.kunci)) }}
                </td>
                <td class="text-right text-redup tabular-nums">{{ rupiah(nilai(data.sebelumnya, p.kunci)) }}</td>
                <td class="text-right text-sm text-redup tabular-nums">
                  {{ perubahan(nilai(data.sekarang, p.kunci), nilai(data.sebelumnya, p.kunci)) ?? '' }}
                </td>
              </tr>
              <template v-if="p.kunci === 'biaya'">
                <tr v-for="b in kategoriBiaya" :key="b.kategori" class="text-sm text-redup">
                  <td class="pl-6!">{{ b.kategori }}</td>
                  <td class="text-right tabular-nums">{{ rupiah(b.sekarang) }}</td>
                  <td class="text-right tabular-nums">{{ rupiah(b.sebelumnya) }}</td>
                  <td></td>
                </tr>
              </template>
            </template>
            <tr>
              <td>Margin laba kotor</td>
              <td class="text-right tabular-nums">{{ persenBps(data.sekarang.margin_kotor_bps) }}</td>
              <td class="text-right text-redup tabular-nums">{{ persenBps(data.sebelumnya.margin_kotor_bps) }}</td>
              <td></td>
            </tr>
            <tr>
              <td>Margin laba bersih</td>
              <td class="text-right tabular-nums">{{ persenBps(data.sekarang.margin_bersih_bps) }}</td>
              <td class="text-right text-redup tabular-nums">{{ persenBps(data.sebelumnya.margin_bersih_bps) }}</td>
              <td></td>
            </tr>
          </tbody>
        </table>
      </div>
      <p class="text-sm text-redup">
        Penjualan hanya dari transaksi selesai; transaksi batal tidak dihitung. Retur dihitung pada tanggal disetujui. Harga pokok memakai
        harga beli saat barang terjual, dikurangi barang yang diretur. Margin kosong bila penjualan bersih nol atau minus.
      </p>
      <p class="text-sm">
        <RouterLink to="/admin/laporan/biaya">Kelola biaya operasional</RouterLink>
      </p>
    </template>
  </UiHalaman>
</template>

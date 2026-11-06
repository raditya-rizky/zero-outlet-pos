<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { api, pesanError } from '@/api'
import { useAuth } from '@/stores/auth'
import { namaMetode, rupiah, ribuan, tanggalSaja, waktu } from '@/format'
import { rentangCepat, type Periode } from '@/periode'
import GrafikTren from '@/components/GrafikTren.vue'
import NavLaporan from '@/components/NavLaporan.vue'
import PilihPeriode from '@/components/PilihPeriode.vue'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTabel from '@/components/ui/UiTabel.vue'
import UiTombol from '@/components/ui/UiTombol.vue'
import type { BarisRincian, LaporanPenjualan } from '@/types'

// Laporan penjualan harian/mingguan/bulanan (KF-15) dan ekspornya (KF-16).
// Kasir hanya melihat penjualannya sendiri (Tabel 2).
const auth = useAuth()
const periode = ref<Periode>(rentangCepat('bulan'))
const data = ref<LaporanPenjualan | null>(null)
const galat = ref<string | null>(null)
const memuat = ref(false)

async function muat() {
  memuat.value = true
  galat.value = null
  try {
    data.value = await api.get<LaporanPenjualan>('/laporan/penjualan', { ...periode.value })
  } catch (e) {
    galat.value = pesanError(e)
  } finally {
    memuat.value = false
  }
}
watch(periode, muat, { immediate: true })

async function unduhCsv() {
  galat.value = null
  try {
    await api.unduh('/laporan/penjualan.csv', { ...periode.value })
  } catch (e) {
    galat.value = pesanError(e)
  }
}

const cetak = () => window.print()

const judulPeriode = computed(() =>
  periode.value.dari === periode.value.sampai
    ? tanggalSaja(periode.value.dari)
    : `${tanggalSaja(periode.value.dari)} sampai ${tanggalSaja(periode.value.sampai)}`,
)
const porsi = (b: BarisRincian) => {
  const total = data.value?.per_produk.reduce((n, p) => n + p.penjualan, 0) ?? 0
  return total > 0 ? `${Math.round((b.penjualan / total) * 100)}%` : ''
}
</script>

<template>
  <UiHalaman>
    <UiKepalaHalaman :judul="auth.isAdmin ? 'Laporan penjualan' : 'Laporan penjualan saya'">
      <template #deskripsi>{{ judulPeriode }}</template>
      <template #aksi>
        <span class="flex flex-wrap gap-2.5 print:hidden">
          <UiTombol v-if="auth.isAdmin" varian="kedua" :disabled="!data" @click="unduhCsv">Unduh CSV</UiTombol>
          <UiTombol varian="kedua" :disabled="!data" @click="cetak">Cetak / simpan PDF</UiTombol>
        </span>
      </template>
    </UiKepalaHalaman>
    <NavLaporan />
    <PilihPeriode v-model="periode" />
    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <template v-if="data">
      <!-- Angka utama: penjualan bersih menjadi satu-satunya angka besar. -->
      <section class="grid gap-6 border-t border-garis pt-6 md:grid-cols-[minmax(0,1.2fr)_minmax(0,2fr)]" :class="{ 'opacity-60': memuat }">
        <div>
          <p class="text-sm text-redup">Penjualan bersih</p>
          <p class="text-[2.4rem] leading-tight font-bold font-stretch-115% tabular-nums">{{ rupiah(data.ringkasan.bersih) }}</p>
          <p class="text-sm text-redup">Setelah diskon dan retur yang disetujui pada periode ini.</p>
        </div>
        <dl class="grid grid-cols-2 gap-x-6 gap-y-3 sm:grid-cols-3 [&_dd]:text-lg [&_dd]:font-semibold [&_dd]:tabular-nums [&_dt]:text-sm [&_dt]:text-redup">
          <div><dt>Transaksi</dt><dd>{{ ribuan(data.ringkasan.transaksi) }}</dd></div>
          <div><dt>Barang terjual</dt><dd>{{ ribuan(data.ringkasan.unit) }}</dd></div>
          <div><dt>Rata-rata per transaksi</dt><dd>{{ rupiah(data.ringkasan.rata_rata) }}</dd></div>
          <div><dt>Penjualan kotor</dt><dd>{{ rupiah(data.ringkasan.kotor) }}</dd></div>
          <div><dt>Diskon</dt><dd>{{ rupiah(data.ringkasan.diskon) }}</dd></div>
          <div><dt>Retur</dt><dd>{{ rupiah(data.ringkasan.retur) }}</dd></div>
        </dl>
      </section>

      <GrafikTren
        :titik="data.tren"
        :kelompok="data.kelompok"
        :judul="`Total penjualan per ${data.kelompok === 'bulan' ? 'bulan' : 'hari'}, setelah diskon dan sebelum retur`"
      />

      <div class="grid gap-8 lg:grid-cols-2">
        <section class="grid content-start gap-3">
          <h2>Terlaris</h2>
          <UiTabel :kosong="data.terlaris.length ? null : 'Belum ada penjualan pada periode ini.'">
            <thead><tr><th>Produk</th><th class="text-right">Terjual</th><th class="text-right">Penjualan</th></tr></thead>
            <tbody>
              <tr v-for="b in data.terlaris" :key="b.kunci">
                <td>{{ b.label }}</td>
                <td class="text-right tabular-nums">{{ b.unit }}</td>
                <td class="text-right tabular-nums">{{ rupiah(b.penjualan) }}</td>
              </tr>
            </tbody>
          </UiTabel>
        </section>

        <section class="grid content-start gap-3">
          <h2>Metode bayar</h2>
          <UiTabel>
            <thead><tr><th>Metode</th><th class="text-right">Transaksi</th><th class="text-right">Penjualan</th><th class="text-right">Retur</th></tr></thead>
            <tbody>
              <tr v-for="m in data.per_metode" :key="m.metode">
                <td>{{ namaMetode[m.metode] }}</td>
                <td class="text-right tabular-nums">{{ m.transaksi }}</td>
                <td class="text-right tabular-nums">{{ rupiah(m.penjualan) }}</td>
                <td class="text-right tabular-nums">{{ m.retur ? rupiah(m.retur) : '' }}</td>
              </tr>
            </tbody>
          </UiTabel>
        </section>

        <section class="grid content-start gap-3">
          <h2>Per kategori</h2>
          <UiTabel :kosong="data.per_kategori.length ? null : 'Belum ada penjualan.'">
            <thead><tr><th>Kategori</th><th class="text-right">Terjual</th><th class="text-right">Penjualan</th><th class="text-right">Porsi</th></tr></thead>
            <tbody>
              <tr v-for="b in data.per_kategori" :key="b.kunci">
                <td>{{ b.label }}</td>
                <td class="text-right tabular-nums">{{ b.unit }}</td>
                <td class="text-right tabular-nums">{{ rupiah(b.penjualan) }}</td>
                <td class="text-right text-redup tabular-nums">{{ porsi(b) }}</td>
              </tr>
            </tbody>
          </UiTabel>
        </section>

        <section class="grid content-start gap-3">
          <h2>Per ukuran</h2>
          <UiTabel :kosong="data.per_ukuran.length ? null : 'Belum ada penjualan.'">
            <thead><tr><th>Ukuran</th><th class="text-right">Terjual</th><th class="text-right">Penjualan</th><th class="text-right">Porsi</th></tr></thead>
            <tbody>
              <tr v-for="b in data.per_ukuran" :key="b.kunci">
                <td>{{ b.label }}</td>
                <td class="text-right tabular-nums">{{ b.unit }}</td>
                <td class="text-right tabular-nums">{{ rupiah(b.penjualan) }}</td>
                <td class="text-right text-redup tabular-nums">{{ porsi(b) }}</td>
              </tr>
            </tbody>
          </UiTabel>
        </section>

        <section v-if="data.per_kasir" class="grid content-start gap-3">
          <h2>Per kasir</h2>
          <UiTabel :kosong="data.per_kasir.length ? null : 'Belum ada penjualan.'">
            <thead><tr><th>Kasir</th><th class="text-right">Transaksi</th><th class="text-right">Penjualan</th><th class="text-right">Retur</th></tr></thead>
            <tbody>
              <tr v-for="k in data.per_kasir" :key="k.id_pengguna">
                <td>{{ k.nama }}</td>
                <td class="text-right tabular-nums">{{ k.transaksi }}</td>
                <td class="text-right tabular-nums">{{ rupiah(k.penjualan) }}</td>
                <td class="text-right tabular-nums">{{ k.retur ? rupiah(k.retur) : '' }}</td>
              </tr>
            </tbody>
          </UiTabel>
        </section>

        <section v-if="data.lambat_terjual" class="grid content-start gap-3 print:hidden">
          <h2>Lambat terjual</h2>
          <p class="text-sm text-redup">Per hari ini, tidak bergantung pada periode laporan.</p>
          <UiTabel :kosong="data.lambat_terjual.length ? null : 'Tidak ada barang yang lambat terjual.'">
            <thead><tr><th>Barang</th><th class="text-right">Stok</th><th>Terakhir terjual</th></tr></thead>
            <tbody>
              <tr v-for="v in data.lambat_terjual.slice(0, 10)" :key="v.id_varian">
                <td><RouterLink :to="`/admin/stok/kartu/${v.id_varian}`">{{ v.nama_produk }} {{ v.ukuran }}/{{ v.warna }}</RouterLink></td>
                <td class="text-right tabular-nums">{{ v.stok }}</td>
                <td class="text-sm">{{ v.terakhir_terjual ? waktu(v.terakhir_terjual) : 'Belum pernah' }}</td>
              </tr>
            </tbody>
          </UiTabel>
        </section>
      </div>

      <section class="grid gap-3">
        <h2>Semua produk</h2>
        <UiTabel :kosong="data.per_produk.length ? null : 'Belum ada penjualan pada periode ini.'">
          <thead><tr><th>Produk</th><th class="text-right">Terjual</th><th class="text-right">Penjualan</th><th class="text-right">Porsi</th></tr></thead>
          <tbody>
            <tr v-for="b in data.per_produk" :key="b.kunci">
              <td>{{ b.label }}</td>
              <td class="text-right tabular-nums">{{ b.unit }}</td>
              <td class="text-right tabular-nums">{{ rupiah(b.penjualan) }}</td>
              <td class="text-right text-redup tabular-nums">{{ porsi(b) }}</td>
            </tr>
          </tbody>
        </UiTabel>
        <p class="text-sm text-redup">Nilai per produk sudah dikurangi diskon baris dan bagian diskon transaksi; retur ditampilkan terpisah di ringkasan.</p>
      </section>
    </template>
  </UiHalaman>
</template>

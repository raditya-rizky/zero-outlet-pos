<script setup lang="ts">
import { computed, ref } from 'vue'
import { api, pesanError } from '@/api'
import { usePolling } from '@/composables/polling'
import { hanyaJam, perubahan, rupiah, tanggalSaja, waktu } from '@/format'
import GrafikTren from '@/components/GrafikTren.vue'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiLencana from '@/components/ui/UiLencana.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import type { Dasbor } from '@/types'

// Dasbor pemilik (Tabel 21): ringkasan hari ini, tren, terlaris, stok, sesi kasir.
const d = ref<Dasbor | null>(null)
const galat = ref<string | null>(null)

async function muat() {
  try {
    d.value = await api.latar<Dasbor>('/ringkasan/dasbor')
    galat.value = null
  } catch (e) {
    galat.value = pesanError(e)
  }
}
usePolling(muat, 60_000)

const banding = computed(() => (d.value ? perubahan(d.value.ringkasan_hari_ini.bersih, d.value.ringkasan_kemarin.bersih) : null))
</script>

<template>
  <UiHalaman>
    <UiKepalaHalaman judul="Dasbor">
      <template #deskripsi>{{ d ? tanggalSaja(d.hari_ini) : '' }}. Diperbarui setiap menit.</template>
    </UiKepalaHalaman>
    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <template v-if="d">
      <section class="grid gap-6 md:grid-cols-[minmax(0,1.3fr)_repeat(3,minmax(0,1fr))]">
        <div>
          <p class="text-sm text-redup">Penjualan bersih hari ini</p>
          <p class="text-[2.4rem] leading-tight font-bold font-stretch-115% tabular-nums">{{ rupiah(d.ringkasan_hari_ini.bersih) }}</p>
          <p class="text-sm text-redup">
            Kemarin {{ rupiah(d.ringkasan_kemarin.bersih) }}<template v-if="banding"> ({{ banding }})</template>
          </p>
        </div>
        <div>
          <p class="text-sm text-redup">Transaksi hari ini</p>
          <p class="text-2xl font-semibold tabular-nums">{{ d.ringkasan_hari_ini.transaksi }}</p>
          <p class="text-sm text-redup">Kemarin {{ d.ringkasan_kemarin.transaksi }}</p>
        </div>
        <RouterLink to="/admin/persetujuan" class="text-tinta no-underline hover:text-denim">
          <p class="text-sm text-redup">Menunggu persetujuan</p>
          <p class="text-2xl font-semibold tabular-nums" :class="{ 'text-bahaya': d.pengajuan_menunggu }">{{ d.pengajuan_menunggu }}</p>
          <p class="text-sm text-redup">Pembatalan, retur, tukar</p>
        </RouterLink>
        <RouterLink to="/admin/stok" class="text-tinta no-underline hover:text-denim">
          <p class="text-sm text-redup">Stok menipis</p>
          <p class="text-2xl font-semibold tabular-nums" :class="{ 'text-waspada': d.stok_menipis }">{{ d.stok_menipis }}</p>
          <p class="text-sm text-redup">varian</p>
        </RouterLink>
      </section>

      <GrafikTren :titik="d.tren" kelompok="hari" judul="Total penjualan 14 hari terakhir, setelah diskon dan sebelum retur" />

      <div class="grid gap-8 lg:grid-cols-2">
        <section class="grid content-start gap-2">
          <h2>Terlaris 7 hari</h2>
          <p v-if="!d.terlaris_7_hari.length" class="text-redup">Belum ada penjualan.</p>
          <ol class="grid">
            <li v-for="(b, i) in d.terlaris_7_hari" :key="b.kunci" class="flex justify-between gap-3 border-b border-garis py-2 text-sm">
              <span><span class="mr-2 text-redup tabular-nums">{{ i + 1 }}.</span>{{ b.label }}</span>
              <span class="tabular-nums">{{ b.unit }} terjual</span>
            </li>
          </ol>
          <RouterLink to="/laporan" class="text-sm">Laporan penjualan lengkap</RouterLink>
        </section>

        <section class="grid content-start gap-2">
          <h2>Stok paling menipis</h2>
          <p v-if="!d.varian_menipis.length" class="text-redup">Semua varian di atas stok minimum.</p>
          <ul class="grid">
            <li v-for="v in d.varian_menipis" :key="v.id_varian" class="flex justify-between gap-3 border-b border-garis py-2 text-sm">
              <RouterLink :to="`/admin/stok/kartu/${v.id_varian}`">{{ v.nama_produk }} {{ v.ukuran }}/{{ v.warna }}</RouterLink>
              <span class="tabular-nums" :class="v.stok === 0 ? 'font-semibold text-bahaya' : 'text-waspada'">{{ v.stok }} / min {{ v.stok_minimum }}</span>
            </li>
          </ul>
        </section>

        <section class="grid content-start gap-2">
          <h2>Sesi kasir dibuka</h2>
          <p v-if="!d.sesi_aktif.length" class="text-redup">Tidak ada sesi yang sedang dibuka.</p>
          <ul class="grid">
            <li v-for="s in d.sesi_aktif" :key="s.id_sesi" class="flex justify-between gap-3 border-b border-garis py-2 text-sm">
              <span>{{ s.nama_kasir }}</span><span class="text-redup">sejak {{ hanyaJam(s.waktu_buka) }}</span>
            </li>
          </ul>
        </section>

        <section class="grid content-start gap-2">
          <h2>Sesi terakhir ditutup</h2>
          <p v-if="!d.sesi_terakhir.length" class="text-redup">Belum ada sesi yang ditutup.</p>
          <ul class="grid">
            <li v-for="s in d.sesi_terakhir" :key="s.id_sesi" class="flex flex-wrap items-center justify-between gap-2 border-b border-garis py-2 text-sm">
              <span>{{ s.nama_kasir }} <span class="text-redup">{{ s.waktu_tutup ? waktu(s.waktu_tutup) : '' }}</span></span>
              <UiLencana :warna="s.selisih ? 'bahaya' : 'aman'">
                {{ s.selisih ? `Selisih ${rupiah(s.selisih)}` : 'Kas sesuai' }}
              </UiLencana>
            </li>
          </ul>
          <RouterLink to="/admin/sesi" class="text-sm">Semua sesi kasir</RouterLink>
        </section>
      </div>
    </template>
  </UiHalaman>
</template>

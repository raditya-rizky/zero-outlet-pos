<script setup lang="ts">
import { computed, ref } from 'vue'
import { api, pesanError } from '@/api'
import { useAuth } from '@/stores/auth'
import { useSesi } from '@/stores/sesi'
import { usePolling } from '@/composables/polling'
import { namaJenisPengajuan, namaMetode, rupiah, sejak, waktu } from '@/format'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTombol from '@/components/ui/UiTombol.vue'
import type { Pengajuan } from '@/types'

// Status satu pengajuan untuk kasir; diperbarui otomatis selama menunggu pemilik.
const props = defineProps<{ id: string }>()
const auth = useAuth()
const sesi = useSesi()

const p = ref<Pengajuan | null>(null)
const galat = ref<string | null>(null)
const menarik = ref(false)

async function muat() {
  try {
    const sebelumnya = p.value?.status
    p.value = await api.latar<Pengajuan>(`/pengajuan/${props.id}`)
    if (p.value.status !== 'menunggu') {
      polling.berhenti()
      // Keputusan memengaruhi kas seharusnya pada sesi ini.
      if (sebelumnya === 'menunggu') sesi.muat().catch(() => undefined)
    }
  } catch (e) {
    galat.value = pesanError(e)
  }
}
const polling = usePolling(muat, 4000)

const milikSaya = computed(() => p.value?.id_pengguna === auth.pengguna?.id_pengguna)

async function tarik() {
  if (!p.value) return
  menarik.value = true
  try {
    p.value = await api.post<Pengajuan>(`/pengajuan/${p.value.id_retur}/tarik`)
    polling.berhenti()
  } catch (e) {
    galat.value = pesanError(e)
  } finally {
    menarik.value = false
  }
}
</script>

<template>
  <UiHalaman ukuran="sedang">
    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <template v-if="p">
      <UiKepalaHalaman :judul="`${namaJenisPengajuan[p.jenis]} ${p.no_faktur}`">
        <template #deskripsi>Diajukan {{ p.nama_pengaju }}, {{ sejak(p.tanggal_waktu) }}</template>
      </UiKepalaHalaman>

      <!-- Keadaan utama: apa yang harus dilakukan kasir sekarang -->
      <section v-if="p.status === 'menunggu'" class="grid gap-2" aria-live="polite">
        <p class="flex items-center gap-2 text-lg font-semibold">
          <span class="size-2.5 animate-pulse rounded-full bg-waspada" aria-hidden="true"></span>
          Menunggu persetujuan pemilik
        </p>
        <p class="text-redup">Halaman ini diperbarui otomatis. Minta pelanggan menunggu sebentar.</p>
      </section>
      <UiPesan v-else-if="p.status === 'disetujui'" jenis="berhasil" role="status">
        <strong>Disetujui oleh {{ p.nama_otorisator }}.</strong>
        <template v-if="p.jenis === 'tukar'">
          Terima barang lama dan serahkan
          <span v-for="(i, n) in p.items" :key="i.id_detail_retur">{{ n ? ', ' : ' ' }}{{ i.jumlah }} x {{ i.nama_pengganti }}</span>
          kepada pelanggan.
        </template>
        <template v-else>
          Terima barangnya dan kembalikan {{ rupiah(p.total_pengembalian) }}
          {{ p.metode_pengembalian === 'tunai' ? 'tunai dari laci kas' : `lewat ${namaMetode[p.metode_pengembalian ?? 'tunai']}` }}.
        </template>
      </UiPesan>
      <UiPesan v-else-if="p.status === 'ditolak'" jenis="galat">
        <strong>Ditolak oleh {{ p.nama_otorisator }}.</strong> {{ p.alasan_tolak }}
      </UiPesan>
      <UiPesan v-else>Pengajuan ini sudah ditarik.</UiPesan>

      <section class="grid gap-3 border-t border-garis pt-6">
        <h2>Rincian</h2>
        <ul class="grid">
          <li v-for="i in p.items" :key="i.id_detail_retur" class="flex justify-between gap-3 border-b border-garis py-2 text-sm">
            <span>
              {{ i.jumlah }} x {{ i.nama_barang }}
              <span v-if="i.nama_pengganti" class="text-redup"> ditukar {{ i.nama_pengganti }}</span>
            </span>
            <span v-if="p.jenis !== 'tukar'" class="tabular-nums">{{ rupiah(i.nilai) }}</span>
          </li>
        </ul>
        <p v-if="p.jenis !== 'tukar'" class="flex justify-between font-semibold">
          <span>Uang kembali ({{ namaMetode[p.metode_pengembalian ?? 'tunai'] }})</span>
          <span class="tabular-nums">{{ rupiah(p.total_pengembalian) }}</span>
        </p>
        <p class="text-sm text-redup">Alasan: {{ p.alasan }}. Transaksi {{ waktu(p.tanggal_transaksi) }}.</p>
      </section>

      <div class="flex flex-wrap gap-2.5">
        <UiTombol v-if="p.status === 'menunggu' && milikSaya" varian="bahaya" :disabled="menarik" @click="tarik">Tarik pengajuan</UiTombol>
        <UiTombol to="/kasir" varian="kedua">Kembali ke kasir</UiTombol>
        <UiTombol to="/riwayat" varian="polos">Riwayat</UiTombol>
      </div>
    </template>
  </UiHalaman>
</template>

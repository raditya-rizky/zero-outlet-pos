<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { api, pesanError } from '@/api'
import { usePolling } from '@/composables/polling'
import { namaJenisPengajuan, namaMetode, namaStatusPengajuan, rupiah, sejak } from '@/format'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiLencana from '@/components/ui/UiLencana.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTombol from '@/components/ui/UiTombol.vue'
import type { Pengajuan } from '@/types'

// Persetujuan pemilik untuk pembatalan, retur, dan tukar ukuran (KF-10).
// Dirancang untuk layar ponsel karena pemilik sering tidak berada di toko.
const daftar = ref<Pengajuan[]>([])
const galat = ref<string | null>(null)
const pesan = ref<string | null>(null)
const memproses = ref<number | null>(null)
const alasanTolak = reactive<Record<number, string>>({})
const formTolak = ref<number | null>(null)

const menunggu = computed(() => daftar.value.filter((p) => p.status === 'menunggu'))
const diputuskan = computed(() => daftar.value.filter((p) => p.status !== 'menunggu').slice(0, 10))

async function muat() {
  try {
    daftar.value = await api.latar<Pengajuan[]>('/pengajuan')
    galat.value = null
  } catch (e) {
    galat.value = pesanError(e)
  }
}
usePolling(muat, 10_000)

async function putuskan(p: Pengajuan, setuju: boolean) {
  memproses.value = p.id_retur
  galat.value = null
  try {
    if (setuju) {
      await api.post(`/pengajuan/${p.id_retur}/setujui`)
      pesan.value = `${namaJenisPengajuan[p.jenis]} ${p.no_faktur} disetujui. Kasir ${p.nama_pengaju} sudah diberi tahu di layarnya.`
    } else {
      await api.post(`/pengajuan/${p.id_retur}/tolak`, { alasan: alasanTolak[p.id_retur] ?? '' })
      pesan.value = `${namaJenisPengajuan[p.jenis]} ${p.no_faktur} ditolak.`
      formTolak.value = null
    }
    await muat()
  } catch (e) {
    galat.value = pesanError(e)
  } finally {
    memproses.value = null
  }
}

function warnaStatus(s: Pengajuan['status']) {
  return s === 'disetujui' ? 'aman' : s === 'ditolak' ? 'bahaya' : 'netral'
}
</script>

<template>
  <UiHalaman ukuran="lebar">
    <UiKepalaHalaman judul="Persetujuan">
      <template #deskripsi>Pengajuan pembatalan, retur, dan tukar ukuran dari kasir. Daftar diperbarui otomatis.</template>
    </UiKepalaHalaman>

    <UiPesan v-if="pesan" jenis="berhasil">{{ pesan }}</UiPesan>
    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <p v-if="!menunggu.length" class="py-6 text-redup">Tidak ada pengajuan yang menunggu.</p>

    <ul class="grid">
      <li v-for="p in menunggu" :key="p.id_retur" class="grid gap-3 border-t border-garis py-5">
        <div class="flex flex-wrap items-baseline justify-between gap-2">
          <h2>{{ namaJenisPengajuan[p.jenis] }} {{ p.no_faktur }}</h2>
          <span class="text-sm text-redup">{{ p.nama_pengaju }}, {{ sejak(p.tanggal_waktu) }}</span>
        </div>
        <UiPesan v-if="p.lewat_batas_hari">Transaksi sudah melewati batas hari retur.</UiPesan>
        <ul class="grid text-sm">
          <li v-for="i in p.items" :key="i.id_detail_retur" class="flex justify-between gap-3 py-1">
            <span>
              {{ i.jumlah }} x {{ i.nama_barang }}
              <span v-if="i.nama_pengganti" class="text-redup"> ditukar {{ i.nama_pengganti }}</span>
            </span>
            <span v-if="p.jenis !== 'tukar'" class="tabular-nums">{{ rupiah(i.nilai) }}</span>
          </li>
        </ul>
        <p v-if="p.jenis !== 'tukar'" class="flex justify-between font-semibold">
          <span>Uang kembali, {{ namaMetode[p.metode_pengembalian ?? 'tunai'] }}</span>
          <span class="text-lg tabular-nums">{{ rupiah(p.total_pengembalian) }}</span>
        </p>
        <p class="text-sm">Alasan: {{ p.alasan }}</p>

        <div v-if="formTolak === p.id_retur" class="grid gap-2">
          <textarea v-model="alasanTolak[p.id_retur]" :aria-label="`Alasan menolak ${p.no_faktur}`" placeholder="Alasan penolakan untuk kasir"></textarea>
          <div class="grid grid-cols-2 gap-2">
            <UiTombol varian="kedua" @click="formTolak = null">Batal</UiTombol>
            <UiTombol
              varian="bahaya"
              :disabled="memproses === p.id_retur || !(alasanTolak[p.id_retur] ?? '').trim()"
              @click="putuskan(p, false)"
            >
              Kirim penolakan
            </UiTombol>
          </div>
        </div>
        <div v-else class="grid grid-cols-2 gap-2">
          <UiTombol varian="kedua" ukuran="besar" @click="formTolak = p.id_retur">Tolak</UiTombol>
          <UiTombol ukuran="besar" :disabled="memproses === p.id_retur" @click="putuskan(p, true)">Setujui</UiTombol>
        </div>
      </li>
    </ul>

    <section v-if="diputuskan.length" class="grid gap-2 border-t border-garis pt-6">
      <h2>Baru diputuskan</h2>
      <ul class="grid text-sm">
        <li v-for="p in diputuskan" :key="p.id_retur" class="flex flex-wrap justify-between gap-2 border-b border-garis py-2">
          <span>{{ namaJenisPengajuan[p.jenis] }} {{ p.no_faktur }}, {{ p.nama_pengaju }}</span>
          <UiLencana :warna="warnaStatus(p.status)">{{ namaStatusPengajuan[p.status] }}</UiLencana>
        </li>
      </ul>
    </section>
  </UiHalaman>
</template>

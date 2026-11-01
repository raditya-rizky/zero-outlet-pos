<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { api, pesanError } from '@/api'
import { useAuth } from '@/stores/auth'
import { hariIni, namaJenisPengajuan, namaMetode, namaStatusPengajuan, rupiah, sejak, waktu } from '@/format'
import type { FakturPengajuan, Pengajuan, Pengguna, RingkasTransaksi } from '@/types'
import DialogPengajuan from '@/components/DialogPengajuan.vue'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiLencana from '@/components/ui/UiLencana.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTabel from '@/components/ui/UiTabel.vue'
import UiTombol from '@/components/ui/UiTombol.vue'

const auth = useAuth()
const router = useRouter()

// ---------- Retur/tukar dari struk pelanggan (KF-10) ----------
const fakturDicari = ref('')
const faktur = ref<FakturPengajuan | null>(null)
const galatFaktur = ref<string | null>(null)
const pengajuan = ref<Pengajuan[]>([])

async function cariFaktur() {
  galatFaktur.value = null
  const no = fakturDicari.value.trim()
  if (!no) return
  try {
    faktur.value = await api.get<FakturPengajuan>(`/transaksi/faktur/${encodeURIComponent(no)}`)
  } catch (e) {
    galatFaktur.value = pesanError(e)
  }
}

function terkirim(p: Pengajuan) {
  faktur.value = null
  router.push(`/pengajuan/${p.id_retur}`)
}

function warnaStatus(s: Pengajuan['status']) {
  return s === 'disetujui' ? 'aman' : s === 'ditolak' ? 'bahaya' : s === 'menunggu' ? 'waspada' : 'netral'
}
const noFaktur = ref('')
const dari = ref(hariIni())
const sampai = ref(hariIni())
const idKasir = ref('')
const kasir = ref<Pengguna[]>([])
const data = ref<RingkasTransaksi[]>([])
const galat = ref<string | null>(null)
const memuat = ref(false)

async function muat() {
  memuat.value = true
  galat.value = null
  try {
    data.value = await api.get<RingkasTransaksi[]>('/transaksi', {
      no_faktur: noFaktur.value,
      dari: dari.value,
      sampai: sampai.value,
      id_pengguna: idKasir.value,
    })
  } catch (e) {
    galat.value = pesanError(e)
  } finally {
    memuat.value = false
  }
}

onMounted(async () => {
  muat()
  api
    .get<Pengajuan[]>('/pengajuan')
    .then((d) => (pengajuan.value = d.slice(0, 5)))
    .catch(() => undefined)
  if (auth.isAdmin) kasir.value = await api.get<Pengguna[]>('/pengguna').catch(() => [])
})
</script>

<template>
  <UiHalaman>
    <UiKepalaHalaman judul="Riwayat transaksi">
      <template #deskripsi>
        {{ auth.isAdmin ? 'Semua transaksi toko.' : 'Transaksi yang Anda proses.' }} Buka struk untuk melihat rincian atau mencetak ulang.
      </template>
    </UiKepalaHalaman>

    <section class="grid gap-3">
      <h2>Retur, tukar ukuran, atau pembatalan</h2>
      <form class="flex flex-wrap items-end gap-3 [&>label]:flex-[1_1_240px]" @submit.prevent="cariFaktur">
        <UiIsian label="Nomor faktur pada struk pelanggan">
          <input v-model="fakturDicari" placeholder="mis. ZO-261031-000012" autocapitalize="characters" />
        </UiIsian>
        <UiTombol varian="kedua" :disabled="!fakturDicari.trim()">Cari faktur</UiTombol>
      </form>
      <UiPesan v-if="galatFaktur" jenis="galat">{{ galatFaktur }}</UiPesan>

      <ul v-if="pengajuan.length" class="grid text-sm" aria-label="Pengajuan terbaru">
        <li v-for="p in pengajuan" :key="p.id_retur" class="flex flex-wrap items-center justify-between gap-2 border-b border-garis py-2">
          <RouterLink :to="`/pengajuan/${p.id_retur}`">{{ namaJenisPengajuan[p.jenis] }} {{ p.no_faktur }}</RouterLink>
          <span class="flex items-center gap-3">
            <span class="text-redup">{{ sejak(p.tanggal_waktu) }}</span>
            <UiLencana :warna="warnaStatus(p.status)">{{ namaStatusPengajuan[p.status] }}</UiLencana>
          </span>
        </li>
      </ul>
    </section>

    <form class="flex flex-wrap items-end gap-3 border-t border-garis pt-8 [&>label]:flex-[1_1_160px]" @submit.prevent="muat">
      <UiIsian label="No. faktur"><input v-model="noFaktur" placeholder="mis. ZO-261024" /></UiIsian>
      <UiIsian label="Dari"><input v-model="dari" type="date" /></UiIsian>
      <UiIsian label="Sampai"><input v-model="sampai" type="date" /></UiIsian>
      <UiIsian v-if="auth.isAdmin" label="Kasir">
        <select v-model="idKasir">
          <option value="">Semua kasir</option>
          <option v-for="k in kasir" :key="k.id_pengguna" :value="k.id_pengguna">{{ k.nama }}</option>
        </select>
      </UiIsian>
      <UiTombol :disabled="memuat">Tampilkan</UiTombol>
    </form>

    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <UiTabel :kosong="!memuat && !data.length ? 'Belum ada transaksi pada rentang tanggal ini.' : null">
      <thead>
        <tr>
          <th>Waktu</th>
          <th>No. faktur</th>
          <th>Kasir</th>
          <th class="text-right">Barang</th>
          <th>Bayar</th>
          <th class="text-right">Total</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="t in data" :key="t.id_transaksi">
          <td>{{ waktu(t.tanggal_waktu) }}</td>
          <td>
            {{ t.no_faktur }}
            <UiLencana v-if="t.status === 'batal'" warna="bahaya">Batal</UiLencana>
          </td>
          <td>{{ t.nama_kasir }}</td>
          <td class="text-right tabular-nums">{{ t.jumlah_barang }}</td>
          <td>{{ namaMetode[t.metode_bayar] }}</td>
          <td class="text-right tabular-nums">{{ rupiah(t.total) }}</td>
          <td class="py-1 text-right whitespace-nowrap">
            <UiTombol :to="`/struk/${t.id_transaksi}`" varian="polos" ukuran="kecil">Lihat</UiTombol>
            <UiTombol :to="{ path: `/struk/${t.id_transaksi}`, query: { cetak_ulang: '1' } }" varian="polos" ukuran="kecil">Cetak ulang</UiTombol>
          </td>
        </tr>
      </tbody>
    </UiTabel>
    <DialogPengajuan v-if="faktur" :faktur="faktur" @tutup="faktur = null" @terkirim="terkirim" />
  </UiHalaman>
</template>

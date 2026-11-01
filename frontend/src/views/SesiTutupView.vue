<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { api, pesanError } from '@/api'
import { useSesi } from '@/stores/sesi'
import { useKeranjang } from '@/stores/keranjang'
import { namaJenisPengajuan, parseRupiah, ribuan, rupiah, waktu } from '@/format'
import type { Pengajuan, Rekap } from '@/types'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTombol from '@/components/ui/UiTombol.vue'

const sesi = useSesi()
const router = useRouter()
const keranjang = useKeranjang()

const kasFisik = ref('')
const alasan = ref('')
const galat = ref<string | null>(null)
const memproses = ref(false)
const hasil = ref<Rekap | null>(null)

// Pengajuan yang belum diputuskan menahan penutupan sesi (KF-10, KF-19).
const menunggu = ref<Pengajuan[]>([])

async function muatMenunggu() {
  const daftar = await api.get<Pengajuan[]>('/pengajuan', { status: 'menunggu' }).catch(() => [])
  menunggu.value = daftar.filter((p) => p.id_sesi === sesi.aktif?.sesi.id_sesi)
}

async function tarik(p: Pengajuan) {
  try {
    await api.post(`/pengajuan/${p.id_retur}/tarik`)
    await muatMenunggu()
  } catch (e) {
    galat.value = pesanError(e)
  }
}

onMounted(async () => {
  try {
    await sesi.muat()
    await muatMenunggu()
  } catch (e) {
    galat.value = pesanError(e)
  }
})

const rekap = computed(() => hasil.value ?? sesi.aktif)
const terisi = computed(() => kasFisik.value.trim() !== '')
const selisih = computed(() => (rekap.value ? parseRupiah(kasFisik.value) - rekap.value.kas_seharusnya : 0))
const perluAlasan = computed(() => terisi.value && selisih.value !== 0)

function rapikan() {
  if (terisi.value) kasFisik.value = ribuan(parseRupiah(kasFisik.value))
}

function tandaSelisih(n: number) {
  return n === 0 ? 'Sesuai' : n > 0 ? `Lebih ${rupiah(n)}` : `Kurang ${rupiah(-n)}`
}

async function tutup() {
  if (perluAlasan.value && !alasan.value.trim()) {
    galat.value = 'Tuliskan alasan selisih kas sebelum menutup sesi.'
    return
  }
  memproses.value = true
  galat.value = null
  try {
    hasil.value = await sesi.tutup(parseRupiah(kasFisik.value), alasan.value.trim() || null)
    keranjang.kosongkan()
  } catch (e) {
    galat.value = pesanError(e)
  } finally {
    memproses.value = false
  }
}
</script>

<template>
  <UiHalaman ukuran="sedang">
    <div>
      <h1>{{ hasil ? 'Sesi ditutup' : 'Tutup sesi kasir' }}</h1>
      <p v-if="rekap" class="text-redup">
        {{ rekap.sesi.nama_kasir }}, dibuka {{ waktu(rekap.sesi.waktu_buka) }}
        <template v-if="rekap.sesi.waktu_tutup">, ditutup {{ waktu(rekap.sesi.waktu_tutup) }}</template>
      </p>
    </div>

    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <template v-if="rekap">
      <div class="grid gap-3">
        <h2>Kas di laci</h2>
        <dl class="grid gap-1.5 [&_dd]:font-semibold [&>div]:flex [&>div]:justify-between [&>div]:gap-3">
          <div><dt>Modal awal</dt><dd class="tabular-nums">{{ rupiah(rekap.sesi.modal_awal) }}</dd></div>
          <div><dt>Penjualan tunai</dt><dd class="tabular-nums">+ {{ rupiah(rekap.penjualan_tunai) }}</dd></div>
          <div v-if="rekap.refund_tunai"><dt>Pengembalian tunai</dt><dd class="tabular-nums">− {{ rupiah(rekap.refund_tunai) }}</dd></div>
          <div class="border-t border-garis pt-2 text-[1.1rem] font-bold">
            <dt>Kas seharusnya</dt><dd class="tabular-nums">{{ rupiah(rekap.kas_seharusnya) }}</dd>
          </div>
        </dl>
        <h2>Pembayaran nontunai</h2>
        <dl class="grid gap-1.5 [&_dd]:font-semibold [&>div]:flex [&>div]:justify-between [&>div]:gap-3">
          <div><dt>Transfer</dt><dd class="tabular-nums">{{ rupiah(rekap.penjualan_transfer) }}</dd></div>
          <div><dt>QRIS</dt><dd class="tabular-nums">{{ rupiah(rekap.penjualan_qris) }}</dd></div>
        </dl>
        <p class="text-sm text-redup">
          {{ rekap.jumlah_transaksi }} transaksi, total penjualan {{ rupiah(rekap.total_penjualan) }}. Cocokkan transfer dan QRIS dengan
          mutasi rekening atau aplikasi pembayaran.
        </p>
      </div>

      <div v-if="hasil" class="grid gap-4 border-t border-garis pt-8">
        <UiPesan :jenis="hasil.sesi.selisih === 0 ? 'berhasil' : 'info'">
          Kas fisik {{ rupiah(hasil.sesi.kas_fisik ?? 0) }}. {{ tandaSelisih(hasil.sesi.selisih ?? 0) }}.
          <template v-if="hasil.sesi.alasan_selisih"> Alasan: {{ hasil.sesi.alasan_selisih }}</template>
        </UiPesan>
        <div class="flex flex-wrap gap-2.5">
          <UiTombol to="/sesi/buka">Buka sesi baru</UiTombol>
          <UiTombol varian="kedua" @click="router.push('/riwayat')">Lihat riwayat</UiTombol>
        </div>
      </div>

      <section v-else-if="menunggu.length" class="grid gap-2 border-t border-garis pt-8">
        <h2>Pengajuan belum diputuskan</h2>
        <p class="text-sm text-redup">
          Sesi baru dapat ditutup setelah pemilik memutuskan pengajuan ini, atau setelah Anda menariknya.
        </p>
        <ul class="grid text-sm">
          <li v-for="p in menunggu" :key="p.id_retur" class="flex flex-wrap items-center justify-between gap-2 border-b border-garis py-2">
            <RouterLink :to="`/pengajuan/${p.id_retur}`">{{ namaJenisPengajuan[p.jenis] }} {{ p.no_faktur }}</RouterLink>
            <UiTombol varian="bahaya" ukuran="kecil" @click="tarik(p)">Tarik</UiTombol>
          </li>
        </ul>
      </section>

      <form v-else class="grid gap-4 border-t border-garis pt-8" @submit.prevent="tutup">
        <UiIsian label="Kas fisik yang dihitung" petunjuk="Hitung seluruh uang tunai di laci, termasuk modal awal.">
          <input v-model="kasFisik" inputmode="numeric" class="min-h-14 text-2xl font-bold tabular-nums" placeholder="0" autofocus @blur="rapikan" />
        </UiIsian>

        <div
          v-if="terisi"
          class="flex justify-between rounded-kecil px-3.5 py-3 font-semibold"
          :class="selisih === 0 ? 'bg-aman-terang text-aman' : 'bg-bahaya-terang text-bahaya'"
        >
          <span>Selisih</span>
          <strong class="tabular-nums">{{ tandaSelisih(selisih) }}</strong>
        </div>

        <UiIsian v-if="perluAlasan" label="Alasan selisih" petunjuk="Wajib diisi. Pemilik dapat meninjau alasan ini.">
          <textarea v-model="alasan" required placeholder="mis. salah memberi kembalian pada transaksi siang"></textarea>
        </UiIsian>

        <UiTombol ukuran="besar" :disabled="memproses || !terisi">Tutup sesi</UiTombol>
      </form>
    </template>
  </UiHalaman>
</template>

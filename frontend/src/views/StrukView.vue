<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api, pesanError } from '@/api'
import KertasStruk from '@/components/KertasStruk.vue'
import { namaJenisPengajuan, namaStatusPengajuan, rupiah } from '@/format'
import type { FakturPengajuan, Pengajuan, Struk } from '@/types'
import DialogPengajuan from '@/components/DialogPengajuan.vue'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTombol from '@/components/ui/UiTombol.vue'

const props = defineProps<{ id: string }>()
const route = useRoute()
const router = useRouter()

const struk = ref<Struk | null>(null)
const galat = ref<string | null>(null)
const tombolCetak = ref<{ $el: HTMLElement }>()
const baru = route.query.baru === '1'

onMounted(async () => {
  const cetakUlang = route.query.cetak_ulang === '1'
  try {
    struk.value = await api.get<Struk>(`/transaksi/${props.id}/struk`, { cetak_ulang: cetakUlang || undefined })
    // Hapus penanda dari URL agar memuat ulang halaman tidak menambah hitungan cetak.
    if (cetakUlang) router.replace({ query: { ...route.query, cetak_ulang: undefined } })
    await nextTick()
    tombolCetak.value?.$el.focus()
  } catch (e) {
    galat.value = pesanError(e)
  }
})

function cetak() {
  window.print()
}

// Pembatalan/retur/tukar untuk transaksi ini (KF-10).
const faktur = ref<FakturPengajuan | null>(null)
async function ajukan() {
  if (!struk.value) return
  galat.value = null
  try {
    faktur.value = await api.get<FakturPengajuan>(`/transaksi/faktur/${encodeURIComponent(struk.value.no_faktur)}`)
  } catch (e) {
    galat.value = pesanError(e)
  }
}
</script>

<template>
  <UiHalaman ukuran="lebar" class="print:m-0 print:p-0">
    <UiPesan v-if="galat" jenis="galat" class="print:hidden">{{ galat }}</UiPesan>

    <template v-if="struk">
      <UiKepalaHalaman :judul="baru ? 'Transaksi tersimpan' : 'Struk transaksi'" class="print:hidden">
        <template #deskripsi>
          {{ struk.no_faktur }}<template v-if="baru && struk.metode_bayar === 'tunai'">. Berikan kembalian {{ rupiah(struk.kembalian) }} kepada pelanggan.</template>
        </template>
      </UiKepalaHalaman>

      <div class="flex flex-wrap items-start gap-6">
        <div class="rounded-sedang bg-kertas p-6 print:bg-transparent print:p-0">
          <KertasStruk :struk="struk" class="border border-garis print:border-0" />
        </div>
        <div class="grid flex-[1_1_220px] gap-2.5 print:hidden">
          <UiTombol ref="tombolCetak" ukuran="besar" @click="cetak">Cetak struk</UiTombol>
          <UiTombol v-if="baru" to="/kasir" varian="kedua" ukuran="besar">Transaksi baru</UiTombol>
          <UiTombol v-else to="/riwayat" varian="kedua">Kembali ke riwayat</UiTombol>
          <UiTombol v-if="struk.status === 'selesai'" varian="polos" @click="ajukan">Ajukan pembatalan, retur, atau tukar</UiTombol>
          <ul v-if="struk.pengajuan.length" class="grid text-sm">
            <li v-for="p in struk.pengajuan" :key="p.id_retur" class="flex justify-between gap-2 border-b border-garis py-1.5">
              <RouterLink :to="`/pengajuan/${p.id_retur}`">{{ namaJenisPengajuan[p.jenis] }}</RouterLink>
              <span class="text-redup">{{ namaStatusPengajuan[p.status] }}</span>
            </li>
          </ul>
          <p class="text-sm text-redup">Tanpa printer thermal, pilih “Simpan sebagai PDF” pada jendela cetak.</p>
        </div>
      </div>
    </template>
    <DialogPengajuan
      v-if="faktur"
      :faktur="faktur"
      @tutup="faktur = null"
      @terkirim="(p: Pengajuan) => router.push(`/pengajuan/${p.id_retur}`)"
    />
  </UiHalaman>
</template>

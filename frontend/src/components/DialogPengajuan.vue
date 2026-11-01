<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { api, pesanError } from '@/api'
import { namaMetode, nilaiRetur, rupiah, waktu } from '@/format'
import DialogModal from '@/components/DialogModal.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTombol from '@/components/ui/UiTombol.vue'
import type { FakturPengajuan, JenisPengajuan, MetodeBayar, Pengajuan } from '@/types'

// Pengajuan pembatalan, retur, atau tukar ukuran untuk satu faktur (KF-10).
const props = defineProps<{ faktur: FakturPengajuan }>()
const emit = defineEmits<{ tutup: []; terkirim: [Pengajuan] }>()

const jenis = ref<JenisPengajuan>('retur')
const alasan = ref('')
const metode = ref<MetodeBayar>(props.faktur.metode_bayar)
const jumlah = reactive<Record<number, number>>({})
const pengganti = reactive<Record<number, number | ''>>({})
const galat = ref<string | null>(null)
const mengirim = ref(false)

const dipilih = computed(() => props.faktur.items.filter((b) => (jumlah[b.id_detail] ?? 0) > 0))

const totalPengembalian = computed(() => {
  if (jenis.value === 'batal') return props.faktur.total
  if (jenis.value === 'tukar') return 0
  return dipilih.value.reduce((n, b) => n + nilaiRetur(b.dibayar, b.jumlah, b.jumlah - b.sisa, jumlah[b.id_detail] ?? 0), 0)
})

const bisaKirim = computed(() => {
  if (!alasan.value.trim() || mengirim.value) return false
  if (jenis.value === 'batal') return props.faktur.bisa_dibatalkan
  if (!dipilih.value.length) return false
  return jenis.value !== 'tukar' || dipilih.value.every((b) => pengganti[b.id_detail])
})

function ubahJumlah(id: number, sisa: number, nilai: number) {
  jumlah[id] = Math.max(0, Math.min(sisa, Math.floor(nilai) || 0))
}

async function kirim() {
  if (!bisaKirim.value) return
  mengirim.value = true
  galat.value = null
  try {
    const p = await api.post<Pengajuan>('/pengajuan', {
      id_transaksi: props.faktur.id_transaksi,
      jenis: jenis.value,
      alasan: alasan.value,
      metode_pengembalian: jenis.value === 'tukar' ? null : metode.value,
      items:
        jenis.value === 'batal'
          ? []
          : dipilih.value.map((b) => ({
              id_detail: b.id_detail,
              jumlah: jumlah[b.id_detail],
              id_varian_pengganti: jenis.value === 'tukar' ? pengganti[b.id_detail] : null,
            })),
    })
    emit('terkirim', p)
  } catch (e) {
    galat.value = pesanError(e)
  } finally {
    mengirim.value = false
  }
}

const pilihanJenis: { nilai: JenisPengajuan; label: string }[] = [
  { nilai: 'retur', label: 'Retur' },
  { nilai: 'tukar', label: 'Tukar ukuran' },
  { nilai: 'batal', label: 'Batalkan' },
]
</script>

<template>
  <DialogModal :judul="`Pengajuan untuk ${faktur.no_faktur}`" @tutup="emit('tutup')">
    <form class="grid gap-4" @submit.prevent="kirim">
      <p class="text-sm text-redup">
        {{ waktu(faktur.tanggal_waktu) }}, kasir {{ faktur.nama_kasir }}, total {{ rupiah(faktur.total) }} ({{ namaMetode[faktur.metode_bayar] }})
      </p>
      <UiPesan v-if="faktur.status === 'batal'" jenis="galat">Transaksi ini sudah dibatalkan.</UiPesan>
      <UiPesan v-else-if="faktur.lewat_batas_hari">Transaksi ini sudah melewati batas hari retur. Pemilik akan memutuskan.</UiPesan>

      <template v-if="faktur.status !== 'batal'">
        <fieldset class="grid grid-cols-3 gap-2">
          <legend class="mb-1.5 text-sm text-redup">Jenis pengajuan</legend>
          <label
            v-for="j in pilihanJenis"
            :key="j.nilai"
            class="flex min-h-11 cursor-pointer items-center justify-center rounded-kecil border border-garis-kuat text-sm font-medium has-checked:border-denim has-checked:bg-denim has-checked:text-white has-disabled:cursor-not-allowed has-disabled:opacity-50 has-focus-visible:shadow-fokus"
          >
            <input v-model="jenis" type="radio" name="jenis" :value="j.nilai" class="sr-only" :disabled="j.nilai === 'batal' && !faktur.bisa_dibatalkan" />
            {{ j.label }}
          </label>
        </fieldset>
        <p v-if="!faktur.bisa_dibatalkan" class="-mt-2 text-[0.8rem] text-redup">
          Pembatalan hanya untuk transaksi di sesi Anda yang sedang dibuka. Untuk transaksi lain, gunakan retur semua barang.
        </p>

        <p v-if="jenis === 'batal'" class="text-sm">Semua barang dikembalikan ke stok dan transaksi ditandai batal.</p>
        <ul v-else class="grid border-t border-garis">
          <li v-for="b in faktur.items" :key="b.id_detail" class="grid gap-2 border-b border-garis py-3">
            <div class="flex items-start justify-between gap-3">
              <div>
                <strong class="text-sm">{{ b.nama_barang }}</strong>
                <p class="text-[0.8rem] text-redup">Dibeli {{ b.jumlah }}, dapat diretur {{ b.sisa }}</p>
              </div>
              <input
                :value="jumlah[b.id_detail] ?? 0"
                type="number"
                min="0"
                :max="b.sisa"
                :disabled="b.sisa === 0"
                class="min-h-10 w-20 text-center tabular-nums"
                :aria-label="`Jumlah ${b.nama_barang}`"
                @input="ubahJumlah(b.id_detail, b.sisa, Number(($event.target as HTMLInputElement).value))"
              />
            </div>
            <select
              v-if="jenis === 'tukar' && (jumlah[b.id_detail] ?? 0) > 0"
              v-model="pengganti[b.id_detail]"
              class="min-h-10 text-sm"
              :aria-label="`Pengganti ${b.nama_barang}`"
            >
              <option value="">Pilih ukuran/warna pengganti</option>
              <option v-for="v in b.pengganti" :key="v.id_varian" :value="v.id_varian" :disabled="v.stok < (jumlah[b.id_detail] ?? 0)">
                {{ v.ukuran }} / {{ v.warna }} (stok {{ v.stok }})
              </option>
            </select>
          </li>
        </ul>

        <UiIsian v-if="jenis !== 'tukar'" label="Pengembalian uang">
          <select v-model="metode">
            <option value="tunai">Tunai dari laci kas</option>
            <option value="transfer">Transfer</option>
            <option value="qris">QRIS</option>
          </select>
        </UiIsian>

        <UiIsian label="Alasan" petunjuk="Wajib diisi; dibaca pemilik saat memutuskan.">
          <textarea v-model="alasan" required placeholder="mis. ukuran kekecilan, barang cacat jahitan"></textarea>
        </UiIsian>

        <p v-if="jenis !== 'tukar'" class="flex items-baseline justify-between font-semibold">
          <span>Uang kembali</span>
          <span class="text-xl tabular-nums">{{ rupiah(totalPengembalian) }}</span>
        </p>

        <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>
        <UiTombol ukuran="besar" :disabled="!bisaKirim">{{ mengirim ? 'Mengirim…' : 'Kirim pengajuan' }}</UiTombol>
      </template>
    </form>
  </DialogModal>
</template>

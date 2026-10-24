<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { pesanError } from '@/api'
import { useSesi } from '@/stores/sesi'
import { useKeranjang } from '@/stores/keranjang'
import { parseRupiah, ribuan, rupiah, waktu } from '@/format'
import type { Rekap } from '@/types'

const sesi = useSesi()
const router = useRouter()
const keranjang = useKeranjang()

const kasFisik = ref('')
const alasan = ref('')
const galat = ref<string | null>(null)
const memproses = ref(false)
const hasil = ref<Rekap | null>(null)

onMounted(() => sesi.muat().catch((e) => (galat.value = pesanError(e))))

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
  <div class="halaman sempit">
    <div>
      <h1>{{ hasil ? 'Sesi ditutup' : 'Tutup sesi kasir' }}</h1>
      <p v-if="rekap" class="redup">
        {{ rekap.sesi.nama_kasir }}, dibuka {{ waktu(rekap.sesi.waktu_buka) }}
        <template v-if="rekap.sesi.waktu_tutup">, ditutup {{ waktu(rekap.sesi.waktu_tutup) }}</template>
      </p>
    </div>

    <p v-if="galat" class="pesan galat" role="alert">{{ galat }}</p>

    <template v-if="rekap">
      <div class="panel rekap">
        <h2>Kas di laci</h2>
        <dl>
          <div><dt>Modal awal</dt><dd class="angka">{{ rupiah(rekap.sesi.modal_awal) }}</dd></div>
          <div><dt>Penjualan tunai</dt><dd class="angka">+ {{ rupiah(rekap.penjualan_tunai) }}</dd></div>
          <div v-if="rekap.refund_tunai"><dt>Pengembalian tunai</dt><dd class="angka">− {{ rupiah(rekap.refund_tunai) }}</dd></div>
          <div class="jumlah"><dt>Kas seharusnya</dt><dd class="angka">{{ rupiah(rekap.kas_seharusnya) }}</dd></div>
        </dl>
        <h2>Pembayaran nontunai</h2>
        <dl>
          <div><dt>Transfer</dt><dd class="angka">{{ rupiah(rekap.penjualan_transfer) }}</dd></div>
          <div><dt>QRIS</dt><dd class="angka">{{ rupiah(rekap.penjualan_qris) }}</dd></div>
        </dl>
        <p class="kecil redup">
          {{ rekap.jumlah_transaksi }} transaksi, total penjualan {{ rupiah(rekap.total_penjualan) }}. Cocokkan transfer dan QRIS dengan
          mutasi rekening atau aplikasi pembayaran.
        </p>
      </div>

      <div v-if="hasil" class="panel isi">
        <p class="pesan" :class="hasil.sesi.selisih === 0 ? 'berhasil' : 'info'">
          Kas fisik {{ rupiah(hasil.sesi.kas_fisik ?? 0) }}. {{ tandaSelisih(hasil.sesi.selisih ?? 0) }}.
          <template v-if="hasil.sesi.alasan_selisih"> Alasan: {{ hasil.sesi.alasan_selisih }}</template>
        </p>
        <div class="tombol-baris">
          <RouterLink to="/sesi/buka" class="tombol">Buka sesi baru</RouterLink>
          <button class="tombol kedua" @click="router.push('/riwayat')">Lihat riwayat</button>
        </div>
      </div>

      <form v-else class="panel isi" @submit.prevent="tutup">
        <label class="isian">
          Kas fisik yang dihitung
          <small>Hitung seluruh uang tunai di laci, termasuk modal awal.</small>
          <input v-model="kasFisik" inputmode="numeric" class="angka besar" placeholder="0" autofocus @blur="rapikan" />
        </label>

        <div v-if="terisi" class="selisih" :class="selisih === 0 ? 'sesuai' : 'beda'">
          <span>Selisih</span>
          <strong class="angka">{{ tandaSelisih(selisih) }}</strong>
        </div>

        <label v-if="perluAlasan" class="isian">
          Alasan selisih
          <small>Wajib diisi. Pemilik dapat meninjau alasan ini.</small>
          <textarea v-model="alasan" required placeholder="mis. salah memberi kembalian pada transaksi siang"></textarea>
        </label>

        <button class="tombol besar" :disabled="memproses || !terisi">Tutup sesi</button>
      </form>
    </template>
  </div>
</template>

<style scoped>
.sempit {
  max-width: 560px;
}
.rekap {
  display: grid;
  gap: 12px;
  padding: 20px;
}
dl {
  margin: 0;
  display: grid;
  gap: 6px;
}
dl div {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}
dd {
  margin: 0;
  font-weight: 600;
}
dl .jumlah {
  padding-top: 8px;
  border-top: 1px solid var(--garis);
  font-weight: 700;
  font-size: 1.1rem;
}
.isi {
  display: grid;
  gap: 16px;
  padding: 20px;
}
input.besar {
  font-size: 1.5rem;
  font-weight: 700;
  min-height: 56px;
}
.selisih {
  display: flex;
  justify-content: space-between;
  padding: 12px 14px;
  border-radius: var(--r-kecil);
  font-weight: 600;
}
.selisih.sesuai {
  background: var(--aman-terang);
  color: var(--aman);
}
.selisih.beda {
  background: var(--bahaya-terang);
  color: var(--bahaya);
}
.tombol-baris {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
</style>

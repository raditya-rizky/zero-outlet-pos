<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api, pesanError } from '@/api'
import { useAuth } from '@/stores/auth'
import { hariIni, namaMetode, rupiah, waktu } from '@/format'
import type { Pengguna, RingkasTransaksi } from '@/types'

const auth = useAuth()
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
  if (auth.isAdmin) kasir.value = await api.get<Pengguna[]>('/pengguna').catch(() => [])
})
</script>

<template>
  <div class="halaman">
    <div class="kepala-halaman">
      <div>
        <h1>Riwayat transaksi</h1>
        <p class="redup">{{ auth.isAdmin ? 'Semua transaksi toko.' : 'Transaksi yang Anda proses.' }} Buka struk untuk melihat rincian atau mencetak ulang.</p>
      </div>
    </div>

    <form class="saring" @submit.prevent="muat">
      <label class="isian">No. faktur <input v-model="noFaktur" placeholder="mis. ZO-261024" /></label>
      <label class="isian">Dari <input v-model="dari" type="date" /></label>
      <label class="isian">Sampai <input v-model="sampai" type="date" /></label>
      <label v-if="auth.isAdmin" class="isian">
        Kasir
        <select v-model="idKasir">
          <option value="">Semua kasir</option>
          <option v-for="k in kasir" :key="k.id_pengguna" :value="k.id_pengguna">{{ k.nama }}</option>
        </select>
      </label>
      <button class="tombol" :disabled="memuat">Tampilkan</button>
    </form>

    <p v-if="galat" class="pesan galat">{{ galat }}</p>

    <div class="tabel-bungkus">
      <table>
        <thead>
          <tr>
            <th>Waktu</th>
            <th>No. faktur</th>
            <th>Kasir</th>
            <th class="kanan">Barang</th>
            <th>Bayar</th>
            <th class="kanan">Total</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="t in data" :key="t.id_transaksi">
            <td>{{ waktu(t.tanggal_waktu) }}</td>
            <td>
              {{ t.no_faktur }}
              <span v-if="t.status === 'batal'" class="lencana bahaya">Batal</span>
            </td>
            <td>{{ t.nama_kasir }}</td>
            <td class="kanan angka">{{ t.jumlah_barang }}</td>
            <td>{{ namaMetode[t.metode_bayar] }}</td>
            <td class="kanan angka">{{ rupiah(t.total) }}</td>
            <td class="kanan aksi">
              <RouterLink :to="`/struk/${t.id_transaksi}`" class="tombol polos">Lihat</RouterLink>
              <RouterLink :to="{ path: `/struk/${t.id_transaksi}`, query: { cetak_ulang: '1' } }" class="tombol polos">Cetak ulang</RouterLink>
            </td>
          </tr>
        </tbody>
      </table>
      <p v-if="!memuat && !data.length" class="kosong">Belum ada transaksi pada rentang tanggal ini.</p>
    </div>
  </div>
</template>

<style scoped>
.aksi {
  white-space: nowrap;
  padding-block: 4px;
}
.aksi .tombol {
  min-height: 36px;
}
</style>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { api, pesanError } from '@/api'
import { hariIni, parseRupiah, ribuan, rupiah } from '@/format'
import NavStok from '@/components/NavStok.vue'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTabel from '@/components/ui/UiTabel.vue'
import UiTombol from '@/components/ui/UiTombol.vue'
import type { BarangMasuk, Pemasok, Produk, Varian } from '@/types'

// Formulir barang masuk dari pemasok (KF-11).
const router = useRouter()

interface Baris {
  id_varian: number
  nama: string
  sku: string
  jumlah: number
  harga: string
}

const tanggal = ref(hariIni())
const idPemasok = ref<number | ''>('')
const pemasok = ref<Pemasok[]>([])
const pemasokBaru = ref('')
const noReferensi = ref('')
const keterangan = ref('')
const baris = ref<Baris[]>([])
const galat = ref<string | null>(null)
const menyimpan = ref(false)

const cari = ref('')
const hasil = ref<Produk[]>([])
let jeda: ReturnType<typeof setTimeout> | undefined
watch(cari, () => {
  clearTimeout(jeda)
  jeda = setTimeout(async () => {
    hasil.value = cari.value.trim() ? await api.get<Produk[]>('/produk', { cari: cari.value }).catch(() => []) : []
  }, 300)
})

function tambahVarian(p: Produk, v: Varian) {
  if (baris.value.some((b) => b.id_varian === v.id_varian)) return
  baris.value.push({
    id_varian: v.id_varian,
    nama: `${p.nama} ${v.ukuran}/${v.warna}`,
    sku: v.sku,
    jumlah: 1,
    harga: ribuan(p.harga_beli ?? 0),
  })
}

const total = computed(() => baris.value.reduce((n, b) => n + b.jumlah * parseRupiah(b.harga), 0))

async function tambahPemasok() {
  if (!pemasokBaru.value.trim()) return
  try {
    const p = await api.post<Pemasok>('/pemasok', { nama: pemasokBaru.value })
    pemasok.value.push(p)
    idPemasok.value = p.id_pemasok
    pemasokBaru.value = ''
  } catch (e) {
    galat.value = pesanError(e)
  }
}

async function simpan() {
  galat.value = null
  if (!baris.value.length) {
    galat.value = 'Tambahkan minimal satu barang lewat pencarian di atas.'
    return
  }
  menyimpan.value = true
  try {
    const hasilSimpan = await api.post<{ barang_masuk: BarangMasuk; peringatan: string[] }>('/barang-masuk', {
      tanggal: tanggal.value || null,
      id_pemasok: idPemasok.value || null,
      no_referensi: noReferensi.value,
      keterangan: keterangan.value,
      items: baris.value.map((b) => ({ id_varian: b.id_varian, jumlah: Number(b.jumlah) || 0, harga_beli: parseRupiah(b.harga) })),
    })
    const bm = hasilSimpan.barang_masuk
    router.push({
      path: '/admin/stok/masuk',
      query: {
        pesan: `BM-${bm.id_barang_masuk} dicatat: ${bm.jumlah_unit} unit, ${rupiah(bm.total_nilai)}.`,
        peringatan: hasilSimpan.peringatan.join('\n') || undefined,
      },
    })
  } catch (e) {
    galat.value = pesanError(e)
  } finally {
    menyimpan.value = false
  }
}

onMounted(async () => {
  pemasok.value = await api.get<Pemasok[]>('/pemasok').catch(() => [])
})
</script>

<template>
  <UiHalaman as="form" @submit.prevent="simpan">
    <UiKepalaHalaman judul="Catat barang masuk">
      <template #aksi><UiTombol :disabled="menyimpan">Simpan barang masuk</UiTombol></template>
    </UiKepalaHalaman>
    <NavStok />
    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <section class="grid grid-cols-[repeat(auto-fit,minmax(200px,1fr))] items-end gap-4">
      <UiIsian label="Tanggal terima"><input v-model="tanggal" type="date" /></UiIsian>
      <UiIsian label="Pemasok">
        <select v-model="idPemasok">
          <option value="">Tanpa pemasok</option>
          <option v-for="p in pemasok" :key="p.id_pemasok" :value="p.id_pemasok">{{ p.nama }}</option>
        </select>
      </UiIsian>
      <div class="flex items-end gap-2">
        <UiIsian label="Pemasok baru" class="flex-1"><input v-model="pemasokBaru" placeholder="mis. CV Konveksi Bali" /></UiIsian>
        <UiTombol type="button" varian="kedua" :disabled="!pemasokBaru.trim()" @click="tambahPemasok">Tambah</UiTombol>
      </div>
      <UiIsian label="No. nota pemasok"><input v-model="noReferensi" placeholder="Opsional" /></UiIsian>
    </section>

    <section class="grid gap-3 border-t border-garis pt-8">
      <h2>Barang</h2>
      <UiIsian label="Cari barang">
        <input v-model="cari" type="search" placeholder="Nama, kode, SKU, atau warna" />
      </UiIsian>
      <ul v-if="hasil.length" class="grid max-h-72 overflow-y-auto border-y border-garis">
        <li v-for="p in hasil" :key="p.id_produk" class="grid gap-2 border-b border-garis py-2.5 last:border-b-0">
          <span class="text-sm font-semibold">{{ p.nama }} <span class="font-normal text-redup">{{ p.kode }}</span></span>
          <div class="flex flex-wrap gap-1.5">
            <button
              v-for="v in p.varian.filter((x) => x.aktif)"
              :key="v.id_varian"
              type="button"
              class="min-h-9 rounded-kecil border border-garis-kuat px-2.5 text-sm hover:border-denim hover:text-denim"
              @click="tambahVarian(p, v)"
            >
              {{ v.ukuran }} {{ v.warna }} <span class="text-redup tabular-nums">{{ v.stok }}</span>
            </button>
          </div>
        </li>
      </ul>

      <UiTabel :kosong="baris.length ? null : 'Belum ada barang. Cari lalu pilih ukuran dan warnanya.'">
        <thead>
          <tr><th>Barang</th><th class="text-right">Jumlah</th><th class="text-right">Harga beli (Rp)</th><th class="text-right">Subtotal</th><th></th></tr>
        </thead>
        <tbody>
          <tr v-for="(b, i) in baris" :key="b.id_varian">
            <td>{{ b.nama }}<br /><span class="text-sm text-redup">{{ b.sku }}</span></td>
            <td class="text-right">
              <input v-model.number="b.jumlah" type="number" min="1" class="w-24 text-right tabular-nums" :aria-label="`Jumlah ${b.nama}`" />
            </td>
            <td class="text-right">
              <input v-model="b.harga" inputmode="numeric" class="w-32 text-right tabular-nums" :aria-label="`Harga beli ${b.nama}`" />
            </td>
            <td class="text-right tabular-nums">{{ rupiah(b.jumlah * parseRupiah(b.harga)) }}</td>
            <td class="text-right"><UiTombol type="button" varian="polos" ukuran="kecil" @click="baris.splice(i, 1)">Hapus</UiTombol></td>
          </tr>
        </tbody>
      </UiTabel>
      <p v-if="baris.length" class="flex justify-between font-semibold">
        <span>Total nilai barang masuk</span><span class="tabular-nums">{{ rupiah(total) }}</span>
      </p>
      <UiIsian label="Keterangan"><textarea v-model="keterangan" placeholder="Opsional"></textarea></UiIsian>
    </section>
  </UiHalaman>
</template>

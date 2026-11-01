<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { api, pesanError } from '@/api'
import NavStok from '@/components/NavStok.vue'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTabel from '@/components/ui/UiTabel.vue'
import UiTombol from '@/components/ui/UiTombol.vue'
import type { HasilOpname, Kategori, Produk } from '@/types'

// Stok opname: hitung fisik lalu sesuaikan stok sistem disertai alasan (KF-12, UJ-08).
const kategori = ref<Kategori[]>([])
const idKategori = ref('')
const cari = ref('')
const produk = ref<Produk[]>([])
// Vue mengubah v-model pada input type=number menjadi angka; kosong tetap string.
const fisik = reactive<Record<number, string | number>>({})
const isian = (id: number) => String(fisik[id] ?? '').trim()
const alasan = ref('')
const galat = ref<string | null>(null)
const hasil = ref<HasilOpname | null>(null)
const menyimpan = ref(false)

const baris = computed(() =>
  produk.value.flatMap((p) => p.varian.filter((v) => v.aktif).map((v) => ({ ...v, nama: p.nama }))),
)
const dihitung = computed(() => baris.value.filter((v) => isian(v.id_varian) !== ''))

function selisih(id: number, stok: number) {
  const isi = isian(id)
  return isi === '' ? null : Number(isi) - stok
}

async function muat() {
  produk.value = await api.get<Produk[]>('/produk', { cari: cari.value, id_kategori: idKategori.value }).catch(() => [])
}
let jeda: ReturnType<typeof setTimeout> | undefined
watch(cari, () => {
  clearTimeout(jeda)
  jeda = setTimeout(muat, 300)
})
watch(idKategori, muat)

async function simpan() {
  galat.value = null
  if (!alasan.value.trim()) {
    galat.value = 'Alasan penyesuaian stok wajib diisi.'
    return
  }
  menyimpan.value = true
  try {
    hasil.value = await api.post<HasilOpname>('/stok/opname', {
      alasan: alasan.value,
      items: dihitung.value.map((v) => ({ id_varian: v.id_varian, stok_fisik: Number(fisik[v.id_varian]) })),
    })
    for (const k of Object.keys(fisik)) delete fisik[Number(k)]
    alasan.value = ''
    await muat()
  } catch (e) {
    galat.value = pesanError(e)
  } finally {
    menyimpan.value = false
  }
}

onMounted(async () => {
  kategori.value = await api.get<Kategori[]>('/kategori').catch(() => [])
  muat()
})
</script>

<template>
  <UiHalaman as="form" @submit.prevent="simpan">
    <UiKepalaHalaman judul="Stok">
      <template #deskripsi>Hitung barang di rak, isi jumlah fisiknya, lalu simpan. Hanya varian yang diisi yang disesuaikan.</template>
    </UiKepalaHalaman>
    <NavStok />

    <UiPesan v-if="hasil" jenis="berhasil">
      Stok opname {{ hasil.referensi }} disimpan. {{ hasil.sesuai }} dari {{ hasil.dihitung }} varian sesuai (akurasi
      {{ hasil.akurasi_persen.toLocaleString('id-ID') }}%).
      <template v-if="hasil.selisih.some((s) => s.selisih)">
        Disesuaikan:
        <span v-for="(s, i) in hasil.selisih.filter((x) => x.selisih)" :key="s.id_varian">
          {{ i ? ', ' : '' }}{{ s.nama_barang }} {{ s.selisih > 0 ? '+' : '' }}{{ s.selisih }}
        </span>
      </template>
    </UiPesan>
    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <div class="flex flex-wrap items-end gap-3 [&>label]:flex-[1_1_200px]">
      <UiIsian label="Kategori">
        <select v-model="idKategori">
          <option value="">Semua kategori</option>
          <option v-for="k in kategori" :key="k.id_kategori" :value="k.id_kategori">{{ k.nama_kategori }}</option>
        </select>
      </UiIsian>
      <UiIsian label="Cari"><input v-model="cari" type="search" placeholder="Nama, kode, SKU, warna" /></UiIsian>
    </div>

    <UiTabel :kosong="baris.length ? null : 'Tidak ada varian yang cocok.'">
      <thead>
        <tr><th>Barang</th><th>SKU</th><th class="text-right">Stok sistem</th><th class="text-right">Stok fisik</th><th class="text-right">Selisih</th></tr>
      </thead>
      <tbody>
        <tr v-for="v in baris" :key="v.id_varian">
          <td>{{ v.nama }} <span class="text-redup">{{ v.ukuran }} / {{ v.warna }}</span></td>
          <td class="text-sm text-redup">{{ v.sku }}</td>
          <td class="text-right tabular-nums">{{ v.stok }}</td>
          <td class="text-right">
            <input
              v-model="fisik[v.id_varian]"
              type="number"
              min="0"
              class="w-24 text-right tabular-nums"
              :aria-label="`Stok fisik ${v.nama} ${v.ukuran} ${v.warna}`"
            />
          </td>
          <td
            class="text-right font-semibold tabular-nums"
            :class="selisih(v.id_varian, v.stok) ? 'text-bahaya' : 'text-aman'"
          >
            <template v-if="selisih(v.id_varian, v.stok) !== null">
              {{ selisih(v.id_varian, v.stok)! > 0 ? '+' : '' }}{{ selisih(v.id_varian, v.stok) }}
            </template>
          </td>
        </tr>
      </tbody>
    </UiTabel>

    <section class="grid gap-3 border-t border-garis pt-8">
      <UiIsian label="Alasan penyesuaian" petunjuk="Wajib diisi, misalnya opname bulanan, barang rusak, atau salah ambil ukuran.">
        <textarea v-model="alasan"></textarea>
      </UiIsian>
      <UiTombol ukuran="besar" :disabled="menyimpan || !dihitung.length">
        Simpan stok opname ({{ dihitung.length }} varian dihitung)
      </UiTombol>
    </section>
  </UiHalaman>
</template>

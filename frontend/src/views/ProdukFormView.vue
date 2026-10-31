<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { api, pesanError } from '@/api'
import { parseRupiah, ribuan } from '@/format'
import type { Kategori, Produk } from '@/types'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTabel from '@/components/ui/UiTabel.vue'
import UiTombol from '@/components/ui/UiTombol.vue'

const props = defineProps<{ id?: string }>()
const router = useRouter()
const ubah = computed(() => !!props.id)

interface BarisVarian {
  id_varian: number | null
  sku: string
  ukuran: string
  warna: string
  stok: number
  stok_awal: number
  stok_minimum: number
  aktif: boolean
}

const kode = ref('')
const nama = ref('')
const idKategori = ref<number | ''>('')
const hargaBeli = ref('')
const hargaJual = ref('')
const aktif = ref(true)
const varian = ref<BarisVarian[]>([])
const kategori = ref<Kategori[]>([])

const daftarUkuran = ref('S, M, L, XL')
const daftarWarna = ref('')
const kategoriBaru = ref('')

const galat = ref<string | null>(null)
const memproses = ref(false)

function pisah(s: string) {
  return s.split(',').map((x) => x.trim()).filter(Boolean)
}

/** Membuat baris varian untuk setiap kombinasi ukuran x warna yang belum ada. */
function buatKombinasi() {
  for (const ukuran of pisah(daftarUkuran.value)) {
    for (const warna of pisah(daftarWarna.value)) {
      const ada = varian.value.some((v) => v.ukuran.toUpperCase() === ukuran.toUpperCase() && v.warna.toLowerCase() === warna.toLowerCase())
      if (!ada) varian.value.push({ id_varian: null, sku: '', ukuran: ukuran.toUpperCase(), warna, stok: 0, stok_awal: 0, stok_minimum: 2, aktif: true })
    }
  }
}

function tambahBaris() {
  varian.value.push({ id_varian: null, sku: '', ukuran: '', warna: '', stok: 0, stok_awal: 0, stok_minimum: 2, aktif: true })
}

function hapusBaris(i: number) {
  varian.value.splice(i, 1)
}

async function tambahKategori() {
  if (!kategoriBaru.value.trim()) return
  try {
    const k = await api.post<Kategori>('/kategori', { nama_kategori: kategoriBaru.value })
    kategori.value.push(k)
    idKategori.value = k.id_kategori
    kategoriBaru.value = ''
  } catch (e) {
    galat.value = pesanError(e)
  }
}

const rapikan = (s: string) => (s.trim() ? ribuan(parseRupiah(s)) : s)

async function simpan() {
  galat.value = null
  if (!varian.value.length) {
    galat.value = 'Tambahkan minimal satu varian ukuran dan warna.'
    return
  }
  memproses.value = true
  const body = {
    kode: kode.value,
    nama: nama.value,
    id_kategori: idKategori.value,
    harga_beli: parseRupiah(hargaBeli.value),
    harga_jual: parseRupiah(hargaJual.value),
    aktif: aktif.value,
    varian: varian.value.map((v) => ({
      id_varian: v.id_varian,
      sku: v.sku || null,
      ukuran: v.ukuran,
      warna: v.warna,
      stok_awal: v.id_varian ? 0 : Number(v.stok_awal) || 0,
      stok_minimum: Number(v.stok_minimum) || 0,
      aktif: v.aktif,
    })),
  }
  try {
    const p = ubah.value ? await api.put<Produk>(`/produk/${props.id}`, body) : await api.post<Produk>('/produk', body)
    router.push({ path: '/admin/produk', query: { pesan: `${p.nama} disimpan.` } })
  } catch (e) {
    galat.value = pesanError(e)
    window.scrollTo({ top: 0, behavior: 'smooth' })
  } finally {
    memproses.value = false
  }
}

async function hapus() {
  if (!confirm(`Hapus ${nama.value}? Produk yang sudah punya riwayat stok atau transaksi akan dinonaktifkan.`)) return
  try {
    const hasil = await api.del<{ pesan: string }>(`/produk/${props.id}`)
    router.push({ path: '/admin/produk', query: { pesan: hasil.pesan } })
  } catch (e) {
    galat.value = pesanError(e)
  }
}

onMounted(async () => {
  kategori.value = await api.get<Kategori[]>('/kategori').catch(() => [])
  if (!props.id) return
  try {
    const p = await api.get<Produk>(`/produk/${props.id}`)
    kode.value = p.kode
    nama.value = p.nama
    idKategori.value = p.id_kategori
    hargaBeli.value = ribuan(p.harga_beli ?? 0)
    hargaJual.value = ribuan(p.harga_jual)
    aktif.value = p.aktif
    varian.value = p.varian.map((v) => ({ ...v, stok_awal: 0 }))
  } catch (e) {
    galat.value = pesanError(e)
  }
})
</script>

<template>
  <UiHalaman as="form" @submit.prevent="simpan">
    <UiKepalaHalaman :judul="ubah ? `Ubah ${nama || 'produk'}` : 'Produk baru'">
      <template #atas><RouterLink to="/admin/produk" class="text-sm">Kembali ke daftar produk</RouterLink></template>
      <template #aksi>
        <UiTombol v-if="ubah" type="button" varian="bahaya" @click="hapus">Hapus</UiTombol>
        <UiTombol :disabled="memproses">Simpan produk</UiTombol>
      </template>
    </UiKepalaHalaman>

    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <section class="grid gap-4">
      <h2>Data produk</h2>
      <div class="grid grid-cols-[repeat(auto-fit,minmax(220px,1fr))] items-end gap-4">
        <UiIsian label="Kode produk"><input v-model="kode" required placeholder="mis. KMJ-FLN" /></UiIsian>
        <UiIsian label="Nama produk" class="sm:col-span-2"><input v-model="nama" required placeholder="mis. Kemeja Flanel Kotak" /></UiIsian>
        <UiIsian label="Kategori">
          <select v-model="idKategori" required>
            <option value="" disabled>Pilih kategori</option>
            <option v-for="k in kategori" :key="k.id_kategori" :value="k.id_kategori">{{ k.nama_kategori }}</option>
          </select>
        </UiIsian>
        <div class="flex items-end gap-2">
          <UiIsian label="Kategori baru" class="flex-1"><input v-model="kategoriBaru" placeholder="mis. Batik" /></UiIsian>
          <UiTombol type="button" varian="kedua" :disabled="!kategoriBaru.trim()" @click="tambahKategori">Tambah</UiTombol>
        </div>
        <UiIsian label="Harga beli (Rp)">
          <input v-model="hargaBeli" inputmode="numeric" required class="tabular-nums" @blur="hargaBeli = rapikan(hargaBeli)" />
        </UiIsian>
        <UiIsian label="Harga jual (Rp)">
          <input v-model="hargaJual" inputmode="numeric" required class="tabular-nums" @blur="hargaJual = rapikan(hargaJual)" />
        </UiIsian>
        <label class="flex min-h-11 items-center gap-2 font-semibold"><input v-model="aktif" type="checkbox" /> Dijual di kasir</label>
      </div>
    </section>

    <section class="grid gap-4 border-t border-garis pt-8">
      <div>
        <h2>Varian ukuran dan warna</h2>
        <p class="text-sm text-redup">SKU dibuat otomatis dari kode, ukuran, dan warna bila dikosongkan. Isi SKU bila barang sudah memiliki barcode.</p>
      </div>

      <div class="grid items-end gap-3 sm:grid-cols-[1fr_1fr_auto]">
        <UiIsian label="Ukuran" petunjuk="Pisahkan dengan koma"><input v-model="daftarUkuran" placeholder="S, M, L, XL" /></UiIsian>
        <UiIsian label="Warna" petunjuk="Pisahkan dengan koma"><input v-model="daftarWarna" placeholder="Hitam, Putih, Navy" /></UiIsian>
        <UiTombol type="button" varian="kedua" :disabled="!daftarWarna.trim() || !daftarUkuran.trim()" @click="buatKombinasi">
          Buat kombinasi
        </UiTombol>
      </div>

      <UiTabel :kosong="!varian.length ? 'Isi daftar ukuran dan warna di atas lalu pilih Buat kombinasi.' : null" class="[&_td_input]:min-h-[38px] [&_td_input]:px-2 [&_td_input]:py-1.5">
        <thead>
          <tr>
            <th>Ukuran</th>
            <th>Warna</th>
            <th>SKU / barcode</th>
            <th class="text-right">Stok</th>
            <th class="text-right">Stok minimum</th>
            <th>Dijual</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(v, i) in varian" :key="v.id_varian ?? `baru-${i}`" :class="{ '[&>td]:bg-kertas [&>td]:text-redup': !v.aktif }">
            <td><input v-model="v.ukuran" required aria-label="Ukuran" class="w-[90px]" /></td>
            <td><input v-model="v.warna" required aria-label="Warna" /></td>
            <td><input v-model="v.sku" placeholder="Otomatis" aria-label="SKU" /></td>
            <td class="text-right">
              <span v-if="v.id_varian" class="tabular-nums">{{ v.stok }}</span>
              <input
                v-else
                v-model.number="v.stok_awal"
                type="number"
                min="0"
                aria-label="Stok awal"
                class="w-[90px] tabular-nums"
                title="Stok awal"
              />
            </td>
            <td class="text-right">
              <input v-model.number="v.stok_minimum" type="number" min="0" aria-label="Stok minimum" class="w-[90px] tabular-nums" />
            </td>
            <td><input v-model="v.aktif" type="checkbox" aria-label="Dijual" /></td>
            <td>
              <UiTombol v-if="!v.id_varian" type="button" varian="polos" @click="hapusBaris(i)">Hapus</UiTombol>
            </td>
          </tr>
        </tbody>
      </UiTabel>
      <div>
        <UiTombol type="button" varian="polos" @click="tambahBaris">Tambah satu varian</UiTombol>
      </div>
      <p v-if="ubah" class="text-sm text-redup">
        Stok varian yang sudah ada tidak diubah di sini; perubahan stok tercatat lewat barang masuk dan stok opname (siklus berikutnya).
        Varian yang tidak dijual lagi cukup dihilangkan centangnya agar riwayatnya tetap tersimpan.
      </p>
    </section>
  </UiHalaman>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { api, pesanError } from '@/api'
import { parseRupiah, ribuan } from '@/format'
import type { Kategori, Produk } from '@/types'

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
  <form class="halaman" @submit.prevent="simpan">
    <div class="kepala-halaman">
      <div>
        <RouterLink to="/admin/produk" class="kecil">Kembali ke daftar produk</RouterLink>
        <h1>{{ ubah ? `Ubah ${nama || 'produk'}` : 'Produk baru' }}</h1>
      </div>
      <div class="tombol-baris">
        <button v-if="ubah" type="button" class="tombol bahaya" @click="hapus">Hapus</button>
        <button class="tombol" :disabled="memproses">Simpan produk</button>
      </div>
    </div>

    <p v-if="galat" class="pesan galat" role="alert">{{ galat }}</p>

    <section class="panel bagian">
      <h2>Data produk</h2>
      <div class="kisi">
        <label class="isian">Kode produk <input v-model="kode" required placeholder="mis. KMJ-FLN" /></label>
        <label class="isian lebar">Nama produk <input v-model="nama" required placeholder="mis. Kemeja Flanel Kotak" /></label>
        <label class="isian">
          Kategori
          <select v-model="idKategori" required>
            <option value="" disabled>Pilih kategori</option>
            <option v-for="k in kategori" :key="k.id_kategori" :value="k.id_kategori">{{ k.nama_kategori }}</option>
          </select>
        </label>
        <div class="isian kategori-baru">
          <label class="isian">Atau kategori baru <input v-model="kategoriBaru" placeholder="mis. Batik" /></label>
          <button type="button" class="tombol kedua" :disabled="!kategoriBaru.trim()" @click="tambahKategori">Tambah</button>
        </div>
        <label class="isian">Harga beli (Rp) <input v-model="hargaBeli" inputmode="numeric" required class="angka" @blur="hargaBeli = rapikan(hargaBeli)" /></label>
        <label class="isian">Harga jual (Rp) <input v-model="hargaJual" inputmode="numeric" required class="angka" @blur="hargaJual = rapikan(hargaJual)" /></label>
        <label class="centang"><input v-model="aktif" type="checkbox" /> Dijual di kasir</label>
      </div>
    </section>

    <section class="panel bagian">
      <div class="kepala-halaman">
        <div>
          <h2>Varian ukuran dan warna</h2>
          <p class="redup kecil">SKU dibuat otomatis dari kode, ukuran, dan warna bila dikosongkan. Isi SKU bila barang sudah memiliki barcode.</p>
        </div>
      </div>

      <div class="kombinasi">
        <label class="isian">Ukuran <small>Pisahkan dengan koma</small><input v-model="daftarUkuran" placeholder="S, M, L, XL" /></label>
        <label class="isian">Warna <small>Pisahkan dengan koma</small><input v-model="daftarWarna" placeholder="Hitam, Putih, Navy" /></label>
        <button type="button" class="tombol kedua" :disabled="!daftarWarna.trim() || !daftarUkuran.trim()" @click="buatKombinasi">
          Buat kombinasi
        </button>
      </div>

      <div class="tabel-bungkus">
        <table>
          <thead>
            <tr>
              <th>Ukuran</th>
              <th>Warna</th>
              <th>SKU / barcode</th>
              <th class="kanan">Stok</th>
              <th class="kanan">Stok minimum</th>
              <th>Dijual</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(v, i) in varian" :key="v.id_varian ?? `baru-${i}`" :class="{ mati: !v.aktif }">
              <td><input v-model="v.ukuran" required aria-label="Ukuran" class="pendek" /></td>
              <td><input v-model="v.warna" required aria-label="Warna" /></td>
              <td><input v-model="v.sku" placeholder="Otomatis" aria-label="SKU" /></td>
              <td class="kanan">
                <span v-if="v.id_varian" class="angka">{{ v.stok }}</span>
                <input v-else v-model.number="v.stok_awal" type="number" min="0" aria-label="Stok awal" class="pendek angka" title="Stok awal" />
              </td>
              <td class="kanan"><input v-model.number="v.stok_minimum" type="number" min="0" aria-label="Stok minimum" class="pendek angka" /></td>
              <td><input v-model="v.aktif" type="checkbox" aria-label="Dijual" /></td>
              <td>
                <button v-if="!v.id_varian" type="button" class="tombol polos" @click="hapusBaris(i)">Hapus</button>
              </td>
            </tr>
          </tbody>
        </table>
        <p v-if="!varian.length" class="kosong">Isi daftar ukuran dan warna di atas lalu pilih Buat kombinasi.</p>
      </div>
      <div>
        <button type="button" class="tombol polos" @click="tambahBaris">Tambah satu varian</button>
      </div>
      <p v-if="ubah" class="kecil redup">
        Stok varian yang sudah ada tidak diubah di sini; perubahan stok tercatat lewat barang masuk dan stok opname (siklus berikutnya).
        Varian yang tidak dijual lagi cukup dihilangkan centangnya agar riwayatnya tetap tersimpan.
      </p>
    </section>
  </form>
</template>

<style scoped>
.bagian {
  display: grid;
  gap: 16px;
  padding: 20px;
}
.kisi {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 16px;
  align-items: end;
}
.kisi .lebar {
  grid-column: span 2;
}
.kategori-baru {
  display: flex;
  gap: 8px;
  align-items: end;
}
.kategori-baru .isian {
  flex: 1;
}
.centang {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 44px;
  font-weight: 600;
}
.kombinasi {
  display: grid;
  grid-template-columns: 1fr 1fr auto;
  gap: 12px;
  align-items: end;
}
td input {
  min-height: 38px;
  padding: 6px 8px;
}
input.pendek {
  width: 90px;
}
tr.mati td {
  color: var(--redup);
  background: var(--kertas);
}
.tombol-baris {
  display: flex;
  gap: 10px;
}
@media (max-width: 640px) {
  .kisi .lebar {
    grid-column: auto;
  }
  .kombinasi {
    grid-template-columns: 1fr;
  }
}
</style>

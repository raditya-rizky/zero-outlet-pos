<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { api, ApiError, pesanError } from '@/api'
import { useKeranjang } from '@/stores/keranjang'
import { useSesi } from '@/stores/sesi'
import { parseRupiah, ribuan, rupiah } from '@/format'
import LabelHarga from '@/components/LabelHarga.vue'
import DialogModal from '@/components/DialogModal.vue'
import type { MetodeBayar, Produk, Struk, Varian } from '@/types'

const router = useRouter()
const keranjang = useKeranjang()
const sesi = useSesi()

// ---------- Pencarian produk (KF-05) ----------
const cari = ref('')
const hasil = ref<Produk[]>([])
const memuat = ref(false)
const galatCari = ref<string | null>(null)
const kotakCari = ref<HTMLInputElement>()
let jedaCari: ReturnType<typeof setTimeout> | undefined
let nomorPermintaan = 0

async function muatProduk() {
  const nomor = ++nomorPermintaan
  memuat.value = true
  galatCari.value = null
  try {
    const data = await api.get<Produk[]>('/produk', { cari: cari.value })
    if (nomor === nomorPermintaan) hasil.value = data
  } catch (e) {
    if (nomor === nomorPermintaan) galatCari.value = pesanError(e)
  } finally {
    if (nomor === nomorPermintaan) memuat.value = false
  }
}

watch(cari, () => {
  clearTimeout(jedaCari)
  jedaCari = setTimeout(muatProduk, 250)
})

/** Enter pada kotak pencarian: coba sebagai SKU/barcode (pemindai USB mengetik lalu menekan Enter). */
async function enterCari() {
  const kata = cari.value.trim()
  if (!kata || /\s/.test(kata)) return
  clearTimeout(jedaCari)
  try {
    const { produk, varian } = await api.get<{ produk: Produk; varian: Varian }>(`/varian/sku/${encodeURIComponent(kata)}`)
    if (keranjang.tambah(produk, varian)) cari.value = ''
  } catch (e) {
    if (e instanceof ApiError && e.status === 404) muatProduk()
    else keranjang.pesan = pesanError(e)
  }
}

function sisaStok(v: Varian) {
  const diKeranjang = keranjang.items.find((i) => i.id_varian === v.id_varian)?.jumlah ?? 0
  return v.stok - diKeranjang
}

function tambah(p: Produk, v: Varian) {
  keranjang.tambah(p, v)
}

// ---------- Pembayaran (KF-08) ----------
const dialogBayar = ref(false)
const metode = ref<MetodeBayar>('tunai')
const uangDiterima = ref('')
const menyimpan = ref(false)
const galatBayar = ref<string | null>(null)
const isianUang = ref<HTMLInputElement>()

const diterima = computed(() => parseRupiah(uangDiterima.value))
const kembalian = computed(() => diterima.value - keranjang.total)
const bisaSimpan = computed(
  () => keranjang.items.length > 0 && !menyimpan.value && (metode.value !== 'tunai' || kembalian.value >= 0),
)

/** Nominal uang yang lazim diterima: uang pas lalu pembulatan ke atas. */
const nominalCepat = computed(() => {
  const t = keranjang.total
  const calon = [t, ...[10_000, 50_000, 100_000].map((k) => Math.ceil(t / k) * k)]
  return [...new Set(calon)].filter((n) => n >= t).slice(0, 4)
})

function bukaBayar() {
  if (!keranjang.items.length) return
  galatBayar.value = null
  uangDiterima.value = ''
  metode.value = 'tunai'
  dialogBayar.value = true
  nextTick(() => isianUang.value?.focus())
}

watch(metode, (m) => {
  if (m === 'tunai') nextTick(() => isianUang.value?.focus())
})

async function simpan() {
  if (!bisaSimpan.value) return
  menyimpan.value = true
  galatBayar.value = null
  try {
    const struk = await api.post<Struk>('/transaksi', {
      items: keranjang.items.map((i) => ({ id_varian: i.id_varian, jumlah: i.jumlah })),
      metode_bayar: metode.value,
      bayar: metode.value === 'tunai' ? diterima.value : null,
    })
    keranjang.kosongkan()
    dialogBayar.value = false
    sesi.muat().catch(() => undefined)
    router.push({ path: `/struk/${struk.id_transaksi}`, query: { baru: '1' } })
  } catch (e) {
    galatBayar.value = pesanError(e)
    // Stok mungkin berubah karena kasir lain; segarkan daftar produk.
    if (e instanceof ApiError && e.status === 409) muatProduk()
  } finally {
    menyimpan.value = false
  }
}

// ---------- Pintasan keyboard ----------
function pintasan(e: KeyboardEvent) {
  if (e.key === 'F2') {
    e.preventDefault()
    if (!dialogBayar.value) bukaBayar()
  } else if (e.key === 'F3') {
    e.preventDefault()
    kotakCari.value?.focus()
    kotakCari.value?.select()
  }
}

onMounted(() => {
  muatProduk()
  window.addEventListener('keydown', pintasan)
})
onBeforeUnmount(() => window.removeEventListener('keydown', pintasan))
</script>

<template>
  <div class="kasir">
    <!-- Kolom kiri: pencarian dan daftar barang -->
    <section class="katalog" aria-label="Cari barang">
      <div class="cari">
        <input
          ref="kotakCari"
          v-model="cari"
          type="search"
          placeholder="Cari nama, kode, warna, atau pindai barcode"
          aria-label="Cari barang"
          autofocus
          @keydown.enter.prevent="enterCari"
        />
        <span class="kecil redup pintas">F3</span>
      </div>

      <p v-if="galatCari" class="pesan galat">{{ galatCari }}</p>
      <p v-else-if="!memuat && !hasil.length" class="kosong">
        Tidak ada barang yang cocok dengan “{{ cari }}”. Coba kata lain atau periksa ejaan.
      </p>

      <ul class="daftar" :class="{ memuat }">
        <li v-for="p in hasil" :key="p.id_produk" class="produk">
          <div class="judul-produk">
            <div>
              <h3>{{ p.nama }}</h3>
              <span class="kecil redup">{{ p.kode }} / {{ p.nama_kategori }}</span>
            </div>
            <span class="harga angka">{{ rupiah(p.harga_jual) }}</span>
          </div>
          <div class="varian" role="group" :aria-label="`Varian ${p.nama}`">
            <button
              v-for="v in p.varian"
              :key="v.id_varian"
              type="button"
              class="chip"
              :class="{ menipis: sisaStok(v) > 0 && v.stok <= v.stok_minimum }"
              :disabled="sisaStok(v) <= 0"
              :title="`${v.sku}, stok ${v.stok}`"
              @click="tambah(p, v)"
            >
              <strong>{{ v.ukuran }}</strong>
              <span>{{ v.warna }}</span>
              <span class="stok angka">{{ sisaStok(v) > 0 ? sisaStok(v) : 'habis' }}</span>
            </button>
          </div>
        </li>
      </ul>
    </section>

    <!-- Kolom kanan: keranjang -->
    <section class="keranjang" aria-label="Keranjang belanja">
      <header class="kepala-keranjang">
        <h2>Keranjang</h2>
        <button v-if="keranjang.items.length" class="tombol polos kecil" @click="keranjang.kosongkan()">Kosongkan</button>
      </header>

      <p v-if="keranjang.pesan" class="pesan info" role="alert">{{ keranjang.pesan }}</p>

      <p v-if="!keranjang.items.length" class="kosong kecil">
        Pilih ukuran dan warna barang di sebelah kiri, atau pindai barcode, untuk menambahkannya ke keranjang.
      </p>

      <ul class="item-keranjang">
        <li v-for="item in keranjang.items" :key="item.id_varian">
          <div class="nama-item">
            <strong>{{ item.nama }}</strong>
            <span class="kecil redup">{{ item.ukuran }}, {{ item.warna }} / {{ rupiah(item.harga) }}</span>
          </div>
          <div class="atur">
            <div class="jumlah">
              <button type="button" :aria-label="`Kurangi ${item.nama}`" @click="keranjang.ubahJumlah(item.id_varian, item.jumlah - 1)">−</button>
              <input
                :value="item.jumlah"
                inputmode="numeric"
                class="angka"
                :aria-label="`Jumlah ${item.nama}`"
                @change="keranjang.ubahJumlah(item.id_varian, Number(($event.target as HTMLInputElement).value) || 0)"
              />
              <button
                type="button"
                :aria-label="`Tambah ${item.nama}`"
                :disabled="item.jumlah >= item.stok"
                @click="keranjang.ubahJumlah(item.id_varian, item.jumlah + 1)"
              >
                +
              </button>
            </div>
            <span class="subtotal angka">{{ ribuan(item.jumlah * item.harga) }}</span>
            <button type="button" class="hapus" :aria-label="`Hapus ${item.nama}`" @click="keranjang.hapus(item.id_varian)">
              ×
            </button>
          </div>
        </li>
      </ul>

      <footer class="bayar">
        <LabelHarga :nilai="keranjang.total" :keterangan="`Total ${keranjang.jumlahBarang} barang`" />
        <button class="tombol besar" :disabled="!keranjang.items.length" @click="bukaBayar">Bayar <span class="kunci">F2</span></button>
      </footer>
    </section>

    <DialogModal v-if="dialogBayar" judul="Pembayaran" @tutup="dialogBayar = false">
      <form class="form-bayar" @submit.prevent="simpan">
        <LabelHarga :nilai="keranjang.total" keterangan="Total belanja" />

        <fieldset class="metode">
          <legend class="kecil redup">Metode pembayaran</legend>
          <label v-for="m in (['tunai', 'transfer', 'qris'] as const)" :key="m" :class="{ dipilih: metode === m }">
            <input v-model="metode" type="radio" name="metode" :value="m" />
            {{ m === 'qris' ? 'QRIS' : m === 'tunai' ? 'Tunai' : 'Transfer' }}
          </label>
        </fieldset>

        <template v-if="metode === 'tunai'">
          <label class="isian">
            Uang diterima
            <input ref="isianUang" v-model="uangDiterima" inputmode="numeric" class="angka uang" placeholder="0" />
          </label>
          <div class="cepat">
            <button v-for="n in nominalCepat" :key="n" type="button" class="tombol kedua" @click="uangDiterima = ribuan(n)">
              {{ n === keranjang.total ? 'Uang pas' : rupiah(n) }}
            </button>
          </div>
          <div class="kembalian" :class="{ kurang: uangDiterima && kembalian < 0 }">
            <span>{{ kembalian < 0 ? 'Kurang' : 'Kembalian' }}</span>
            <strong class="angka">{{ rupiah(Math.abs(uangDiterima ? kembalian : 0)) }}</strong>
          </div>
        </template>
        <p v-else class="pesan info">
          Pastikan dana {{ metode === 'qris' ? 'QRIS' : 'transfer' }} sebesar {{ rupiah(keranjang.total) }} sudah masuk sebelum
          menyimpan transaksi.
        </p>

        <p v-if="galatBayar" class="pesan galat" role="alert">{{ galatBayar }}</p>
        <button class="tombol besar" :disabled="!bisaSimpan">{{ menyimpan ? 'Menyimpan…' : 'Simpan transaksi' }}</button>
      </form>
    </DialogModal>
  </div>
</template>

<style scoped>
.kasir {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(340px, 420px);
  height: calc(100dvh - 60px);
}

/* ---------- Katalog ---------- */
.katalog {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  overflow: hidden;
}
.cari {
  position: relative;
}
.cari input {
  min-height: 52px;
  padding-right: 48px;
  font-size: 1.05rem;
}
.pintas {
  position: absolute;
  right: 14px;
  top: 50%;
  transform: translateY(-50%);
}
.daftar {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  background: var(--putih);
  border: 1px solid var(--garis);
  border-radius: var(--r);
}
.daftar.memuat {
  opacity: 0.6;
}
.produk {
  display: grid;
  gap: 10px;
  padding: 12px 14px;
  border-bottom: 1px solid var(--garis);
}
.produk:last-child {
  border-bottom: 0;
}
.judul-produk {
  display: flex;
  justify-content: space-between;
  align-items: start;
  gap: 12px;
}
.harga {
  font-weight: 700;
  white-space: nowrap;
}
.varian {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-height: 40px;
  padding: 4px 6px 4px 10px;
  border: 1px solid var(--garis-kuat);
  border-radius: var(--r-kecil);
  background: var(--putih);
  color: var(--tinta);
  font-size: 0.9rem;
}
.chip:hover:not(:disabled) {
  border-color: var(--denim);
  background: var(--denim-terang);
}
.chip:active:not(:disabled) {
  transform: translateY(1px);
}
.chip .stok {
  min-width: 26px;
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--kertas);
  color: var(--redup);
  font-size: 0.8rem;
  text-align: center;
}
.chip.menipis .stok {
  background: var(--waspada-terang);
  color: var(--waspada);
  font-weight: 700;
}
.chip:disabled {
  opacity: 1;
  color: var(--redup);
  background: var(--kertas);
  border-style: dashed;
}

/* ---------- Keranjang ---------- */
.keranjang {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  background: var(--putih);
  border-left: 1px solid var(--garis);
  overflow: hidden;
}
.kepala-keranjang {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.kepala-keranjang .tombol {
  min-height: 32px;
}
.item-keranjang {
  list-style: none;
  margin: 0;
  padding: 0;
  flex: 1;
  overflow-y: auto;
}
.item-keranjang li {
  display: grid;
  gap: 8px;
  padding: 10px 0;
  border-bottom: 1px dashed var(--garis-kuat);
}
.nama-item {
  display: grid;
}
.atur {
  display: flex;
  align-items: center;
  gap: 10px;
}
.jumlah {
  display: flex;
  border: 1px solid var(--garis-kuat);
  border-radius: var(--r-kecil);
  overflow: hidden;
}
.jumlah button {
  width: 40px;
  border: 0;
  background: var(--kertas);
  font-size: 1.2rem;
  color: var(--tinta);
}
.jumlah input {
  width: 52px;
  min-height: 40px;
  border: 0;
  border-radius: 0;
  text-align: center;
  font-weight: 700;
}
.subtotal {
  margin-left: auto;
  font-weight: 700;
}
.hapus {
  width: 36px;
  height: 36px;
  border: 0;
  border-radius: var(--r-kecil);
  background: transparent;
  color: var(--redup);
  font-size: 1.4rem;
  line-height: 1;
}
.hapus:hover {
  background: var(--bahaya-terang);
  color: var(--bahaya);
}
.bayar {
  display: grid;
  gap: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--garis);
}
.kunci {
  padding: 1px 6px;
  border-radius: 4px;
  background: rgb(255 255 255 / 0.2);
  font-size: 0.8rem;
}

/* ---------- Dialog bayar ---------- */
.form-bayar {
  display: grid;
  gap: 16px;
}
.metode {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
  margin: 0;
  padding: 0;
  border: 0;
}
.metode legend {
  margin-bottom: 6px;
}
.metode label {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 48px;
  border: 1px solid var(--garis-kuat);
  border-radius: var(--r-kecil);
  font-weight: 600;
  cursor: pointer;
}
.metode label.dipilih {
  border-color: var(--denim);
  background: var(--denim);
  color: var(--putih);
}
.metode label:focus-within {
  box-shadow: var(--fokus);
}
.metode input {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}
.uang {
  font-size: 1.6rem;
  font-weight: 700;
  min-height: 56px;
}
.cepat {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
  gap: 8px;
}
.kembalian {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  padding: 12px 14px;
  border-radius: var(--r-kecil);
  background: var(--aman-terang);
  color: var(--aman);
  font-weight: 600;
}
.kembalian strong {
  font-size: 1.5rem;
  font-stretch: 115%;
}
.kembalian.kurang {
  background: var(--bahaya-terang);
  color: var(--bahaya);
}

@media (max-width: 860px) {
  .kasir {
    grid-template-columns: 1fr;
    height: auto;
  }
  .katalog,
  .keranjang {
    overflow: visible;
  }
  .daftar {
    max-height: 60dvh;
  }
  .keranjang {
    border-left: 0;
    border-top: 1px solid var(--garis);
  }
  .bayar {
    position: sticky;
    bottom: 0;
    background: var(--putih);
    padding-bottom: 12px;
  }
}
</style>

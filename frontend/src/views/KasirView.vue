<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { api, ApiError, pesanError } from '@/api'
import { useKeranjang } from '@/stores/keranjang'
import { useSesi } from '@/stores/sesi'
import { parseRupiah, ribuan, rupiah } from '@/format'
import LabelHarga from '@/components/LabelHarga.vue'
import DialogModal from '@/components/DialogModal.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTombol from '@/components/ui/UiTombol.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
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
  <div class="grid min-[860px]:h-[calc(100dvh-60px)] min-[860px]:grid-cols-[minmax(0,1fr)_minmax(340px,420px)]">
    <!-- Kolom kiri: pencarian dan daftar barang -->
    <section class="flex flex-col gap-3 p-4 min-[860px]:overflow-hidden" aria-label="Cari barang">
      <div class="relative">
        <input
          ref="kotakCari"
          v-model="cari"
          type="search"
          class="min-h-[52px] pr-12 text-[1.05rem]"
          placeholder="Cari nama, kode, warna, atau pindai barcode"
          aria-label="Cari barang"
          autofocus
          @keydown.enter.prevent="enterCari"
        />
        <span class="absolute top-1/2 right-3.5 -translate-y-1/2 text-sm text-redup">F3</span>
      </div>

      <UiPesan v-if="galatCari" jenis="galat">{{ galatCari }}</UiPesan>
      <p v-else-if="!memuat && !hasil.length" class="px-4 py-10 text-center text-redup">
        Tidak ada barang yang cocok dengan “{{ cari }}”. Coba kata lain atau periksa ejaan.
      </p>

      <ul
        class="overflow-y-auto rounded-sedang border border-garis bg-white max-[859px]:max-h-[60dvh]"
        :class="{ 'opacity-60': memuat }"
      >
        <li v-for="p in hasil" :key="p.id_produk" class="grid gap-2.5 border-b border-garis px-3.5 py-3 last:border-b-0">
          <div class="flex items-start justify-between gap-3">
            <div>
              <h3>{{ p.nama }}</h3>
              <span class="text-sm text-redup">{{ p.kode }} / {{ p.nama_kategori }}</span>
            </div>
            <span class="font-bold whitespace-nowrap tabular-nums">{{ rupiah(p.harga_jual) }}</span>
          </div>
          <div class="flex flex-wrap gap-1.5" role="group" :aria-label="`Varian ${p.nama}`">
            <button
              v-for="v in p.varian"
              :key="v.id_varian"
              type="button"
              class="inline-flex min-h-10 items-center gap-1.5 rounded-kecil border border-garis-kuat bg-white py-1 pr-1.5 pl-2.5 text-[0.9rem] text-tinta hover:enabled:border-denim hover:enabled:bg-denim-terang active:enabled:translate-y-px disabled:border-dashed disabled:bg-kertas disabled:text-redup disabled:opacity-100"
              :disabled="sisaStok(v) <= 0"
              :title="`${v.sku}, stok ${v.stok}`"
              @click="tambah(p, v)"
            >
              <strong>{{ v.ukuran }}</strong>
              <span>{{ v.warna }}</span>
              <span
                class="min-w-[26px] rounded-[4px] px-1.5 py-px text-center text-[0.8rem] tabular-nums"
                :class="sisaStok(v) > 0 && v.stok <= v.stok_minimum ? 'bg-waspada-terang font-bold text-waspada' : 'bg-kertas text-redup'"
              >
                {{ sisaStok(v) > 0 ? sisaStok(v) : 'habis' }}
              </span>
            </button>
          </div>
        </li>
      </ul>
    </section>

    <!-- Kolom kanan: keranjang -->
    <section
      class="flex flex-col gap-3 border-t border-garis bg-white p-4 min-[860px]:overflow-hidden min-[860px]:border-t-0 min-[860px]:border-l"
      aria-label="Keranjang belanja"
    >
      <header class="flex items-center justify-between">
        <h2>Keranjang</h2>
        <UiTombol v-if="keranjang.items.length" varian="polos" ukuran="kecil" class="text-sm" @click="keranjang.kosongkan()">
          Kosongkan
        </UiTombol>
      </header>

      <UiPesan v-if="keranjang.pesan" role="alert">{{ keranjang.pesan }}</UiPesan>

      <p v-if="!keranjang.items.length" class="px-4 py-10 text-center text-sm text-redup">
        Pilih ukuran dan warna barang di sebelah kiri, atau pindai barcode, untuk menambahkannya ke keranjang.
      </p>

      <ul class="flex-1 overflow-y-auto">
        <li v-for="item in keranjang.items" :key="item.id_varian" class="grid gap-2 border-b border-dashed border-garis-kuat py-2.5">
          <div class="grid">
            <strong>{{ item.nama }}</strong>
            <span class="text-sm text-redup">{{ item.ukuran }}, {{ item.warna }} / {{ rupiah(item.harga) }}</span>
          </div>
          <div class="flex items-center gap-2.5">
            <div class="flex overflow-hidden rounded-kecil border border-garis-kuat">
              <button
                type="button"
                class="w-10 bg-kertas text-xl text-tinta"
                :aria-label="`Kurangi ${item.nama}`"
                @click="keranjang.ubahJumlah(item.id_varian, item.jumlah - 1)"
              >
                −
              </button>
              <input
                :value="item.jumlah"
                inputmode="numeric"
                class="min-h-10 w-[52px] rounded-none border-0 text-center font-bold tabular-nums"
                :aria-label="`Jumlah ${item.nama}`"
                @change="keranjang.ubahJumlah(item.id_varian, Number(($event.target as HTMLInputElement).value) || 0)"
              />
              <button
                type="button"
                class="w-10 bg-kertas text-xl text-tinta"
                :aria-label="`Tambah ${item.nama}`"
                :disabled="item.jumlah >= item.stok"
                @click="keranjang.ubahJumlah(item.id_varian, item.jumlah + 1)"
              >
                +
              </button>
            </div>
            <span class="ml-auto font-bold tabular-nums">{{ ribuan(item.jumlah * item.harga) }}</span>
            <button
              type="button"
              class="size-9 rounded-kecil text-[1.4rem] leading-none text-redup hover:bg-bahaya-terang hover:text-bahaya"
              :aria-label="`Hapus ${item.nama}`"
              @click="keranjang.hapus(item.id_varian)"
            >
              ×
            </button>
          </div>
        </li>
      </ul>

      <footer class="grid gap-3 border-t border-garis pt-3 max-[859px]:sticky max-[859px]:bottom-0 max-[859px]:bg-white max-[859px]:pb-3">
        <LabelHarga :nilai="keranjang.total" :keterangan="`Total ${keranjang.jumlahBarang} barang`" />
        <UiTombol ukuran="besar" :disabled="!keranjang.items.length" @click="bukaBayar">
          Bayar <span class="rounded-[4px] bg-white/20 px-1.5 py-px text-[0.8rem]">F2</span>
        </UiTombol>
      </footer>
    </section>

    <DialogModal v-if="dialogBayar" judul="Pembayaran" @tutup="dialogBayar = false">
      <form class="grid gap-4" @submit.prevent="simpan">
        <LabelHarga :nilai="keranjang.total" keterangan="Total belanja" />

        <fieldset class="grid grid-cols-3 gap-2">
          <legend class="mb-1.5 text-sm text-redup">Metode pembayaran</legend>
          <label
            v-for="m in (['tunai', 'transfer', 'qris'] as const)"
            :key="m"
            class="flex min-h-12 cursor-pointer items-center justify-center rounded-kecil border border-garis-kuat font-semibold has-checked:border-denim has-checked:bg-denim has-checked:text-white has-focus-visible:shadow-fokus"
          >
            <input v-model="metode" type="radio" name="metode" :value="m" class="sr-only" />
            {{ m === 'qris' ? 'QRIS' : m === 'tunai' ? 'Tunai' : 'Transfer' }}
          </label>
        </fieldset>

        <template v-if="metode === 'tunai'">
          <UiIsian label="Uang diterima">
            <input ref="isianUang" v-model="uangDiterima" inputmode="numeric" class="min-h-14 text-[1.6rem] font-bold tabular-nums" placeholder="0" />
          </UiIsian>
          <div class="grid grid-cols-[repeat(auto-fit,minmax(100px,1fr))] gap-2">
            <UiTombol v-for="n in nominalCepat" :key="n" type="button" varian="kedua" @click="uangDiterima = ribuan(n)">
              {{ n === keranjang.total ? 'Uang pas' : rupiah(n) }}
            </UiTombol>
          </div>
          <div
            class="flex items-baseline justify-between rounded-kecil px-3.5 py-3 font-semibold"
            :class="uangDiterima && kembalian < 0 ? 'bg-bahaya-terang text-bahaya' : 'bg-aman-terang text-aman'"
          >
            <span>{{ uangDiterima && kembalian < 0 ? 'Kurang' : 'Kembalian' }}</span>
            <strong class="text-2xl font-stretch-115% tabular-nums">{{ rupiah(Math.abs(uangDiterima ? kembalian : 0)) }}</strong>
          </div>
        </template>
        <UiPesan v-else>
          Pastikan dana {{ metode === 'qris' ? 'QRIS' : 'transfer' }} sebesar {{ rupiah(keranjang.total) }} sudah masuk sebelum
          menyimpan transaksi.
        </UiPesan>

        <UiPesan v-if="galatBayar" jenis="galat">{{ galatBayar }}</UiPesan>
        <UiTombol ukuran="besar" :disabled="!bisaSimpan">{{ menyimpan ? 'Menyimpan…' : 'Simpan transaksi' }}</UiTombol>
      </form>
    </DialogModal>
  </div>
</template>

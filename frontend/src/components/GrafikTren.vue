<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { ringkas, rupiah, tanggalBulan, tanggalPendek, tanggalSaja } from '@/format'
import type { TitikTren } from '@/types'

// Grafik batang satu seri: total penjualan per hari/bulan (setelah diskon,
// sebelum retur). Satu warna (denim), batang tipis dengan ujung membulat,
// tooltip saat diarahkan/difokus, dan tabel sebagai tampilan alternatif.
const props = defineProps<{ titik: TitikTren[]; kelompok: 'hari' | 'bulan'; judul: string }>()

const wadah = ref<HTMLDivElement>()
const lebar = ref(320)
const TINGGI = 220
const KIRI = 52
const BAWAH = 26
const ATAS = 20
let pengamat: ResizeObserver | undefined
onMounted(() => {
  pengamat = new ResizeObserver(([e]) => (lebar.value = Math.max(280, Math.floor(e!.contentRect.width))))
  if (wadah.value) pengamat.observe(wadah.value)
})
onBeforeUnmount(() => pengamat?.disconnect())

/** Kelipatan "bulat" (1, 2, 5 x 10^n) untuk sekitar empat garis bantu. */
function langkahBulat(maks: number) {
  if (maks <= 0) return 1
  const kasar = maks / 4
  const pangkat = 10 ** Math.floor(Math.log10(kasar))
  return [1, 2, 5, 10].map((k) => k * pangkat).find((k) => k >= kasar)!
}

const maksimum = computed(() => Math.max(0, ...props.titik.map((t) => t.total)))
const langkah = computed(() => langkahBulat(maksimum.value))
const puncak = computed(() => Math.max(langkah.value, Math.ceil(maksimum.value / langkah.value) * langkah.value))
const garis = computed(() => Array.from({ length: Math.round(puncak.value / langkah.value) + 1 }, (_, i) => i * langkah.value))

const lebarDalam = computed(() => lebar.value - KIRI - 8)
const tinggiDalam = TINGGI - ATAS - BAWAH
const pita = computed(() => lebarDalam.value / Math.max(1, props.titik.length))
const lebarBatang = computed(() => Math.max(2, Math.min(24, pita.value - 2)))
const y = (nilai: number) => ATAS + tinggiDalam - (nilai / puncak.value) * tinggiDalam
const xTengah = (i: number) => KIRI + pita.value * i + pita.value / 2

/** Batang dengan ujung atas membulat 4px dan dasar persegi. */
function jalurBatang(i: number, nilai: number) {
  const w = lebarBatang.value
  const x = xTengah(i) - w / 2
  const atas = y(nilai)
  const dasar = ATAS + tinggiDalam
  const r = Math.min(4, w / 2, dasar - atas)
  return `M${x},${dasar} V${atas + r} Q${x},${atas} ${x + r},${atas} H${x + w - r} Q${x + w},${atas} ${x + w},${atas + r} V${dasar} Z`
}

const formatSumbu = (iso: string) => (props.kelompok === 'bulan' ? tanggalBulan(iso) : tanggalPendek(iso))
const setiap = computed(() => Math.max(1, Math.ceil(props.titik.length / Math.max(2, Math.floor(lebarDalam.value / 64)))))
const indeksPuncak = computed(() => (maksimum.value > 0 ? props.titik.findIndex((t) => t.total === maksimum.value) : -1))

const aktif = ref<number | null>(null)
const tooltip = computed(() => {
  if (aktif.value === null) return null
  const t = props.titik[aktif.value]!
  const x = Math.min(Math.max(xTengah(aktif.value), 90), lebar.value - 90)
  return { t, x, y: y(t.total) }
})
</script>

<template>
  <figure class="grid min-w-0 gap-2">
    <figcaption class="text-sm text-redup">{{ judul }}</figcaption>
    <!-- Lebar diambil dari wadah (bukan dari SVG) agar grafik menyusut di layar sempit. -->
    <div ref="wadah" class="relative w-full min-w-0 overflow-x-clip" @pointerleave="aktif = null">
      <svg :width="lebar" :height="TINGGI" :viewBox="`0 0 ${lebar} ${TINGGI}`" role="img" :aria-label="judul" class="block">
        <g aria-hidden="true">
          <template v-for="g in garis" :key="g">
            <line :x1="KIRI" :x2="lebar - 8" :y1="y(g)" :y2="y(g)" stroke="var(--color-garis)" stroke-width="1" />
            <text :x="KIRI - 8" :y="y(g) + 4" text-anchor="end" class="fill-redup text-[11px] tabular-nums">{{ ringkas(g) }}</text>
          </template>
          <template v-for="(t, i) in titik" :key="t.tanggal">
            <text
              v-if="i % setiap === 0"
              :x="xTengah(i)"
              :y="TINGGI - 8"
              text-anchor="middle"
              class="fill-redup text-[11px]"
            >
              {{ formatSumbu(t.tanggal) }}
            </text>
          </template>
        </g>
        <g>
          <g v-for="(t, i) in titik" :key="t.tanggal">
            <path
              v-if="t.total > 0"
              :d="jalurBatang(i, t.total)"
              :fill="aktif === i ? 'var(--color-indigo)' : 'var(--color-denim)'"
            />
            <text
              v-if="i === indeksPuncak"
              :x="xTengah(i)"
              :y="y(t.total) - 6"
              text-anchor="middle"
              class="fill-tinta text-[11px] font-semibold tabular-nums"
              aria-hidden="true"
            >
              {{ ringkas(t.total) }}
            </text>
            <!-- Area sentuh selebar pita dan setinggi grafik, lebih besar dari batangnya. -->
            <rect
              :x="KIRI + pita * i"
              :y="ATAS"
              :width="pita"
              :height="tinggiDalam"
              fill="transparent"
              tabindex="0"
              class="outline-none focus-visible:fill-denim/10"
              :aria-label="`${tanggalSaja(t.tanggal)}: ${rupiah(t.total)}, ${t.transaksi} transaksi`"
              @pointerenter="aktif = i"
              @focus="aktif = i"
              @blur="aktif = null"
            />
          </g>
        </g>
      </svg>
      <div
        v-if="tooltip"
        class="pointer-events-none absolute z-10 -translate-x-1/2 -translate-y-full rounded-kecil border border-garis bg-white px-3 py-2 text-sm shadow-dialog"
        :style="{ left: `${tooltip.x}px`, top: `${tooltip.y - 8}px` }"
        role="status"
      >
        <strong class="block tabular-nums">{{ rupiah(tooltip.t.total) }}</strong>
        <span class="whitespace-nowrap text-redup">
          {{ kelompok === 'bulan' ? tanggalBulan(tooltip.t.tanggal) : tanggalSaja(tooltip.t.tanggal) }}, {{ tooltip.t.transaksi }} transaksi
        </span>
      </div>
    </div>
    <details class="text-sm print:hidden">
      <summary class="cursor-pointer text-denim">Tampilkan sebagai tabel</summary>
      <table class="mt-2 w-full">
        <thead><tr><th>{{ kelompok === 'bulan' ? 'Bulan' : 'Tanggal' }}</th><th class="text-right">Transaksi</th><th class="text-right">Total</th></tr></thead>
        <tbody>
          <tr v-for="t in titik" :key="t.tanggal">
            <td>{{ kelompok === 'bulan' ? tanggalBulan(t.tanggal) : tanggalSaja(t.tanggal) }}</td>
            <td class="text-right tabular-nums">{{ t.transaksi }}</td>
            <td class="text-right tabular-nums">{{ rupiah(t.total) }}</td>
          </tr>
        </tbody>
      </table>
    </details>
  </figure>
</template>

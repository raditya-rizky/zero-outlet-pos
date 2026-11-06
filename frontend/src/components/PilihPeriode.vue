<script setup lang="ts">
import { ref } from 'vue'
import { rentangCepat, type Periode, type PilihanPeriode } from '@/periode'

// Pemilih periode laporan dengan pilihan cepat dan rentang bebas.
const periode = defineModel<Periode>({ required: true })
// Pilihan yang disorot di awal harus sama dengan periode awal milik halaman.
const props = withDefaults(defineProps<{ awal?: PilihanPeriode }>(), { awal: 'bulan' })
const pilihan = ref<PilihanPeriode>(props.awal)

function pilih(p: PilihanPeriode) {
  pilihan.value = p
  if (p !== 'bebas') periode.value = rentangCepat(p)
}

const daftar: { nilai: PilihanPeriode; label: string }[] = [
  { nilai: 'hari', label: 'Hari ini' },
  { nilai: 'minggu', label: 'Minggu ini' },
  { nilai: 'bulan', label: 'Bulan ini' },
  { nilai: 'bulan_lalu', label: 'Bulan lalu' },
  { nilai: 'bebas', label: 'Pilih tanggal' },
]
</script>

<template>
  <div class="grid gap-3 print:hidden">
    <div class="flex flex-wrap gap-1.5" role="group" aria-label="Periode laporan">
      <button
        v-for="d in daftar"
        :key="d.nilai"
        type="button"
        class="min-h-10 rounded-kecil border px-3 text-sm font-medium"
        :class="pilihan === d.nilai ? 'border-denim bg-denim text-white' : 'border-garis-kuat text-tinta hover:bg-kertas'"
        :aria-pressed="pilihan === d.nilai"
        @click="pilih(d.nilai)"
      >
        {{ d.label }}
      </button>
    </div>
    <div v-if="pilihan === 'bebas'" class="flex flex-wrap items-end gap-3 [&>label]:flex-[1_1_160px]">
      <label class="grid gap-1.5 text-[0.9rem] font-semibold">
        Dari
        <input :value="periode.dari" type="date" :max="periode.sampai" @change="periode = { ...periode, dari: ($event.target as HTMLInputElement).value }" />
      </label>
      <label class="grid gap-1.5 text-[0.9rem] font-semibold">
        Sampai
        <input :value="periode.sampai" type="date" :min="periode.dari" @change="periode = { ...periode, sampai: ($event.target as HTMLInputElement).value }" />
      </label>
    </div>
  </div>
</template>

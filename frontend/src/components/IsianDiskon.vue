<script setup lang="ts">
import type { Diskon } from '@/stores/keranjang'

// Masukan diskon dalam rupiah atau persen (KF-07).
defineProps<{ label: string }>()
const diskon = defineModel<Diskon>({ required: true })

function ubahNilai(e: Event) {
  const angka = Number((e.target as HTMLInputElement).value.replace(/[^\d]/g, ''))
  diskon.value = { ...diskon.value, nilai: Number.isFinite(angka) ? angka : 0 }
}
</script>

<template>
  <div class="flex items-center gap-2" role="group" :aria-label="label">
    <div class="flex overflow-hidden rounded-kecil border border-garis-kuat text-sm">
      <button
        v-for="m in (['rp', 'persen'] as const)"
        :key="m"
        type="button"
        class="min-h-10 px-3 font-medium"
        :class="diskon.mode === m ? 'bg-denim text-white' : 'text-redup hover:bg-kertas'"
        :aria-pressed="diskon.mode === m"
        @click="diskon = { ...diskon, mode: m }"
      >
        {{ m === 'rp' ? 'Rp' : '%' }}
      </button>
    </div>
    <input
      :value="diskon.nilai || ''"
      inputmode="numeric"
      class="min-h-10 w-28 tabular-nums"
      placeholder="0"
      :aria-label="`${label} (${diskon.mode === 'rp' ? 'rupiah' : 'persen'})`"
      @input="ubahNilai"
    />
  </div>
</template>

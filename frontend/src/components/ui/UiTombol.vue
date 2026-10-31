<script setup lang="ts">
import { RouterLink, type RouteLocationRaw } from 'vue-router'

// Tombol aksi. Dengan `to`, dirender sebagai tautan rute yang tampil seperti tombol.
// Atribut lain (type, disabled, aria-label, @click) diteruskan ke elemen akar.
const props = withDefaults(
  defineProps<{
    varian?: 'utama' | 'kedua' | 'bahaya' | 'polos'
    ukuran?: 'kecil' | 'normal' | 'besar'
    to?: RouteLocationRaw
  }>(),
  { varian: 'utama', ukuran: 'normal', to: undefined },
)

const kelasVarian = {
  utama: 'border-transparent bg-denim text-white hover:not-disabled:bg-indigo-2',
  kedua: 'border-garis-kuat bg-white text-tinta hover:not-disabled:bg-denim-terang',
  bahaya: 'border-current bg-white text-bahaya hover:not-disabled:bg-bahaya-terang',
  polos: 'border-transparent bg-transparent text-denim hover:not-disabled:bg-denim-terang',
}

const kelasUkuran = {
  kecil: 'min-h-9 px-2',
  normal: props.varian === 'polos' ? 'min-h-11 px-2' : 'min-h-11 px-[18px]',
  besar: 'min-h-14 px-[18px] text-[1.1rem]',
}
</script>

<template>
  <component
    :is="props.to ? RouterLink : 'button'"
    :to="props.to"
    class="inline-flex items-center justify-center gap-2 rounded-kecil border py-2 font-semibold whitespace-nowrap no-underline"
    :class="[kelasVarian[props.varian], kelasUkuran[props.ukuran]]"
  >
    <slot />
  </component>
</template>

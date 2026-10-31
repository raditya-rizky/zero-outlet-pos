<script setup lang="ts">
// Pembungkus isi halaman. Pakai `as="form"` bila seluruh halaman adalah formulir.
const props = withDefaults(defineProps<{ ukuran?: 'sempit' | 'sedang' | 'lebar' | 'penuh'; as?: string }>(), {
  ukuran: 'penuh',
  as: 'div',
})

const lebar = {
  sempit: 'max-w-[520px]',
  sedang: 'max-w-[560px]',
  lebar: 'max-w-[760px]',
  penuh: 'max-w-[1200px]',
}
</script>

<template>
  <component :is="props.as" class="mx-auto grid gap-8 px-4 pt-10 pb-16" :class="lebar[props.ukuran]">
    <slot />
  </component>
</template>

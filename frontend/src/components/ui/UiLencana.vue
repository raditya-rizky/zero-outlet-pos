<script setup lang="ts">
const props = withDefaults(defineProps<{ warna?: 'denim' | 'aman' | 'bahaya' | 'waspada' | 'netral' }>(), {
  warna: 'denim',
})

const kelas = {
  denim: 'bg-denim-terang text-denim',
  aman: 'bg-aman-terang text-aman',
  bahaya: 'bg-bahaya-terang text-bahaya',
  waspada: 'bg-waspada-terang text-waspada',
  netral: 'bg-kertas text-redup',
}
</script>

<template>
  <span class="inline-block rounded-full px-2 py-0.5 text-[0.8rem] font-semibold whitespace-nowrap" :class="kelas[props.warna]">
    <slot />
  </span>
</template>

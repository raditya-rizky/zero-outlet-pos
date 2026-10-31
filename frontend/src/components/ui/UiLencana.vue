<script setup lang="ts">
// Status sebagai titik berwarna dan teks; warna tidak pernah menjadi satu-satunya penanda.
const props = withDefaults(defineProps<{ warna?: 'denim' | 'aman' | 'bahaya' | 'waspada' | 'netral' }>(), {
  warna: 'denim',
})

const kelas = {
  denim: 'text-denim',
  aman: 'text-aman',
  bahaya: 'text-bahaya',
  waspada: 'text-waspada',
  netral: 'text-redup',
}
</script>

<template>
  <span
    class="inline-flex items-center gap-1.5 text-[0.85rem] font-medium whitespace-nowrap before:size-1.5 before:rounded-full before:bg-current"
    :class="kelas[props.warna]"
  >
    <slot />
  </span>
</template>

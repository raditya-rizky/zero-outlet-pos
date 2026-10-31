<script setup lang="ts">
const props = withDefaults(defineProps<{ jenis?: 'galat' | 'berhasil' | 'info' }>(), { jenis: 'info' })

const kelas = {
  galat: 'border-bahaya bg-bahaya-terang text-bahaya-gelap',
  berhasil: 'border-aman bg-aman-terang text-aman-gelap',
  info: 'border-waspada bg-waspada-terang text-waspada-gelap',
}
</script>

<template>
  <p
    :role="props.jenis === 'galat' ? 'alert' : undefined"
    class="rounded-kecil border-l-4 px-3.5 py-2.5 text-[0.95rem]"
    :class="kelas[props.jenis]"
  >
    <slot />
  </p>
</template>

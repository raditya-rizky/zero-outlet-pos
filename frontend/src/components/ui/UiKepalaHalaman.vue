<script setup lang="ts">
defineProps<{ judul: string }>()
</script>

<template>
  <div class="flex flex-wrap items-end justify-between gap-3">
    <div>
      <slot name="atas" />
      <h1>{{ judul }}</h1>
      <p v-if="$slots.deskripsi" class="text-redup"><slot name="deskripsi" /></p>
    </div>
    <div v-if="$slots.aksi" class="flex flex-wrap gap-2.5"><slot name="aksi" /></div>
  </div>
</template>

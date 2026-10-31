<script setup lang="ts">
// Label formulir yang membungkus kontrolnya, dengan petunjuk opsional.
defineProps<{ label: string; petunjuk?: string }>()
</script>

<template>
  <label class="grid gap-1.5 text-[0.9rem] font-semibold">
    {{ label }}
    <small v-if="petunjuk" class="font-normal text-redup">{{ petunjuk }}</small>
    <slot />
  </label>
</template>

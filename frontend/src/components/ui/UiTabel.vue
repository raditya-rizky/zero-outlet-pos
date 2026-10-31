<script setup lang="ts">
// Tabel data tanpa bingkai: baris dipisahkan garis tipis, gulir horizontal di layar sempit.
// `kosong` diisi pesan yang tampil di bawah tabel saat tidak ada baris.
defineProps<{ kosong?: string | null }>()
</script>

<template>
  <div class="overflow-x-auto">
    <table class="w-full border-collapse text-[0.95rem] [&_th:first-child]:pl-0 [&_td:first-child]:pl-0">
      <slot />
    </table>
    <p v-if="kosong" class="py-10 text-redup">{{ kosong }}</p>
  </div>
</template>

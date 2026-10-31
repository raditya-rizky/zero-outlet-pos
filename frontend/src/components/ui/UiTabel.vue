<script setup lang="ts">
// Tabel data dengan bingkai dan gulir horizontal di layar sempit.
// `kosong` diisi pesan yang tampil di bawah tabel saat tidak ada baris.
defineProps<{ kosong?: string | null }>()
</script>

<template>
  <div class="overflow-x-auto rounded-sedang border border-garis bg-white">
    <table class="w-full border-collapse text-[0.95rem]">
      <slot />
    </table>
    <p v-if="kosong" class="px-4 py-10 text-center text-redup">{{ kosong }}</p>
  </div>
</template>

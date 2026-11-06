<script setup lang="ts">
// Sub-menu pengaturan: data toko, akun pengguna, dan log aktivitas.
const kelas = '-mb-px border-b-2 border-transparent pb-2 font-medium whitespace-nowrap text-redup no-underline hover:text-tinta'
const aktif = 'border-denim! text-tinta!'
</script>

<template>
  <nav class="flex gap-6 overflow-x-auto border-b border-garis" aria-label="Menu pengaturan">
    <RouterLink to="/admin/pengaturan" :class="kelas" :active-class="aktif">Toko</RouterLink>
    <RouterLink to="/admin/pengguna" :class="kelas" :active-class="aktif">Pengguna</RouterLink>
    <RouterLink to="/admin/log" :class="kelas" :active-class="aktif">Log aktivitas</RouterLink>
  </nav>
</template>

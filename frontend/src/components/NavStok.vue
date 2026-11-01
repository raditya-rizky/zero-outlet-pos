<script setup lang="ts">
// Sub-menu halaman stok (KF-11 sampai KF-14).
const kelas = '-mb-px border-b-2 border-transparent pb-2 font-medium text-redup no-underline hover:text-tinta'
const aktif = 'border-denim! text-tinta!'
</script>

<template>
  <nav class="flex gap-6 overflow-x-auto border-b border-garis" aria-label="Menu stok">
    <RouterLink to="/admin/stok" :class="kelas" :exact-active-class="aktif">Peringatan</RouterLink>
    <RouterLink to="/admin/stok/masuk" :class="kelas" :active-class="aktif">Barang masuk</RouterLink>
    <RouterLink to="/admin/stok/opname" :class="kelas" :active-class="aktif">Stok opname</RouterLink>
  </nav>
</template>

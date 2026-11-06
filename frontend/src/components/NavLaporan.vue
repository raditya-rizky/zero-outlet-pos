<script setup lang="ts">
import { useAuth } from '@/stores/auth'

// Sub-menu laporan (KF-15 sampai KF-18). Kasir hanya melihat laporan penjualannya.
const auth = useAuth()
const kelas = '-mb-px border-b-2 border-transparent pb-2 font-medium whitespace-nowrap text-redup no-underline hover:text-tinta'
const aktif = 'border-denim! text-tinta!'
</script>

<template>
  <nav v-if="auth.isAdmin" class="flex gap-6 overflow-x-auto border-b border-garis print:hidden" aria-label="Menu laporan">
    <RouterLink to="/laporan" :class="kelas" :exact-active-class="aktif">Penjualan</RouterLink>
    <RouterLink to="/admin/laporan/laba-rugi" :class="kelas" :active-class="aktif">Laba rugi</RouterLink>
    <RouterLink to="/admin/laporan/biaya" :class="kelas" :active-class="aktif">Biaya</RouterLink>
    <RouterLink to="/admin/laporan/stok" :class="kelas" :active-class="aktif">Stok</RouterLink>
    <RouterLink to="/admin/sesi" :class="kelas" :active-class="aktif">Sesi kasir</RouterLink>
  </nav>
</template>

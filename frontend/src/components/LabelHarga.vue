<script setup lang="ts">
import { ribuan } from '@/format'

// Angka total ditampilkan seperti label harga gantung pada pakaian.
defineProps<{ nilai: number; keterangan: string }>()
</script>

<template>
  <div class="label-harga" role="status" aria-live="polite">
    <span class="lubang" aria-hidden="true"></span>
    <span class="keterangan">{{ keterangan }}</span>
    <span class="nilai angka"><small>Rp</small>{{ ribuan(nilai) }}</span>
  </div>
</template>

<style scoped>
.label-harga {
  position: relative;
  display: grid;
  gap: 2px;
  padding: 14px 20px 16px 52px;
  background: var(--tag);
  color: var(--indigo);
  /* Ujung kiri dipotong miring seperti label gantung. */
  clip-path: polygon(22px 0, 100% 0, 100% 100%, 22px 100%, 0 50%);
  border-radius: 0 var(--r-kecil) var(--r-kecil) 0;
}
.lubang {
  position: absolute;
  left: 30px;
  top: 50%;
  width: 12px;
  height: 12px;
  margin-top: -6px;
  border-radius: 50%;
  background: var(--putih);
  box-shadow: inset 0 0 0 2px rgb(28 42 82 / 0.25);
}
.keterangan {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--tag-gelap);
}
.nilai {
  font-size: clamp(1.9rem, 4vw, 2.6rem);
  font-weight: 800;
  font-stretch: 125%;
  line-height: 1;
  letter-spacing: -0.02em;
}
.nilai small {
  font-size: 0.5em;
  font-weight: 700;
  margin-right: 4px;
  vertical-align: 0.6em;
}
</style>

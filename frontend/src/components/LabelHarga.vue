<script setup lang="ts">
import { ribuan } from '@/format'

// Angka total ditampilkan seperti label harga gantung pada pakaian.
defineProps<{ nilai: number; keterangan: string }>()
</script>

<template>
  <div
    class="relative grid gap-0.5 rounded-r-kecil bg-tag py-3.5 pr-5 pb-4 pl-[52px] text-indigo [clip-path:polygon(22px_0,100%_0,100%_100%,22px_100%,0_50%)]"
    role="status"
    aria-live="polite"
  >
    <span class="absolute top-1/2 left-[30px] -mt-1.5 size-3 rounded-full bg-white shadow-[inset_0_0_0_2px_rgb(28_42_82/0.25)]" aria-hidden="true"></span>
    <span class="text-[0.85rem] font-semibold text-tag-gelap">{{ keterangan }}</span>
    <span class="text-[clamp(1.9rem,4vw,2.6rem)] leading-none font-extrabold font-stretch-125% tracking-[-0.02em] tabular-nums">
      <small class="mr-1 align-[0.6em] text-[0.5em] font-bold">Rp</small>{{ ribuan(nilai) }}
    </span>
  </div>
</template>

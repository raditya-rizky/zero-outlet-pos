<script setup lang="ts">
import { ribuan, namaMetode, waktu } from '@/format'
import type { Struk } from '@/types'

// Tata letak struk untuk printer thermal 58 mm (area cetak +/- 48 mm).
defineProps<{ struk: Struk }>()
</script>

<template>
  <article class="w-[58mm] bg-white px-[5mm] py-[4mm] font-struk text-[11px] leading-[1.35] text-black tabular-nums">
    <header class="grid gap-0.5 text-center">
      <strong class="text-[14px]">{{ struk.toko.nama }}</strong>
      <span>{{ struk.toko.alamat }}</span>
    </header>
    <p v-if="struk.cetak_ulang" class="mt-1.5 text-center font-bold">*** CETAK ULANG ***</p>
    <p v-if="struk.status === 'batal'" class="mt-1.5 text-center font-bold">*** DIBATALKAN ***</p>

    <dl class="mt-2">
      <div class="flex gap-1.5"><dt class="w-[34px] flex-none">No</dt><dd>{{ struk.no_faktur }}</dd></div>
      <div class="flex gap-1.5"><dt class="w-[34px] flex-none">Tgl</dt><dd>{{ waktu(struk.tanggal_waktu) }}</dd></div>
      <div class="flex gap-1.5"><dt class="w-[34px] flex-none">Kasir</dt><dd>{{ struk.nama_kasir }}</dd></div>
    </dl>

    <hr class="my-1.5 border-dashed border-black" />
    <div class="grid gap-1">
      <div v-for="item in struk.items" :key="item.id_detail">
        <div>{{ item.nama_barang }}</div>
        <div class="flex justify-between gap-2">
          <span>{{ item.jumlah }} x {{ ribuan(item.harga_satuan) }}</span>
          <span>{{ ribuan(item.jumlah * item.harga_satuan) }}</span>
        </div>
        <div v-if="item.diskon_item" class="flex justify-between gap-2">
          <span class="pl-2">Diskon</span>
          <span>-{{ ribuan(item.diskon_item) }}</span>
        </div>
        <div v-if="item.jumlah_diretur" class="flex justify-between gap-2">
          <span class="pl-2">Diretur {{ item.jumlah_diretur }}</span>
          <span></span>
        </div>
      </div>
    </div>
    <hr class="my-1.5 border-dashed border-black" />

    <div class="flex justify-between gap-2"><span>Subtotal</span><span>{{ ribuan(struk.subtotal) }}</span></div>
    <div v-if="struk.diskon" class="flex justify-between gap-2"><span>Diskon transaksi</span><span>-{{ ribuan(struk.diskon) }}</span></div>
    <div class="my-0.5 flex justify-between gap-2 text-[13px] font-bold"><span>TOTAL</span><span>{{ ribuan(struk.total) }}</span></div>
    <div class="flex justify-between gap-2"><span>{{ namaMetode[struk.metode_bayar] }}</span><span>{{ ribuan(struk.bayar) }}</span></div>
    <div v-if="struk.metode_bayar === 'tunai'" class="flex justify-between gap-2">
      <span>Kembali</span><span>{{ ribuan(struk.kembalian) }}</span>
    </div>

    <hr class="my-1.5 border-dashed border-black" />
    <footer class="text-center">{{ struk.toko.catatan }}</footer>
  </article>
</template>

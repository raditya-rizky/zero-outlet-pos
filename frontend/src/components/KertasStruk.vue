<script setup lang="ts">
import { ribuan, namaMetode, waktu } from '@/format'
import type { Struk } from '@/types'

// Tata letak struk untuk printer thermal 58 mm (area cetak +/- 48 mm).
defineProps<{ struk: Struk }>()
</script>

<template>
  <article class="struk angka">
    <header>
      <strong>{{ struk.toko.nama }}</strong>
      <span>{{ struk.toko.alamat }}</span>
    </header>
    <p v-if="struk.cetak_ulang" class="ulang">*** CETAK ULANG ***</p>
    <p v-if="struk.status === 'batal'" class="ulang">*** DIBATALKAN ***</p>

    <dl class="meta">
      <div><dt>No</dt><dd>{{ struk.no_faktur }}</dd></div>
      <div><dt>Tgl</dt><dd>{{ waktu(struk.tanggal_waktu) }}</dd></div>
      <div><dt>Kasir</dt><dd>{{ struk.nama_kasir }}</dd></div>
    </dl>

    <hr />
    <div v-for="item in struk.items" :key="item.id_detail" class="baris">
      <div>{{ item.nama_barang }}</div>
      <div class="rinci">
        <span>{{ item.jumlah }} x {{ ribuan(item.harga_satuan) }}</span>
        <span>{{ ribuan(item.subtotal) }}</span>
      </div>
    </div>
    <hr />

    <div class="rinci"><span>Subtotal</span><span>{{ ribuan(struk.subtotal) }}</span></div>
    <div v-if="struk.diskon" class="rinci"><span>Diskon</span><span>-{{ ribuan(struk.diskon) }}</span></div>
    <div class="rinci total"><span>TOTAL</span><span>{{ ribuan(struk.total) }}</span></div>
    <div class="rinci"><span>{{ namaMetode[struk.metode_bayar] }}</span><span>{{ ribuan(struk.bayar) }}</span></div>
    <div v-if="struk.metode_bayar === 'tunai'" class="rinci"><span>Kembali</span><span>{{ ribuan(struk.kembalian) }}</span></div>

    <hr />
    <footer>{{ struk.toko.catatan }}</footer>
  </article>
</template>

<style scoped>
.struk {
  width: 58mm;
  padding: 4mm 5mm;
  background: var(--putih);
  color: #000;
  font-family: 'Courier New', ui-monospace, monospace;
  font-size: 11px;
  line-height: 1.35;
}
header {
  display: grid;
  text-align: center;
  gap: 2px;
}
header strong {
  font-size: 14px;
}
.ulang {
  margin-top: 6px;
  text-align: center;
  font-weight: 700;
}
.meta {
  margin: 8px 0 0;
}
.meta div {
  display: flex;
  gap: 6px;
}
.meta dt {
  width: 34px;
  flex: none;
}
.meta dd {
  margin: 0;
}
hr {
  border: 0;
  border-top: 1px dashed #000;
  margin: 6px 0;
}
.baris + .baris {
  margin-top: 4px;
}
.rinci {
  display: flex;
  justify-content: space-between;
  gap: 8px;
}
.total {
  font-weight: 700;
  font-size: 13px;
  margin: 2px 0;
}
footer {
  text-align: center;
}
</style>

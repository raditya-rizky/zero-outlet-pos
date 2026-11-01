<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api, pesanError } from '@/api'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTombol from '@/components/ui/UiTombol.vue'
import type { Pengaturan } from '@/types'

// Pengaturan toko yang dipakai struk, diskon kasir, retur, dan peringatan stok.
const atur = ref<Pengaturan | null>(null)
const galat = ref<string | null>(null)
const pesan = ref<string | null>(null)
const menyimpan = ref(false)

onMounted(async () => {
  try {
    atur.value = await api.get<Pengaturan>('/pengaturan')
  } catch (e) {
    galat.value = pesanError(e)
  }
})

async function simpan() {
  if (!atur.value) return
  menyimpan.value = true
  galat.value = null
  pesan.value = null
  try {
    atur.value = await api.put<Pengaturan>('/pengaturan', atur.value)
    pesan.value = 'Pengaturan disimpan.'
  } catch (e) {
    galat.value = pesanError(e)
  } finally {
    menyimpan.value = false
  }
}
</script>

<template>
  <UiHalaman ukuran="sedang" as="form" @submit.prevent="simpan">
    <UiKepalaHalaman judul="Pengaturan" />
    <UiPesan v-if="pesan" jenis="berhasil">{{ pesan }}</UiPesan>
    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <template v-if="atur">
      <section class="grid gap-4">
        <h2>Struk</h2>
        <UiIsian label="Nama toko"><input v-model="atur.nama_toko" required /></UiIsian>
        <UiIsian label="Alamat"><input v-model="atur.alamat_toko" /></UiIsian>
        <UiIsian label="Catatan di bawah struk"><textarea v-model="atur.catatan_struk"></textarea></UiIsian>
      </section>

      <section class="grid gap-4 border-t border-garis pt-8">
        <h2>Aturan kasir dan stok</h2>
        <UiIsian label="Batas diskon kasir (%)" petunjuk="Total diskon per transaksi dari subtotal. Pemilik tidak dibatasi.">
          <input v-model.number="atur.batas_diskon_kasir_persen" type="number" min="0" max="100" class="tabular-nums" />
        </UiIsian>
        <UiIsian label="Batas hari retur" petunjuk="Pengajuan lewat batas tetap boleh, dengan peringatan untuk pemilik.">
          <input v-model.number="atur.batas_hari_retur" type="number" min="1" max="90" class="tabular-nums" />
        </UiIsian>
        <UiIsian label="Lambat terjual setelah (hari)" petunjuk="Varian tanpa penjualan selama ini masuk daftar lambat terjual.">
          <input v-model.number="atur.hari_lambat_terjual" type="number" min="7" max="365" class="tabular-nums" />
        </UiIsian>
      </section>

      <UiTombol ukuran="besar" :disabled="menyimpan">Simpan pengaturan</UiTombol>
    </template>
  </UiHalaman>
</template>

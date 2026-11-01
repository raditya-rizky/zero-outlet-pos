<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api, pesanError } from '@/api'
import { waktu } from '@/format'
import NavStok from '@/components/NavStok.vue'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTabel from '@/components/ui/UiTabel.vue'
import type { PeringatanStok } from '@/types'

// Peringatan stok minimum dan daftar lambat terjual (KF-14).
const data = ref<PeringatanStok | null>(null)
const galat = ref<string | null>(null)

onMounted(async () => {
  try {
    data.value = await api.get<PeringatanStok>('/stok/peringatan')
  } catch (e) {
    galat.value = pesanError(e)
  }
})
</script>

<template>
  <UiHalaman>
    <UiKepalaHalaman judul="Stok">
      <template #deskripsi>Varian yang perlu dipesan ulang dan barang yang berisiko menjadi stok mati.</template>
    </UiKepalaHalaman>
    <NavStok />
    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <template v-if="data">
      <section class="grid gap-3">
        <h2>Stok menipis ({{ data.menipis.length }})</h2>
        <p class="text-sm text-redup">Stok sudah mencapai atau di bawah stok minimum varian.</p>
        <UiTabel :kosong="data.menipis.length ? null : 'Semua varian di atas stok minimum.'">
          <thead>
            <tr><th>Barang</th><th>SKU</th><th class="text-right">Stok</th><th class="text-right">Minimum</th><th>Terakhir terjual</th><th></th></tr>
          </thead>
          <tbody>
            <tr v-for="v in data.menipis" :key="v.id_varian">
              <td><strong>{{ v.nama_produk }}</strong> <span class="text-redup">{{ v.ukuran }} / {{ v.warna }}</span></td>
              <td class="text-sm text-redup">{{ v.sku }}</td>
              <td class="text-right font-semibold tabular-nums" :class="v.stok === 0 ? 'text-bahaya' : 'text-waspada'">{{ v.stok }}</td>
              <td class="text-right tabular-nums">{{ v.stok_minimum }}</td>
              <td class="text-sm">{{ v.terakhir_terjual ? waktu(v.terakhir_terjual) : 'Belum pernah' }}</td>
              <td class="text-right"><RouterLink :to="`/admin/stok/kartu/${v.id_varian}`">Kartu stok</RouterLink></td>
            </tr>
          </tbody>
        </UiTabel>
      </section>

      <section class="grid gap-3 border-t border-garis pt-8">
        <h2>Lambat terjual ({{ data.lambat_terjual.length }})</h2>
        <p class="text-sm text-redup">
          Masih ada stok tetapi tidak terjual selama {{ data.hari_lambat_terjual }} hari terakhir. Pertimbangkan diskon atau obral.
        </p>
        <UiTabel :kosong="data.lambat_terjual.length ? null : 'Tidak ada barang yang lambat terjual.'">
          <thead>
            <tr><th>Barang</th><th>SKU</th><th class="text-right">Stok</th><th>Terakhir terjual</th><th></th></tr>
          </thead>
          <tbody>
            <tr v-for="v in data.lambat_terjual" :key="v.id_varian">
              <td><strong>{{ v.nama_produk }}</strong> <span class="text-redup">{{ v.ukuran }} / {{ v.warna }}</span></td>
              <td class="text-sm text-redup">{{ v.sku }}</td>
              <td class="text-right tabular-nums">{{ v.stok }}</td>
              <td class="text-sm">{{ v.terakhir_terjual ? waktu(v.terakhir_terjual) : 'Belum pernah' }}</td>
              <td class="text-right"><RouterLink :to="`/admin/stok/kartu/${v.id_varian}`">Kartu stok</RouterLink></td>
            </tr>
          </tbody>
        </UiTabel>
      </section>
    </template>
  </UiHalaman>
</template>

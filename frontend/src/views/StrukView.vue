<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api, pesanError } from '@/api'
import KertasStruk from '@/components/KertasStruk.vue'
import { rupiah } from '@/format'
import type { Struk } from '@/types'

const props = defineProps<{ id: string }>()
const route = useRoute()
const router = useRouter()

const struk = ref<Struk | null>(null)
const galat = ref<string | null>(null)
const tombolCetak = ref<HTMLButtonElement>()
const baru = route.query.baru === '1'

onMounted(async () => {
  const cetakUlang = route.query.cetak_ulang === '1'
  try {
    struk.value = await api.get<Struk>(`/transaksi/${props.id}/struk`, { cetak_ulang: cetakUlang || undefined })
    // Hapus penanda dari URL agar memuat ulang halaman tidak menambah hitungan cetak.
    if (cetakUlang) router.replace({ query: { ...route.query, cetak_ulang: undefined } })
    await nextTick()
    tombolCetak.value?.focus()
  } catch (e) {
    galat.value = pesanError(e)
  }
})

function cetak() {
  window.print()
}
</script>

<template>
  <div class="halaman struk-halaman">
    <p v-if="galat" class="pesan galat">{{ galat }}</p>

    <template v-if="struk">
      <div class="kepala-halaman tanpa-cetak">
        <div>
          <h1>{{ baru ? 'Transaksi tersimpan' : 'Struk transaksi' }}</h1>
          <p class="redup">
            {{ struk.no_faktur }}<template v-if="baru && struk.metode_bayar === 'tunai'">. Berikan kembalian {{ rupiah(struk.kembalian) }} kepada pelanggan.</template>
          </p>
        </div>
      </div>

      <div class="tata">
        <div class="area-cetak">
          <KertasStruk :struk="struk" />
        </div>
        <div class="aksi tanpa-cetak">
          <button ref="tombolCetak" class="tombol besar" @click="cetak">Cetak struk</button>
          <RouterLink v-if="baru" to="/kasir" class="tombol kedua besar">Transaksi baru</RouterLink>
          <RouterLink v-else to="/riwayat" class="tombol kedua">Kembali ke riwayat</RouterLink>
          <p class="kecil redup">Tanpa printer thermal, pilih “Simpan sebagai PDF” pada jendela cetak.</p>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.struk-halaman {
  max-width: 760px;
}
.tata {
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  align-items: start;
}
.area-cetak {
  padding: 16px;
  background: var(--garis);
  border-radius: var(--r);
}
.area-cetak :deep(.struk) {
  box-shadow: var(--bayang);
}
.aksi {
  display: grid;
  gap: 10px;
  flex: 1 1 220px;
}
</style>

<style>
@media print {
  @page {
    size: 58mm auto;
    margin: 0;
  }
  body {
    background: #fff;
  }
  .tanpa-cetak {
    display: none !important;
  }
  .struk-halaman {
    padding: 0 !important;
    margin: 0 !important;
  }
  .struk-halaman .area-cetak {
    padding: 0;
    background: none;
  }
  .struk-halaman .struk {
    box-shadow: none !important;
  }
}
</style>

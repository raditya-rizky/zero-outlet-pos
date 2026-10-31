<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useSesi } from '@/stores/sesi'
import { useAuth } from '@/stores/auth'
import { pesanError } from '@/api'
import { parseRupiah, ribuan, rupiah } from '@/format'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTombol from '@/components/ui/UiTombol.vue'

const sesi = useSesi()
const auth = useAuth()
const router = useRouter()

const modal = ref('')
const galat = ref<string | null>(null)
const memproses = ref(false)
const pilihanModal = [100_000, 200_000, 300_000, 500_000]

onMounted(async () => {
  // Bila ternyata masih ada sesi aktif, langsung ke layar kasir.
  if (await sesi.muat().catch(() => null)) router.replace('/kasir')
})

function rapikan() {
  const n = parseRupiah(modal.value)
  modal.value = n ? ribuan(n) : ''
}

async function buka() {
  galat.value = null
  memproses.value = true
  try {
    await sesi.buka(parseRupiah(modal.value))
    router.replace('/kasir')
  } catch (e) {
    galat.value = pesanError(e)
  } finally {
    memproses.value = false
  }
}
</script>

<template>
  <UiHalaman ukuran="sempit">
    <form class="grid gap-[18px] rounded-sedang border border-garis bg-white p-6" @submit.prevent="buka">
      <div>
        <h1>Buka sesi kasir</h1>
        <p class="mt-1.5 text-redup">Hitung uang di laci kas sebelum mulai melayani. Jumlah ini menjadi modal awal sesi.</p>
      </div>

      <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

      <UiIsian label="Modal awal di laci kas">
        <div class="flex items-center rounded-kecil border border-garis-kuat bg-white focus-within:border-denim focus-within:shadow-fokus">
          <span class="pl-3.5 font-bold text-redup">Rp</span>
          <input
            v-model="modal"
            inputmode="numeric"
            placeholder="0"
            autofocus
            class="min-h-14 border-0 text-2xl font-bold tabular-nums focus-visible:shadow-none"
            @blur="rapikan"
          />
        </div>
      </UiIsian>
      <div class="grid grid-cols-[repeat(auto-fit,minmax(110px,1fr))] gap-2">
        <UiTombol v-for="n in pilihanModal" :key="n" type="button" varian="kedua" @click="modal = ribuan(n)">
          {{ rupiah(n) }}
        </UiTombol>
      </div>

      <UiTombol ukuran="besar" :disabled="memproses">Buka sesi</UiTombol>
      <RouterLink v-if="auth.isAdmin" to="/admin/produk" class="text-sm">Kelola toko tanpa membuka sesi</RouterLink>
    </form>
  </UiHalaman>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useSesi } from '@/stores/sesi'
import { useAuth } from '@/stores/auth'
import { pesanError } from '@/api'
import { parseRupiah, ribuan, rupiah } from '@/format'

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
  <div class="halaman sempit">
    <form class="panel isi" @submit.prevent="buka">
      <div>
        <h1>Buka sesi kasir</h1>
        <p class="redup">Hitung uang di laci kas sebelum mulai melayani. Jumlah ini menjadi modal awal sesi.</p>
      </div>

      <p v-if="galat" class="pesan galat" role="alert">{{ galat }}</p>

      <label class="isian">
        Modal awal di laci kas
        <div class="rupiah">
          <span>Rp</span>
          <input v-model="modal" inputmode="numeric" placeholder="0" autofocus class="angka" @blur="rapikan" />
        </div>
      </label>
      <div class="cepat">
        <button v-for="n in pilihanModal" :key="n" type="button" class="tombol kedua" @click="modal = ribuan(n)">
          {{ rupiah(n) }}
        </button>
      </div>

      <button class="tombol besar" :disabled="memproses">Buka sesi</button>
      <RouterLink v-if="auth.isAdmin" to="/admin/produk" class="kecil">Kelola toko tanpa membuka sesi</RouterLink>
    </form>
  </div>
</template>

<style scoped>
.sempit {
  max-width: 520px;
}
.isi {
  display: grid;
  gap: 18px;
  padding: 24px;
}
.isi p.redup {
  margin-top: 6px;
}
.rupiah {
  display: flex;
  align-items: center;
  border: 1px solid var(--garis-kuat);
  border-radius: var(--r-kecil);
  background: var(--putih);
}
.rupiah:focus-within {
  border-color: var(--denim);
  box-shadow: var(--fokus);
}
.rupiah span {
  padding-left: 14px;
  font-weight: 700;
  color: var(--redup);
}
.rupiah input {
  border: 0;
  box-shadow: none;
  font-size: 1.5rem;
  font-weight: 700;
  min-height: 56px;
}
.cepat {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(110px, 1fr));
  gap: 8px;
}
</style>

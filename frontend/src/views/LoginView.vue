<script setup lang="ts">
import { ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuth } from '@/stores/auth'
import { pesanError } from '@/api'

const auth = useAuth()
const router = useRouter()
const route = useRoute()

const username = ref('')
const kataSandi = ref('')
const galat = ref<string | null>(null)
const memproses = ref(false)

async function masuk() {
  galat.value = null
  memproses.value = true
  try {
    await auth.login(username.value, kataSandi.value)
    const lanjut = typeof route.query.lanjut === 'string' ? route.query.lanjut : '/'
    router.replace(lanjut)
  } catch (e) {
    galat.value = pesanError(e)
    kataSandi.value = ''
  } finally {
    memproses.value = false
  }
}
</script>

<template>
  <div class="latar">
    <form class="kartu" @submit.prevent="masuk">
      <div class="merek">
        <svg viewBox="0 0 32 32" width="44" height="44" aria-hidden="true">
          <path d="M4 6h14l10 10-12 12L4 18z" fill="var(--tag)" />
          <circle cx="10" cy="12" r="2.5" fill="var(--indigo)" />
        </svg>
        <div>
          <h1>Zero Outlet</h1>
          <p class="redup">Aplikasi kasir toko</p>
        </div>
      </div>

      <p v-if="galat" class="pesan galat" role="alert">{{ galat }}</p>

      <label class="isian">
        Nama pengguna
        <input v-model="username" autocomplete="username" autocapitalize="none" required autofocus />
      </label>
      <label class="isian">
        Kata sandi
        <input v-model="kataSandi" type="password" autocomplete="current-password" required />
      </label>
      <button class="tombol besar" :disabled="memproses">{{ memproses ? 'Memeriksa…' : 'Masuk' }}</button>
    </form>
  </div>
</template>

<style scoped>
.latar {
  min-height: 100dvh;
  display: grid;
  place-items: center;
  padding: 24px 16px;
  background:
    repeating-linear-gradient(135deg, rgb(255 255 255 / 0.03) 0 2px, transparent 2px 9px),
    var(--indigo);
}
.kartu {
  width: min(400px, 100%);
  display: grid;
  gap: 18px;
  padding: 28px 24px;
  background: var(--putih);
  border-radius: var(--r);
  box-shadow: 0 24px 60px rgb(0 0 0 / 0.25);
}
.merek {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 4px;
}
h1 {
  font-size: 1.7rem;
  font-stretch: 125%;
  font-weight: 800;
}
</style>

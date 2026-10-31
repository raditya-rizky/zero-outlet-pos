<script setup lang="ts">
import { ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuth } from '@/stores/auth'
import { pesanError } from '@/api'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTombol from '@/components/ui/UiTombol.vue'

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
  <div
    class="grid min-h-dvh place-items-center bg-indigo bg-[repeating-linear-gradient(135deg,rgb(255_255_255/0.03)_0_2px,transparent_2px_9px)] px-4 py-6"
  >
    <form class="grid w-full max-w-[400px] gap-[18px] rounded-sedang bg-white px-6 py-7 shadow-login" @submit.prevent="masuk">
      <div class="mb-1 flex items-center gap-3">
        <svg viewBox="0 0 32 32" width="44" height="44" aria-hidden="true">
          <path d="M4 6h14l10 10-12 12L4 18z" fill="var(--color-tag)" />
          <circle cx="10" cy="12" r="2.5" fill="var(--color-indigo)" />
        </svg>
        <div>
          <h1 class="text-[1.7rem] font-extrabold font-stretch-125%">Zero Outlet</h1>
          <p class="text-redup">Aplikasi kasir toko</p>
        </div>
      </div>

      <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

      <UiIsian label="Nama pengguna">
        <input v-model="username" autocomplete="username" autocapitalize="none" required autofocus />
      </UiIsian>
      <UiIsian label="Kata sandi">
        <input v-model="kataSandi" type="password" autocomplete="current-password" required />
      </UiIsian>
      <UiTombol ukuran="besar" :disabled="memproses">{{ memproses ? 'Memeriksa…' : 'Masuk' }}</UiTombol>
    </form>
  </div>
</template>

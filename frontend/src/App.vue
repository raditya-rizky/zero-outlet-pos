<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'
import { useAuth } from '@/stores/auth'
import { useSesi } from '@/stores/sesi'
import { useKeranjang } from '@/stores/keranjang'
import { hanyaJam } from '@/format'

const route = useRoute()
const router = useRouter()
const auth = useAuth()
const sesi = useSesi()

const tampilShell = computed(() => !!auth.pengguna && !route.meta.publik)

const kelasNav =
  'rounded-kecil px-3 py-2 font-semibold whitespace-nowrap text-indigo-pudar no-underline hover:bg-indigo-2 hover:text-white'
const kelasNavAktif = 'bg-white! text-indigo!'

async function keluar() {
  await auth.logout()
  sesi.hapus()
  useKeranjang().kosongkan()
  router.push('/login')
}
</script>

<template>
  <header
    v-if="tampilShell"
    class="sticky top-0 z-20 flex min-h-[60px] items-center gap-5 bg-indigo px-4 text-white print:hidden max-md:flex-wrap max-md:gap-x-3 max-md:gap-y-2 max-md:pt-2"
  >
    <RouterLink
      to="/"
      class="flex items-center gap-2 text-[1.1rem] font-extrabold font-stretch-120% whitespace-nowrap text-inherit no-underline"
      aria-label="Beranda Zero Outlet"
    >
      <svg viewBox="0 0 32 32" width="28" height="28" aria-hidden="true">
        <path d="M4 6h14l10 10-12 12L4 18z" fill="var(--color-tag)" />
        <circle cx="10" cy="12" r="2.5" fill="var(--color-indigo)" />
      </svg>
      <span>Zero Outlet</span>
    </RouterLink>

    <nav class="flex flex-1 gap-1 overflow-x-auto [scrollbar-width:none] max-md:order-3 max-md:basis-full max-md:pb-2" aria-label="Menu utama">
      <RouterLink to="/kasir" :class="kelasNav" :active-class="kelasNavAktif">Kasir</RouterLink>
      <RouterLink to="/riwayat" :class="kelasNav" :active-class="kelasNavAktif">Riwayat</RouterLink>
      <template v-if="auth.isAdmin">
        <RouterLink to="/admin/produk" :class="kelasNav" :active-class="kelasNavAktif">Produk</RouterLink>
        <RouterLink to="/admin/sesi" :class="kelasNav" :active-class="kelasNavAktif">Sesi kasir</RouterLink>
        <RouterLink to="/admin/pengguna" :class="kelasNav" :active-class="kelasNavAktif">Pengguna</RouterLink>
        <RouterLink to="/admin/log" :class="kelasNav" :active-class="kelasNavAktif">Log aktivitas</RouterLink>
      </template>
    </nav>

    <div class="flex items-center gap-3 max-md:ml-auto">
      <RouterLink
        v-if="sesi.aktif"
        to="/sesi/tutup"
        class="rounded-full border border-tag px-2.5 py-1 text-[0.85rem] font-semibold whitespace-nowrap text-tag no-underline hover:bg-tag hover:text-indigo"
        title="Tutup sesi kasir"
      >
        Sesi sejak {{ hanyaJam(sesi.aktif.sesi.waktu_buka) }}
      </RouterLink>
      <span class="grid text-right text-[0.9rem] leading-tight font-semibold max-md:hidden">
        {{ auth.pengguna?.nama }}
        <small class="font-normal text-indigo-samar">{{ auth.isAdmin ? 'Pemilik/Admin' : 'Kasir' }}</small>
      </span>
      <button class="min-h-11 rounded-kecil px-2 font-semibold text-white hover:bg-indigo-2" @click="keluar">Keluar</button>
    </div>
  </header>

  <main>
    <RouterView />
  </main>
</template>

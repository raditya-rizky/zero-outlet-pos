<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'
import { useAuth } from '@/stores/auth'
import { useSesi } from '@/stores/sesi'
import { useKeranjang } from '@/stores/keranjang'
import { hanyaJam } from '@/format'
import { api } from '@/api'
import type { Lencana } from '@/types'

const route = useRoute()
const router = useRouter()
const auth = useAuth()
const sesi = useSesi()

const tampilShell = computed(() => !!auth.pengguna && !route.meta.publik)

// Tautan menu berupa teks; tautan aktif diberi garis bawah denim yang menempel pada garis kepala.
const kelasNav =
  '-mb-px flex items-center border-b-2 border-transparent px-1 font-medium whitespace-nowrap text-redup no-underline hover:text-tinta'
const kelasNavAktif = 'border-denim! text-tinta!'
// Menu yang mewakili beberapa halaman (sub-menu) aktif berdasarkan awalan rute.
const aktifPada = (...awalan: string[]) => (awalan.some((a) => route.path.startsWith(a)) ? kelasNavAktif : '')

// Lencana menu pemilik: pengajuan menunggu dan stok menipis, diperbarui berkala
// sebagai permintaan latar agar tidak menunda keluar otomatis (KF-01).
const lencana = ref<Lencana | null>(null)
let timerLencana: ReturnType<typeof setInterval> | undefined

async function muatLencana() {
  if (!auth.isAdmin) return
  lencana.value = await api.latar<Lencana>('/ringkasan/lencana').catch(() => lencana.value)
}

watch(
  () => auth.isAdmin,
  (admin) => {
    clearInterval(timerLencana)
    lencana.value = null
    if (admin) {
      muatLencana()
      timerLencana = setInterval(muatLencana, 30_000)
    }
  },
  { immediate: true },
)
watch(() => route.fullPath, muatLencana)
onBeforeUnmount(() => clearInterval(timerLencana))

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
    class="sticky top-0 z-20 flex min-h-14 items-stretch gap-8 border-b border-garis bg-white px-4 text-tinta print:hidden max-md:flex-wrap max-md:gap-x-4 max-md:gap-y-0 max-md:pt-2"
  >
    <RouterLink
      to="/"
      class="flex items-center gap-2 text-[1.05rem] font-bold font-stretch-120% whitespace-nowrap text-inherit no-underline"
      aria-label="Beranda Zero Outlet"
    >
      <svg viewBox="0 0 32 32" width="22" height="22" aria-hidden="true">
        <path d="M4 6h14l10 10-12 12L4 18z" fill="var(--color-tag)" />
        <circle cx="10" cy="12" r="2.5" fill="var(--color-white)" />
      </svg>
      <span>Zero Outlet</span>
    </RouterLink>

    <nav class="flex flex-1 gap-6 overflow-x-auto [scrollbar-width:none] max-md:order-3 max-md:basis-full max-md:min-h-11" aria-label="Menu utama">
      <RouterLink v-if="auth.isAdmin" to="/admin/dasbor" :class="kelasNav" :active-class="kelasNavAktif">Dasbor</RouterLink>
      <RouterLink to="/kasir" :class="kelasNav" :active-class="kelasNavAktif">Kasir</RouterLink>
      <RouterLink to="/riwayat" :class="kelasNav" :active-class="kelasNavAktif">Riwayat</RouterLink>
      <RouterLink v-if="!auth.isAdmin" to="/laporan" :class="kelasNav" :active-class="kelasNavAktif">Laporan</RouterLink>
      <template v-if="auth.isAdmin">
        <RouterLink to="/admin/persetujuan" :class="kelasNav" :active-class="kelasNavAktif">
          Persetujuan
          <span
            v-if="lencana?.pengajuan_menunggu"
            class="ml-1.5 rounded-full bg-bahaya px-1.5 text-xs leading-5 text-white tabular-nums"
            :aria-label="`${lencana.pengajuan_menunggu} menunggu`"
          >
            {{ lencana.pengajuan_menunggu }}
          </span>
        </RouterLink>
        <RouterLink to="/admin/produk" :class="kelasNav" :active-class="kelasNavAktif">Produk</RouterLink>
        <RouterLink to="/admin/stok" :class="kelasNav" :active-class="kelasNavAktif">
          Stok
          <span
            v-if="lencana?.stok_menipis"
            class="ml-1.5 rounded-full bg-waspada-terang px-1.5 text-xs leading-5 text-waspada tabular-nums"
            :aria-label="`${lencana.stok_menipis} varian menipis`"
          >
            {{ lencana.stok_menipis }}
          </span>
        </RouterLink>
        <RouterLink to="/laporan" :class="[kelasNav, aktifPada('/laporan', '/admin/laporan', '/admin/sesi')]">Laporan</RouterLink>
        <RouterLink to="/admin/pengaturan" :class="[kelasNav, aktifPada('/admin/pengaturan', '/admin/pengguna', '/admin/log')]">
          Pengaturan
        </RouterLink>
      </template>
    </nav>

    <div class="flex items-center gap-4 text-sm max-md:ml-auto">
      <RouterLink
        v-if="sesi.aktif"
        to="/sesi/tutup"
        class="whitespace-nowrap text-redup no-underline hover:text-denim hover:underline"
        title="Tutup sesi kasir"
      >
        Sesi sejak {{ hanyaJam(sesi.aktif.sesi.waktu_buka) }}
      </RouterLink>
      <span class="whitespace-nowrap text-tinta max-md:hidden">
        {{ auth.pengguna?.nama }} <span class="text-redup">({{ auth.isAdmin ? 'Pemilik/Admin' : 'Kasir' }})</span>
      </span>
      <button class="min-h-11 px-1 font-medium text-redup hover:text-tinta" @click="keluar">Keluar</button>
    </div>
  </header>

  <main>
    <RouterView />
  </main>
</template>

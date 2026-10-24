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

async function keluar() {
  await auth.logout()
  sesi.hapus()
  useKeranjang().kosongkan()
  router.push('/login')
}
</script>

<template>
  <header v-if="tampilShell" class="shell">
    <RouterLink to="/" class="merek" aria-label="Beranda Zero Outlet">
      <svg viewBox="0 0 32 32" width="28" height="28" aria-hidden="true">
        <path d="M4 6h14l10 10-12 12L4 18z" fill="var(--tag)" />
        <circle cx="10" cy="12" r="2.5" fill="var(--indigo)" />
      </svg>
      <span>Zero Outlet</span>
    </RouterLink>

    <nav class="navigasi" aria-label="Menu utama">
      <RouterLink to="/kasir">Kasir</RouterLink>
      <RouterLink to="/riwayat">Riwayat</RouterLink>
      <template v-if="auth.isAdmin">
        <RouterLink to="/admin/produk">Produk</RouterLink>
        <RouterLink to="/admin/sesi">Sesi kasir</RouterLink>
        <RouterLink to="/admin/pengguna">Pengguna</RouterLink>
        <RouterLink to="/admin/log">Log aktivitas</RouterLink>
      </template>
    </nav>

    <div class="akun">
      <RouterLink v-if="sesi.aktif" to="/sesi/tutup" class="sesi-chip" title="Tutup sesi kasir">
        Sesi sejak {{ hanyaJam(sesi.aktif.sesi.waktu_buka) }}
      </RouterLink>
      <span class="nama">
        {{ auth.pengguna?.nama }}
        <small>{{ auth.isAdmin ? 'Pemilik/Admin' : 'Kasir' }}</small>
      </span>
      <button class="tombol polos keluar" @click="keluar">Keluar</button>
    </div>
  </header>

  <main>
    <RouterView />
  </main>
</template>

<style scoped>
.shell {
  position: sticky;
  top: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  gap: 20px;
  min-height: 60px;
  padding: 0 16px;
  background: var(--indigo);
  color: var(--putih);
}
.merek {
  display: flex;
  align-items: center;
  gap: 8px;
  color: inherit;
  text-decoration: none;
  font-weight: 800;
  font-stretch: 120%;
  font-size: 1.1rem;
  white-space: nowrap;
}
.navigasi {
  display: flex;
  gap: 4px;
  overflow-x: auto;
  flex: 1;
  scrollbar-width: none;
}
.navigasi a {
  padding: 8px 12px;
  border-radius: var(--r-kecil);
  color: #c9d3ec;
  text-decoration: none;
  font-weight: 600;
  white-space: nowrap;
}
.navigasi a:hover {
  color: var(--putih);
  background: var(--indigo-2);
}
.navigasi a.router-link-active {
  color: var(--indigo);
  background: var(--putih);
}
.akun {
  display: flex;
  align-items: center;
  gap: 12px;
}
.sesi-chip {
  padding: 4px 10px;
  border-radius: 999px;
  border: 1px solid var(--tag);
  color: var(--tag);
  font-size: 0.85rem;
  font-weight: 600;
  text-decoration: none;
  white-space: nowrap;
}
.sesi-chip:hover {
  background: var(--tag);
  color: var(--indigo);
}
.nama {
  display: grid;
  line-height: 1.2;
  text-align: right;
  font-weight: 600;
  font-size: 0.9rem;
}
.nama small {
  font-weight: 400;
  color: #aab6d4;
}
.keluar {
  color: var(--putih);
}
.keluar:hover:not(:disabled) {
  background: var(--indigo-2);
}

@media (max-width: 760px) {
  .shell {
    flex-wrap: wrap;
    gap: 8px 12px;
    padding: 8px 16px 0;
  }
  .navigasi {
    order: 3;
    flex-basis: 100%;
    padding-bottom: 8px;
  }
  .nama {
    display: none;
  }
  .akun {
    margin-left: auto;
  }
}

@media print {
  .shell {
    display: none;
  }
}
</style>

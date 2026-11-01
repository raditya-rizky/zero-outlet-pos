import { createRouter, createWebHistory } from 'vue-router'
import { useAuth } from '@/stores/auth'
import { useSesi } from '@/stores/sesi'

declare module 'vue-router' {
  interface RouteMeta {
    publik?: boolean
    admin?: boolean
    butuhSesi?: boolean
    judul?: string
  }
}

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/login', component: () => import('@/views/LoginView.vue'), meta: { publik: true, judul: 'Masuk' } },
    // Diarahkan oleh guard di bawah sesuai peran setelah status login diketahui.
    { path: '/', component: { render: () => null } },
    { path: '/sesi/buka', component: () => import('@/views/SesiBukaView.vue'), meta: { judul: 'Buka sesi' } },
    { path: '/sesi/tutup', component: () => import('@/views/SesiTutupView.vue'), meta: { butuhSesi: true, judul: 'Tutup sesi' } },
    { path: '/kasir', component: () => import('@/views/KasirView.vue'), meta: { butuhSesi: true, judul: 'Kasir' } },
    { path: '/struk/:id', component: () => import('@/views/StrukView.vue'), props: true, meta: { judul: 'Struk' } },
    { path: '/riwayat', component: () => import('@/views/RiwayatView.vue'), meta: { judul: 'Riwayat transaksi' } },
    { path: '/admin/produk', component: () => import('@/views/ProdukView.vue'), meta: { admin: true, judul: 'Produk' } },
    { path: '/admin/produk/baru', component: () => import('@/views/ProdukFormView.vue'), meta: { admin: true, judul: 'Produk baru' } },
    { path: '/admin/produk/:id', component: () => import('@/views/ProdukFormView.vue'), props: true, meta: { admin: true, judul: 'Ubah produk' } },
    { path: '/admin/pengguna', component: () => import('@/views/PenggunaView.vue'), meta: { admin: true, judul: 'Pengguna' } },
    { path: '/admin/sesi', component: () => import('@/views/SesiDaftarView.vue'), meta: { admin: true, judul: 'Sesi kasir' } },
    { path: '/admin/log', component: () => import('@/views/LogView.vue'), meta: { admin: true, judul: 'Log aktivitas' } },
    { path: '/pengajuan/:id', component: () => import('@/views/PengajuanView.vue'), props: true, meta: { judul: 'Pengajuan' } },
    { path: '/admin/persetujuan', component: () => import('@/views/PersetujuanView.vue'), meta: { admin: true, judul: 'Persetujuan' } },
    { path: '/admin/stok', component: () => import('@/views/StokView.vue'), meta: { admin: true, judul: 'Stok' } },
    { path: '/admin/stok/masuk', component: () => import('@/views/BarangMasukView.vue'), meta: { admin: true, judul: 'Barang masuk' } },
    { path: '/admin/stok/masuk/baru', component: () => import('@/views/BarangMasukFormView.vue'), meta: { admin: true, judul: 'Catat barang masuk' } },
    { path: '/admin/stok/opname', component: () => import('@/views/StokOpnameView.vue'), meta: { admin: true, judul: 'Stok opname' } },
    { path: '/admin/stok/kartu/:id', component: () => import('@/views/KartuStokView.vue'), props: true, meta: { admin: true, judul: 'Kartu stok' } },
    { path: '/admin/pengaturan', component: () => import('@/views/PengaturanView.vue'), meta: { admin: true, judul: 'Pengaturan' } },
    { path: '/:pathMatch(.*)*', redirect: '/' },
  ],
})

router.beforeEach(async (to) => {
  const auth = useAuth()
  await auth.muat()

  if (to.meta.publik) return auth.pengguna ? '/' : true
  if (!auth.pengguna) return { path: '/login', query: to.fullPath !== '/' ? { lanjut: to.fullPath } : {} }
  if (to.path === '/') return auth.isAdmin ? '/admin/produk' : '/kasir'
  if (to.meta.admin && !auth.isAdmin) return '/kasir'

  if (to.meta.butuhSesi) {
    const sesi = useSesi()
    const aktif = sesi.aktif ?? (await sesi.muat().catch(() => null))
    if (!aktif) return '/sesi/buka'
  }
  return true
})

router.afterEach((to) => {
  document.title = to.meta.judul ? `${to.meta.judul} · Zero Outlet` : 'Kasir Zero Outlet'
})

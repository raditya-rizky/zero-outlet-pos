import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '@/api'
import type { Rekap } from '@/types'

/** Sesi kasir aktif milik pengguna yang sedang login (KF-19). */
export const useSesi = defineStore('sesi', () => {
  const aktif = ref<Rekap | null>(null)

  async function muat() {
    aktif.value = await api.get<Rekap | null>('/sesi/aktif')
    return aktif.value
  }

  async function buka(modal_awal: number) {
    aktif.value = await api.post<Rekap>('/sesi/buka', { modal_awal })
  }

  async function tutup(kas_fisik: number, alasan_selisih: string | null) {
    if (!aktif.value) return null
    const hasil = await api.post<Rekap>(`/sesi/${aktif.value.sesi.id_sesi}/tutup`, { kas_fisik, alasan_selisih })
    aktif.value = null
    return hasil
  }

  function hapus() {
    aktif.value = null
  }

  return { aktif, muat, buka, tutup, hapus }
})

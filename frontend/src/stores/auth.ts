import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { api } from '@/api'
import type { PenggunaLogin } from '@/types'

export const useAuth = defineStore('auth', () => {
  const pengguna = ref<PenggunaLogin | null>(null)
  const sudahDicek = ref(false)
  const isAdmin = computed(() => pengguna.value?.peran === 'admin')

  /** Memeriksa cookie sesi yang masih berlaku (mis. setelah halaman dimuat ulang). */
  async function muat() {
    if (sudahDicek.value) return
    try {
      pengguna.value = await api.get<PenggunaLogin>('/auth/me')
    } catch {
      pengguna.value = null
    } finally {
      sudahDicek.value = true
    }
  }

  async function login(username: string, kata_sandi: string) {
    pengguna.value = await api.post<PenggunaLogin>('/auth/login', { username, kata_sandi })
    sudahDicek.value = true
  }

  async function logout() {
    await api.post('/auth/logout').catch(() => undefined)
    hapus()
  }

  function hapus() {
    pengguna.value = null
  }

  return { pengguna, sudahDicek, isAdmin, muat, login, logout, hapus }
})

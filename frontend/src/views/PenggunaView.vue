<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api, pesanError } from '@/api'
import { useAuth } from '@/stores/auth'
import { waktu } from '@/format'
import DialogModal from '@/components/DialogModal.vue'
import type { Pengguna, Peran } from '@/types'

const auth = useAuth()
const data = ref<Pengguna[]>([])
const galat = ref<string | null>(null)
const pesan = ref<string | null>(null)

const formulir = ref<{ id: number | null; nama: string; username: string; peran: Peran; kata_sandi: string } | null>(null)
const galatForm = ref<string | null>(null)
const memproses = ref(false)

async function muat() {
  try {
    data.value = await api.get<Pengguna[]>('/pengguna')
  } catch (e) {
    galat.value = pesanError(e)
  }
}

function tambah() {
  galatForm.value = null
  formulir.value = { id: null, nama: '', username: '', peran: 'kasir', kata_sandi: '' }
}

function ubah(p: Pengguna) {
  galatForm.value = null
  formulir.value = { id: p.id_pengguna, nama: p.nama, username: p.username, peran: p.peran, kata_sandi: '' }
}

async function simpan() {
  const f = formulir.value
  if (!f) return
  memproses.value = true
  galatForm.value = null
  try {
    if (f.id) {
      await api.put(`/pengguna/${f.id}`, { nama: f.nama, peran: f.peran, kata_sandi: f.kata_sandi || null })
    } else {
      await api.post('/pengguna', { nama: f.nama, username: f.username, peran: f.peran, kata_sandi: f.kata_sandi })
    }
    pesan.value = `Akun ${f.nama} disimpan.`
    formulir.value = null
    muat()
  } catch (e) {
    galatForm.value = pesanError(e)
  } finally {
    memproses.value = false
  }
}

async function ubahStatus(p: Pengguna) {
  galat.value = null
  try {
    await api.put(`/pengguna/${p.id_pengguna}/status`, { aktif: !p.aktif })
    pesan.value = `Akun ${p.nama} ${p.aktif ? 'dinonaktifkan' : 'diaktifkan kembali'}.`
    muat()
  } catch (e) {
    galat.value = pesanError(e)
  }
}

onMounted(muat)
</script>

<template>
  <div class="halaman">
    <div class="kepala-halaman">
      <div>
        <h1>Pengguna</h1>
        <p class="redup">Akun kasir dan pemilik. Akun yang tidak dipakai lagi dinonaktifkan, bukan dihapus, agar riwayatnya tetap tercatat.</p>
      </div>
      <button class="tombol" @click="tambah">Tambah pengguna</button>
    </div>

    <p v-if="pesan" class="pesan berhasil">{{ pesan }}</p>
    <p v-if="galat" class="pesan galat">{{ galat }}</p>

    <div class="tabel-bungkus">
      <table>
        <thead>
          <tr>
            <th>Nama</th>
            <th>Nama pengguna</th>
            <th>Peran</th>
            <th>Status</th>
            <th>Dibuat</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="p in data" :key="p.id_pengguna">
            <td><strong>{{ p.nama }}</strong></td>
            <td>{{ p.username }}</td>
            <td>{{ p.peran === 'admin' ? 'Pemilik/Admin' : 'Kasir' }}</td>
            <td>
              <span class="lencana" :class="p.aktif ? 'aman' : 'netral'">{{ p.aktif ? 'Aktif' : 'Nonaktif' }}</span>
            </td>
            <td>{{ waktu(p.dibuat_pada) }}</td>
            <td class="kanan aksi">
              <button class="tombol polos" @click="ubah(p)">Ubah</button>
              <button v-if="p.id_pengguna !== auth.pengguna?.id_pengguna" class="tombol polos" @click="ubahStatus(p)">
                {{ p.aktif ? 'Nonaktifkan' : 'Aktifkan' }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <DialogModal v-if="formulir" :judul="formulir.id ? 'Ubah pengguna' : 'Tambah pengguna'" @tutup="formulir = null">
      <form class="form" @submit.prevent="simpan">
        <p v-if="galatForm" class="pesan galat" role="alert">{{ galatForm }}</p>
        <label class="isian">Nama <input v-model="formulir.nama" required autofocus /></label>
        <label class="isian">
          Nama pengguna
          <input v-model="formulir.username" required :disabled="!!formulir.id" autocapitalize="none" autocomplete="off" />
        </label>
        <label class="isian">
          Peran
          <select v-model="formulir.peran" :disabled="formulir.id === auth.pengguna?.id_pengguna">
            <option value="kasir">Kasir</option>
            <option value="admin">Pemilik/Admin</option>
          </select>
        </label>
        <label class="isian">
          {{ formulir.id ? 'Kata sandi baru' : 'Kata sandi' }}
          <small>{{ formulir.id ? 'Kosongkan bila tidak direset. ' : '' }}Minimal 8 karakter.</small>
          <input v-model="formulir.kata_sandi" type="password" :required="!formulir.id" minlength="8" autocomplete="new-password" />
        </label>
        <button class="tombol" :disabled="memproses">Simpan</button>
      </form>
    </DialogModal>
  </div>
</template>

<style scoped>
.aksi {
  white-space: nowrap;
  padding-block: 4px;
}
.aksi .tombol {
  min-height: 36px;
}
.form {
  display: grid;
  gap: 14px;
}
</style>

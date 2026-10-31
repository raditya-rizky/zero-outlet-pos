<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api, pesanError } from '@/api'
import { useAuth } from '@/stores/auth'
import { waktu } from '@/format'
import DialogModal from '@/components/DialogModal.vue'
import type { Pengguna, Peran } from '@/types'
import UiHalaman from '@/components/ui/UiHalaman.vue'
import UiIsian from '@/components/ui/UiIsian.vue'
import UiKepalaHalaman from '@/components/ui/UiKepalaHalaman.vue'
import UiLencana from '@/components/ui/UiLencana.vue'
import UiPesan from '@/components/ui/UiPesan.vue'
import UiTabel from '@/components/ui/UiTabel.vue'
import UiTombol from '@/components/ui/UiTombol.vue'

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
  <UiHalaman>
    <UiKepalaHalaman judul="Pengguna">
      <template #deskripsi>
        Akun kasir dan pemilik. Akun yang tidak dipakai lagi dinonaktifkan, bukan dihapus, agar riwayatnya tetap tercatat.
      </template>
      <template #aksi><UiTombol @click="tambah">Tambah pengguna</UiTombol></template>
    </UiKepalaHalaman>

    <UiPesan v-if="pesan" jenis="berhasil">{{ pesan }}</UiPesan>
    <UiPesan v-if="galat" jenis="galat">{{ galat }}</UiPesan>

    <UiTabel>
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
            <UiLencana :warna="p.aktif ? 'aman' : 'netral'">{{ p.aktif ? 'Aktif' : 'Nonaktif' }}</UiLencana>
          </td>
          <td>{{ waktu(p.dibuat_pada) }}</td>
          <td class="py-1 text-right whitespace-nowrap">
            <UiTombol varian="polos" ukuran="kecil" @click="ubah(p)">Ubah</UiTombol>
            <UiTombol v-if="p.id_pengguna !== auth.pengguna?.id_pengguna" varian="polos" ukuran="kecil" @click="ubahStatus(p)">
              {{ p.aktif ? 'Nonaktifkan' : 'Aktifkan' }}
            </UiTombol>
          </td>
        </tr>
      </tbody>
    </UiTabel>

    <DialogModal v-if="formulir" :judul="formulir.id ? 'Ubah pengguna' : 'Tambah pengguna'" @tutup="formulir = null">
      <form class="grid gap-3.5" @submit.prevent="simpan">
        <UiPesan v-if="galatForm" jenis="galat">{{ galatForm }}</UiPesan>
        <UiIsian label="Nama"><input v-model="formulir.nama" required autofocus /></UiIsian>
        <UiIsian label="Nama pengguna">
          <input v-model="formulir.username" required :disabled="!!formulir.id" autocapitalize="none" autocomplete="off" />
        </UiIsian>
        <UiIsian label="Peran">
          <select v-model="formulir.peran" :disabled="formulir.id === auth.pengguna?.id_pengguna">
            <option value="kasir">Kasir</option>
            <option value="admin">Pemilik/Admin</option>
          </select>
        </UiIsian>
        <UiIsian
          :label="formulir.id ? 'Kata sandi baru' : 'Kata sandi'"
          :petunjuk="`${formulir.id ? 'Kosongkan bila tidak direset. ' : ''}Minimal 8 karakter.`"
        >
          <input v-model="formulir.kata_sandi" type="password" :required="!formulir.id" minlength="8" autocomplete="new-password" />
        </UiIsian>
        <UiTombol :disabled="memproses">Simpan</UiTombol>
      </form>
    </DialogModal>
  </UiHalaman>
</template>

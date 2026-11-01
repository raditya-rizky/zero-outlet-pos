const angka = new Intl.NumberFormat('id-ID')
const tanggalJam = new Intl.DateTimeFormat('id-ID', { dateStyle: 'medium', timeStyle: 'short' })
const jam = new Intl.DateTimeFormat('id-ID', { timeStyle: 'short' })

export const rupiah = (n: number) => `Rp${angka.format(n)}`
export const ribuan = (n: number) => angka.format(n)
export const waktu = (iso: string) => tanggalJam.format(new Date(iso))
export const hanyaJam = (iso: string) => jam.format(new Date(iso))

/** Mengubah isian seperti "150.000" atau "150000" menjadi angka. */
export function parseRupiah(s: string): number {
  const digit = s.replace(/[^\d]/g, '')
  return digit ? Number(digit) : 0
}

export const namaMetode: Record<string, string> = {
  tunai: 'Tunai',
  transfer: 'Transfer',
  qris: 'QRIS',
}

/** Tanggal hari ini dalam format YYYY-MM-DD menurut jam perangkat. */
export function hariIni(): string {
  const d = new Date()
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

/** Waktu relatif singkat, misalnya "5 menit lalu". */
export function sejak(iso: string): string {
  const detik = Math.max(0, Math.round((Date.now() - new Date(iso).getTime()) / 1000))
  if (detik < 60) return 'baru saja'
  const menit = Math.floor(detik / 60)
  if (menit < 60) return `${menit} menit lalu`
  const jamLalu = Math.floor(menit / 60)
  if (jamLalu < 24) return `${jamLalu} jam lalu`
  return `${Math.floor(jamLalu / 24)} hari lalu`
}

export const namaJenisPengajuan: Record<string, string> = {
  batal: 'Pembatalan',
  retur: 'Retur',
  tukar: 'Tukar ukuran',
}

export const namaStatusPengajuan: Record<string, string> = {
  menunggu: 'Menunggu persetujuan',
  disetujui: 'Disetujui',
  ditolak: 'Ditolak',
  dibatalkan: 'Ditarik',
}

/** Nilai retur sebagian; sama dengan hitung::nilai_retur di backend. */
export function nilaiRetur(dibayar: number, jumlah: number, sudah: number, q: number): number {
  if (jumlah <= 0) return 0
  const bagian = (k: number) => Math.floor((dibayar * k) / jumlah)
  return bagian(sudah + q) - bagian(sudah)
}

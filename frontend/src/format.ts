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

const pendek = new Intl.DateTimeFormat('id-ID', { day: 'numeric', month: 'short' })
const bulanTahun = new Intl.DateTimeFormat('id-ID', { month: 'short', year: '2-digit' })
const tanggalPanjang = new Intl.DateTimeFormat('id-ID', { dateStyle: 'medium' })

/** Tanggal "YYYY-MM-DD" tanpa geser zona waktu. */
export function dariIso(iso: string): Date {
  const [y, m, d] = iso.split('-').map(Number)
  return new Date(y!, (m ?? 1) - 1, d ?? 1)
}
export const tanggalPendek = (iso: string) => pendek.format(dariIso(iso))
export const tanggalBulan = (iso: string) => bulanTahun.format(dariIso(iso))
export const tanggalSaja = (iso: string) => tanggalPanjang.format(dariIso(iso))

/** Angka ringkas untuk sumbu grafik: 950 rb, 1,2 jt. */
export function ringkas(n: number): string {
  const a = Math.abs(n)
  if (a >= 1_000_000_000) return `${(n / 1_000_000_000).toLocaleString('id-ID', { maximumFractionDigits: 1 })} M`
  if (a >= 1_000_000) return `${(n / 1_000_000).toLocaleString('id-ID', { maximumFractionDigits: 1 })} jt`
  if (a >= 1_000) return `${(n / 1_000).toLocaleString('id-ID', { maximumFractionDigits: 0 })} rb`
  return ribuan(n)
}

/** Basis poin ke persen: 2537 -> "25,4%". Kosong menjadi tanda pisah. */
export function persenBps(bps: number | null | undefined): string {
  if (bps === null || bps === undefined) return '–'
  return `${(bps / 100).toLocaleString('id-ID', { minimumFractionDigits: 1, maximumFractionDigits: 1 })}%`
}

/** Perubahan relatif terhadap periode sebelumnya, mis. "+12,5%"; kosong bila pembanding nol. */
export function perubahan(sekarang: number, sebelumnya: number): string | null {
  if (!sebelumnya) return null
  const p = ((sekarang - sebelumnya) / Math.abs(sebelumnya)) * 100
  return `${p > 0 ? '+' : ''}${p.toLocaleString('id-ID', { maximumFractionDigits: 1 })}%`
}

/** Format "YYYY-MM-DD" dari tanggal perangkat. */
export function isoLokal(d: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

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

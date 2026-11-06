import { isoLokal } from '@/format'

/** Periode laporan dalam tanggal perangkat (YYYY-MM-DD). */
export interface Periode {
  dari: string
  sampai: string
}

export type PilihanPeriode = 'hari' | 'minggu' | 'bulan' | 'bulan_lalu' | 'bebas'

/** Rentang untuk pilihan cepat, dihitung dari tanggal perangkat. */
export function rentangCepat(p: Exclude<PilihanPeriode, 'bebas'>, hari = new Date()): Periode {
  const h = new Date(hari.getFullYear(), hari.getMonth(), hari.getDate())
  if (p === 'hari') return { dari: isoLokal(h), sampai: isoLokal(h) }
  if (p === 'minggu') {
    const senin = new Date(h)
    senin.setDate(h.getDate() - ((h.getDay() + 6) % 7))
    return { dari: isoLokal(senin), sampai: isoLokal(h) }
  }
  if (p === 'bulan') return { dari: isoLokal(new Date(h.getFullYear(), h.getMonth(), 1)), sampai: isoLokal(h) }
  return {
    dari: isoLokal(new Date(h.getFullYear(), h.getMonth() - 1, 1)),
    sampai: isoLokal(new Date(h.getFullYear(), h.getMonth(), 0)),
  }
}

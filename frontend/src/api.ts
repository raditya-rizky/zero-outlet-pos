// Pembungkus fetch: cookie sesi ikut terkirim, kesalahan dibaca dari { pesan }.

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message)
  }
}

type Query = Record<string, string | number | boolean | null | undefined>

let saatTidakTerautentikasi: (() => void) | null = null

/** Dipanggil ketika sesi login berakhir (respons 401). */
export function aturPenangan401(fn: () => void) {
  saatTidakTerautentikasi = fn
}

function buatUrl(path: string, query?: Query) {
  const params = new URLSearchParams()
  for (const [k, v] of Object.entries(query ?? {})) {
    if (v !== null && v !== undefined && v !== '') params.set(k, String(v))
  }
  const qs = params.toString()
  return `/api${path}${qs ? `?${qs}` : ''}`
}

async function kirim<T>(method: string, path: string, body?: unknown, query?: Query, latar = false): Promise<T> {
  const headers: Record<string, string> = {}
  if (body !== undefined) headers['Content-Type'] = 'application/json'
  // Polling latar tidak dihitung sebagai aktivitas, jadi tidak menunda keluar otomatis (KF-01).
  if (latar) headers['X-Latar'] = '1'
  let res: Response
  try {
    res = await fetch(buatUrl(path, query), {
      method,
      credentials: 'same-origin',
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
    })
  } catch {
    throw new ApiError(0, 'Tidak dapat terhubung ke server. Periksa jaringan atau komputer server toko.')
  }

  const data = await res.json().catch(() => null)
  if (!res.ok) {
    if (res.status === 401 && path !== '/auth/login') saatTidakTerautentikasi?.()
    throw new ApiError(res.status, data?.pesan ?? `Permintaan gagal (${res.status})`)
  }
  return data as T
}

export const api = {
  get: <T>(path: string, query?: Query) => kirim<T>('GET', path, undefined, query),
  /** GET untuk polling berkala; tidak memperpanjang sesi login. */
  latar: <T>(path: string, query?: Query) => kirim<T>('GET', path, undefined, query, true),
  post: <T>(path: string, body?: unknown) => kirim<T>('POST', path, body ?? {}),
  put: <T>(path: string, body: unknown) => kirim<T>('PUT', path, body),
  del: <T>(path: string) => kirim<T>('DELETE', path),
}

export function pesanError(e: unknown): string {
  return e instanceof Error ? e.message : 'Terjadi kesalahan'
}

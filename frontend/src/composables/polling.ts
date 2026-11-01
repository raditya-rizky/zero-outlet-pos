import { onBeforeUnmount, onMounted } from 'vue'

/**
 * Menjalankan `fn` saat komponen dipasang lalu setiap `ms` milidetik sampai
 * komponen dilepas. Panggilan berkala sebaiknya memakai `api.latar` agar tidak
 * memperpanjang sesi login (KF-01).
 */
export function usePolling(fn: () => unknown, ms: number) {
  let timer: ReturnType<typeof setInterval> | undefined
  const berhenti = () => clearInterval(timer)
  onMounted(() => {
    fn()
    timer = setInterval(fn, ms)
  })
  onBeforeUnmount(berhenti)
  return { berhenti }
}

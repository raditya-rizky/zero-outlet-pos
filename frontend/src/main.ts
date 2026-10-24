import { createApp } from 'vue'
import { createPinia } from 'pinia'
import '@fontsource-variable/archivo/wdth.css'
import './styles.css'
import App from './App.vue'
import { router } from './router'
import { aturPenangan401 } from './api'
import { useAuth } from './stores/auth'
import { useSesi } from './stores/sesi'

const app = createApp(App)
app.use(createPinia())
app.use(router)

// Sesi login berakhir (KF-01): bersihkan status dan kembali ke halaman login.
aturPenangan401(() => {
  useAuth().hapus()
  useSesi().hapus()
  if (router.currentRoute.value.path !== '/login') router.push('/login')
})

app.mount('#app')

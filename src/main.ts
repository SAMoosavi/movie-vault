import { createApp } from 'vue'
import App from './App.vue'
import { router } from './router'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import Vue3Toastify, { type ToastContainerOptions } from 'vue3-toastify'

import 'vue3-toastify/dist/index.css'
import './assets/style.css'

const pinia = createPinia().use(piniaPluginPersistedstate)
createApp(App)
  .use(router)
  .use(pinia)
  .use(Vue3Toastify, {
    pauseOnFocusLoss: false,
    dangerouslyHTMLString: true,
    position: 'bottom-right',
  } as ToastContainerOptions)
  .mount('#app')

import { createApp } from 'vue'
import App from './App.vue'
import { router } from './router'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'

import 'vue3-toastify/dist/index.css'
import './assets/style.css'

const pinia = createPinia().use(piniaPluginPersistedstate)
createApp(App).use(router).use(pinia).mount('#app')

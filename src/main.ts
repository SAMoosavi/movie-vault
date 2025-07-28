import { createApp } from 'vue'
import App from './App.vue'
import { router } from './router'
import { createPinia } from 'pinia'


import 'vue3-toastify/dist/index.css'
import './assets/style.css'


const pinia = createPinia()
createApp(App).use(router).use(pinia).mount('#app')

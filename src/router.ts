import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'

const settingChildren: RouteRecordRaw[] = [
  { path: '', redirect: { name: 'tags_setting' }, name: 'setting_default' },
  { path: 'tags', component: () => import('./pages/settings/TagSetting.vue'), name: 'tags_setting' },
  { path: 'appearance', component: () => import('./pages/settings/AppearanceSetting.vue'), name: 'appearance_setting' },
]

const routes: RouteRecordRaw[] = [
  { path: '/', component: () => import('./pages/HomePage.vue'), name: 'home' },
  { path: '/media/:id', component: () => import('./pages/MediaPage.vue'), name: 'media_page' },
  { path: '/add-media', component: () => import('./pages/AddMediaPage.vue'), name: 'add_media' },
  {
    path: '/setting',
    component: () => import('./pages/SettingPage.vue'),
    name: 'setting_page',
    children: settingChildren,
  },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})

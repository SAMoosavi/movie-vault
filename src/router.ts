import { createRouter, createWebHistory } from 'vue-router'

const routes = [
  { path: '/', component: () => import('./pages/HomePage.vue'), name: 'home' },
  { path: '/movie/:id', component: () => import('./pages/MoviePage.vue'), name: 'movie_page' },
  {
    path: '/setting',
    component: () => import('./pages/SettingPage.vue'),
    name: 'setting_page',
    children: [
      { path: '', name: 'setting_default', redirect: { name: 'tags_setting' } },
      { path: '/tags', component: () => import('./pages/settings/TagSetting.vue'), name: 'tags_setting' },
      {
        path: '/appearance',
        component: () => import('./pages/settings/AppearanceSetting.vue'),
        name: 'appearance_setting',
      },
    ],
  },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})

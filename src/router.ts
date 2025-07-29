import { createRouter, createWebHistory } from 'vue-router'
import HomePage from './pages/HomePage.vue'
import MoviePage from './pages/MoviePage.vue'

const routes = [{ path: '/', component: HomePage, name: 'home' }, { path: '/movie/:id', component: MoviePage, name: 'movie_page' }]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})

import Vue from 'vue'
import VueRouter from 'vue-router'
import Home from '@/views/Home.vue'
import BlogAdd from '@/views/BlogAdd.vue'
import AuthLogin from '@/views/AuthLogin.vue'
import AuthLogout from '@/components/AuthLogout.vue'
import AuthCallback from '@/views/AuthCallback.vue'

Vue.use(VueRouter)

const routes = [
  {
    path: '/',
    name: 'home',
    component: Home
  },
  {
    path: '/blogs/add',
    name: 'blogs-add',
    component: BlogAdd
  },
  {
    path: '/auth/login',
    name: 'auth-login',
    component: AuthLogin
  },
  {
    path: '/auth/callback',
    name: 'auth-callback',
    component: AuthCallback
  },
  {
    path: '/auth/logout',
    name: 'auth-logout',
    component: AuthLogout
  }
]

const router = new VueRouter({
  mode: 'history',
  base: process.env.BASE_URL,
  routes
})

export default router

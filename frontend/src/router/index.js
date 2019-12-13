import Vue from 'vue'
import VueRouter from 'vue-router'
import Home from '@/views/Home.vue'
import Subscriptions from '@/views/Subscriptions.vue'
import AuthLogin from '@/views/AuthLogin.vue'
import AuthLogout from '@/components/AuthLogout.vue'

Vue.use(VueRouter)

const routes = [
  {
    path: '/',
    name: 'home',
    component: Home
  },
  {
    path: '/subs',
    name: 'subscriptions',
    component: Subscriptions
  },
  {
    path: '/auth/login',
    name: 'auth-login',
    component: AuthLogin
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

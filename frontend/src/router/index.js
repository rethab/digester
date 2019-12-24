import Vue from 'vue'
import VueRouter from 'vue-router'
import store from '@/store/index.js';
import Home from '@/views/Home.vue'
import Subscriptions from '@/views/Subscriptions.vue'
import Settings from '@/views/Settings.vue'
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
    component: Subscriptions,
    meta: { requiresAuth: true }
  },
  {
    path: '/settings',
    name: 'settings',
    component: Settings,
    meta: { requiresAuth: true }
  },
  {
    path: '/auth/login',
    name: 'auth-login',
    component: AuthLogin
  },
  {
    path: '/auth/logout',
    name: 'auth-logout',
    component: AuthLogout,
    meta: { requiresAuth: true }
  }
]

const router = new VueRouter({
  mode: 'history',
  base: process.env.BASE_URL,
  routes
})

router.beforeEach((to, from, next) => {
  if (to.matched.some(record => record.meta.requiresAuth)) {
    if (!store.getters.isAuthenticated) {
      next({ name: 'auth-login', query: { requireAuth: true } })
    } else {
      next()
    }
  } else {
    next()
  }
})

export default router

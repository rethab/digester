import Vue from 'vue'
import VueRouter from 'vue-router'
import store from '@/store/index.js';
import Home from '@/views/Home.vue'
import Subscriptions from '@/views/Subscriptions.vue'
import Updates from '@/views/Updates.vue'
import Settings from '@/views/Settings.vue'
import AuthLogin from '@/views/AuthLogin.vue'


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
    path: '/updates',
    name: 'updates',
    component: Updates,
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
  }
]

const router = new VueRouter({
  mode: 'history',
  base: process.env.BASE_URL,
  routes,
  scrollBehavior(to, from, savedPosition) {
    if (savedPosition) {
      // retain position on browser back
      return savedPosition;
    } else {
      // scroll to top when navigating
      return { x: 0, y: 0 }
    }
  }
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

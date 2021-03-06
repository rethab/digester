import Vue from 'vue'
import VueRouter from 'vue-router'
import store from '@/store/index.js';
import Home from '@/views/Home.vue'
import Privacy from '@/views/Privacy.vue'
import Terms from '@/views/Terms.vue'
import Cockpit from '@/views/Cockpit.vue'
import ActivateSubscription from '@/views/ActivateSubscription.vue'
import Subscriptions from '@/views/Subscriptions.vue'
import Subscribe from '@/views/Subscribe.vue'
import EditSubscription from '@/views/EditSubscription.vue'
import Lists from '@/views/Lists.vue'
import ShowList from '@/views/ShowList.vue'
import EditList from '@/views/EditList.vue'
import Updates from '@/views/Updates.vue'
import Settings from '@/views/Settings.vue'
import AuthLogin from '@/views/AuthLogin.vue'
import NotFound from '@/views/NotFound.vue'


Vue.use(VueRouter)

const routes = [
  {
    path: '/',
    name: 'home',
    component: Home
  },
  {
    path: '/privacy',
    name: 'privacy',
    component: Privacy
  },
  {
    path: '/terms',
    name: 'terms',
    component: Terms
  },
  {
    path: '/cockpit',
    name: 'cockpit',
    component: Cockpit,
    meta: { requiresAuth: true }
  },
  {
    path: '/subs/activate/:token',
    name: 'activate-subscription',
    component: ActivateSubscription
  },
  {
    path: '/subs',
    name: 'subscriptions',
    component: Subscriptions,
    meta: { requiresAuth: true }
  },
  {
    path: '/sub/:id/edit',
    name: 'subscriptions-edit',
    component: EditSubscription,
    meta: { requiresAuth: true }
  },
  {
    path: '/subscribe/:type/:id',
    name: 'subscription-subscribe',
    component: Subscribe,
    // for anonymous subscriptions, this flag will have to be false
    // but for now we need it to be true, because otherwise the user
    // won't be redirect to the login page if we send around links to
    // subscribe to specific channels.
    meta: { requiresAuth: true }
  },
  {
    path: '/lists',
    name: 'lists',
    component: Lists,
    meta: { requiresAuth: true }
  },
  {
    path: '/list/:id',
    meta: { requiresAuth: true },
    component: ShowList,
  },
  {

    path: '/list/:id/edit',
    meta: { requiresAuth: true },
    component: EditList,
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
  },
  {
    path: '*',
    component: NotFound
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
      next({ name: 'auth-login', query: { requireAuth: true, redirect: window.location.pathname } })
    } else {
      next()
    }
  } else {
    next()
  }
})

export default router

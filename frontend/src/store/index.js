import Vue from 'vue'
import Vuex from 'vuex'
import auth from '@/store/modules/auth.js';
import subscriptions from '@/store/modules/subscriptions.js';
import settings from '@/store/modules/settings.js';
import offline from '@/store/modules/offline.js';

Vue.use(Vuex)

export default new Vuex.Store({
  strict: process.env.NODE_ENV != 'production',
  modules: {
    auth,
    subscriptions,
    settings,
    offline
  },
})

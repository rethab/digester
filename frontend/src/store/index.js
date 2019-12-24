import Vue from 'vue'
import Vuex from 'vuex'
import auth from '@/store/modules/auth.js';
import subscriptions from '@/store/modules/subscriptions.js';

Vue.use(Vuex)

export default new Vuex.Store({
  strict: process.env.NODE_ENV != 'production',
  modules: {
    auth,
    subscriptions
  },
})

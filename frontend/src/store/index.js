import Vue from 'vue'
import Vuex from 'vuex'
import auth from '@/store/modules/auth.js';
import subscriptions from '@/store/modules/subscriptions.js';
import updates from '@/store/modules/updates.js';
import settings from '@/store/modules/settings.js';
import offline from '@/store/modules/offline.js';

Vue.use(Vuex)


/* Idea of this thing is to refresh the auth state periodically:
 *
 * We store a timestamp in the auth vuex store and periodically
 * call /auth/me. On success, we update the timestamp. Knowing
 * when the last successful auth call was, we can determine whether
 * we want to immediately check the auth state when the user loads
 * the page (eg. if the user was here 10 minutes ago, we don't need
 * to refresh the auth state, but if they weren't here in a week, the
 * session might have expired.
 */
const authRefresher = store => {

  const TIMEOUT = 60000; // ms

  const pokeAuthState = () => {
    store.dispatch("refreshAuth");
  }

  let _refresher = null;

  if (store.getters.isAuthenticated) {
    if (store.getters.hoursSinceLastAuth > 24) {
      // hasn't logged in in a while, trigger refresh directly
      store.dispatch("refreshAuth");
    }
    _refresher = setInterval(pokeAuthState, TIMEOUT);
  }

  store.subscribe((mutation) => {
    if (mutation.type === "AUTHENTICATED") {
      if (!_refresher) {
        _refresher = setInterval(pokeAuthState, TIMEOUT);
      }
    }

    else if (mutation.type === "UNAUTHENTICATED") {
      if (_refresher) {
        clearInterval(_refresher);
        _refresher = null;
      }
    }
  })
}

export default new Vuex.Store({
  strict: process.env.NODE_ENV != 'production',
  modules: {
    auth,
    subscriptions,
    updates,
    settings,
    offline
  },
  plugins: [authRefresher]
})

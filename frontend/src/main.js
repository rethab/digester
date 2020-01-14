import Vue from 'vue'
import VueTyped from 'vue-typed-js'
import VueAxios from 'vue-axios';
import VueAuthenticate from 'vue-authenticate';
import App from './App.vue'
import Axios from 'axios';
import router from './router'
import store from './store'
import vuetify from './plugins/vuetify';

Vue.config.productionTip = false

Vue.use(VueTyped);
Vue.use(VueAxios, Axios);
Vue.use(VueAuthenticate, {
  withCredentials: true,
  baseUrl: process.env.VUE_APP_API_HOST,

  providers: {
    github: {
      clientId: process.env.VUE_APP_OAUTH_GITHUB_CLIENT_ID,
      redirectUri: window.location.origin,
    },
    facebook: {
      clientId: process.env.VUE_APP_OAUTH_GITHUB_CLIENT_ID,
      redirectUri: window.location.origin,
    }
  }
});

new Vue({
  router,
  store,
  vuetify,
  render: h => h(App)
}).$mount('#app')

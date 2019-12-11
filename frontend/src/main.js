import Vue from 'vue'
import VueAxios from 'vue-axios';
import VueAuthenticate from 'vue-authenticate';
import App from './App.vue'
import Axios from 'axios';
import router from './router'
import store from './store'
import vuetify from './plugins/vuetify';

Vue.config.productionTip = false

Vue.use(VueAxios, Axios);
Vue.use(VueAuthenticate, {
  baseUrl: 'http://localhost:8000',

  providers: {
    github: {
      clientId: 'ce2e6a7d28bdf8eca16c',
      redirectUri: 'http://localhost:8080/auth/callback'
    }
  }
});

new Vue({
  router,
  store,
  vuetify,
  render: h => h(App)
}).$mount('#app')

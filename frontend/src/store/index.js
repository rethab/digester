import Vue from 'vue'
import Vuex from 'vuex'
import Api from '@/services/api';

Vue.use(Vuex)

export default new Vuex.Store({
  strict: process.env.NODE_ENV != 'production',
  state: {
    blogs: [
      { url: 'https://acolyer.org' },
      { url: 'https://vuejs.org' },
      { url: 'https://google.com' }
    ],
    user: null
  },
  mutations: {
    SET_BLOGS(state, blogs) {
      state.blogs = blogs;
    },
    ADD_BLOG(state, url) {
      let blogs = state.blogs.concat({ url: url });
      state.blogs = blogs;
    },
    AUTHENTICATE(state, user) {
      state.user = user;
    },
    UNAUTHENTICATE(state) {
      state.user = null;
    }
  },
  actions: {
    async loadBlogs({ commit }) {
      let response = await Api().get("blogs");
      commit('SET_BLOGS', response.data);
    },
    addBlog({ commit }, url) {
      commit('ADD_BLOG', url);
      return url;
    },
    authenticate({ commit }, user) {
      commit('AUTHENTICATE', user);
    },
    async unauthenticate({ commit, getters }) {
      if (!getters.isAuthenticated) {
        console.warn("Cannot logout w/o being authenticated: NOOP");
        return;
      }
      try {
        await Api().post("/auth/logout");
        commit('UNAUTHENTICATE');
      } catch (e) {
        console.error("Failed to logout");
        console.error(e);
      }
    }
  },
  getters: {
    isAuthenticated: state => {
      return state.user !== null;
    },
    username: (state, getters) => {
      if (!getters.isAuthenticated) return "Anonymous";
      else return state.user.username;
    }
  }
})

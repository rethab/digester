import Vue from 'vue'
import Vuex from 'vuex'
import Api from '@/services/api';

Vue.use(Vuex)

export default new Vuex.Store({
  strict: process.env.NODE_ENV != 'production',
  state: {
    subscriptions: [
      {
        type: 'github_release',
        name: 'kubernetes/kubernets',
        frequency: 'every saturday at 10am'
      },
      {
        type: 'github_release',
        name: 'ghc/ghc',
        frequency: 'every day at 9am'
      },
      {
        type: 'github_release',
        name: 'rethab/digester',
        frequency: 'every monday at 7am'
      },
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

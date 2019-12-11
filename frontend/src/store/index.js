import Vue from 'vue'
import Vuex from 'vuex'
import Api from '@/services/api';

Vue.use(Vuex)

export default new Vuex.Store({
  state: {
    blogs: [
      { url: 'https://acolyer.org' },
      { url: 'https://vuejs.org' },
      { url: 'https://google.com' }
    ],
    user: false
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
    }
  },
  actions: {
    async loadBlogs({ commit }) {
      let response = await Api.get("blogs");
      commit('SET_BLOGS', response.data);
    },
    async addBlog({ commit }, url) {
      commit('ADD_BLOG', url);
      return url;
    },
    async authenticate({ commit }, user) {
      commit('AUTHENTICATE', user);
    }
  }
})

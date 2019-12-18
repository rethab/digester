import Vue from 'vue'
import Vuex from 'vuex'
import Api from '@/services/api';

Vue.use(Vuex)

export default new Vuex.Store({
  strict: process.env.NODE_ENV != 'production',
  state: {
    subscriptions: [],
    user: null
  },
  mutations: {
    SET_SUBSCRIPTIONS(state, subscriptions) {
      state.subscriptions = subscriptions;
    },
    ADD_SUBSCRIPTION(state, subscription) {
      state.subscriptions.unshift(subscription);
    },
    AUTHENTICATE(state, user) {
      state.user = user;
    },
    UNAUTHENTICATE(state) {
      state.user = null;
    }
  },
  actions: {
    async loadSubscriptions({ commit }) {
      let response = await Api().get("subscriptions");
      commit('SET_SUBSCRIPTIONS', response.data);
    },
    async subscribe({ commit }, subscription) {
      let response = await Api().post("subscriptions/add", {
        channelName: subscription.name,
        channelType: subscription.type,
        frequency: subscription.frequency,
        day: subscription.day,
        time: subscription.time + ":00.00",
      });
      commit('ADD_SUBSCRIPTION', response.data);
      return subscription;
    },
    authenticate({ commit }, user) {
      commit('AUTHENTICATE', user);
    },
    async unauthenticate({ commit, getters }) {
      if (!getters.isAuthenticated) {
        return;
      }
      await Api().post("/auth/logout");
      commit('UNAUTHENTICATE');
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

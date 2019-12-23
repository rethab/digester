import Vue from 'vue'
import Vuex from 'vuex'
import Api from '@/services/api';
import auth from '@/store/modules/auth.js';

Vue.use(Vuex)

export default new Vuex.Store({
  strict: process.env.NODE_ENV != 'production',
  modules: {
    auth
  },
  state: {
    subscriptions: [],
  },
  mutations: {
    SET_SUBSCRIPTIONS(state, subscriptions) {
      state.subscriptions = subscriptions;
    },
    ADD_SUBSCRIPTION(state, subscription) {
      state.subscriptions.unshift(subscription);
    },
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
  }
})

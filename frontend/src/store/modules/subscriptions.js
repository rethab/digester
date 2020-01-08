import Api from '@/services/api.js';

const state = {
    subscriptions: []
};

const getters = {
    subscriptions: (state) => state.subscriptions
}

const actions = {
    loadSubscriptions({ commit }) {
        return new Promise((resolve, reject) => {
            Api().get("subscriptions")
                .then(resp => {
                    commit('SET_SUBSCRIPTIONS', resp.data);
                    resolve(resp);
                })
                .catch(err => {
                    reject(err)
                })
        })
    },
    subscribe({ commit }, subscription) {
        return new Promise((resolve, reject) => {
            Api().post("subscriptions/add", {
                channelName: subscription.name,
                channelType: subscription.type,
                frequency: subscription.frequency,
                day: subscription.day,
                time: subscription.time,
            }).then(resp => {
                commit('ADD_SUBSCRIPTION', resp.data);
                resolve(subscription);
            }).catch(err => {
                reject(err)
            });
        })
    },
    updateSubscription({ commit }, subscription) {
        return new Promise((resolve, reject) => {
            Api().put("subscriptions/" + subscription.id, {
                frequency: subscription.frequency,
                day: subscription.day,
                time: subscription.time
            }).then(resp => {
                commit('UPDATE_SUBSCRIPTION', resp.data);
                resolve(resp);
            }).catch(err => {
                reject(err);
            })
        });
    }
}

const mutations = {
    SET_SUBSCRIPTIONS: (state, subscriptions) => {
        state.subscriptions = subscriptions;
    },
    ADD_SUBSCRIPTION: (state, subscription) => {
        state.subscriptions.unshift(subscription);
    },
    UPDATE_SUBSCRIPTION: (state, subscription) => {
        state.subscriptions.forEach(sub => {
            if (sub.id == subscription.id) {
                sub.frequency = subscription.frequency;
                sub.day = subscription.day;
                sub.time = subscription.time;
            }
        });
    },
}

export default {
    state,
    getters,
    actions,
    mutations,
}
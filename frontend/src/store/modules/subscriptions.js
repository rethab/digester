import Api from '@/services/api.js';

const state = {
    subscriptions: []
};

const getters = {
    subscriptions: (state) =>
        // directly exposing the array means it is modified directly.
        // therefore we need to deep copy it
        JSON.parse(JSON.stringify(state.subscriptions)),
    alreadySubscribed: (state) => (channelType, channelId) =>
        state.subscriptions.some(sub => sub.channelId == channelId && sub.channelType == channelType)

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
    loadSubscription(_, id) {
        return new Promise((resolve, reject) => {
            Api().get(`subscriptions/${id}`)
                .then(resp => {
                    resolve(resp.data);
                })
                .catch(err => {
                    reject(err)
                })
        })
    },
    subscribe({ commit }, subscription) {

        let payload = {
            channelId: subscription.channelId,
            channelType: subscription.channelType,
            frequency: subscription.frequency,
            day: subscription.day,
            time: subscription.time,
        };

        return new Promise((resolve, reject) => {
            Api().post("subscriptions/add", payload)
                .then(resp => {
                    commit('ADD_SUBSCRIPTION', resp.data);
                    resolve(subscription);
                }).catch(err => {
                    reject(err)
                });
        })
    },
    updateSubscription({ commit }, subscription) {
        return new Promise((resolve, reject) => {
            Api().put(`subscriptions/${subscription.id}`, {
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
    },
    deleteSubscription({ commit }, id) {
        return new Promise((resolve, reject) => {
            Api().delete(`subscriptions/${id}`)
                .then(() => {
                    commit('DELETE_SUBSCRIPTION', id);
                    resolve();
                }).catch(err => reject(err));
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
    DELETE_SUBSCRIPTION: (state, id) => {
        state.subscriptions = state.subscriptions.filter(s => s.id != id);
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
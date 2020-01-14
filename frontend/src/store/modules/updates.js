import Api from '@/services/api.js';

const state = {
    updates: []
};

const getters = {
    updates: (state) =>
        // directly exposing the array means it is modified directly.
        // therefore we need to deep copy it
        JSON.parse(JSON.stringify(state.updates))
}

const actions = {
    loadUpdates({ commit }, { offset, limit }) {
        return new Promise((resolve, reject) => {
            let params = { offset: offset, limit: limit };
            Api().get("updates", { params: params })
                .then(resp => {
                    commit('ADD_UPDATES', resp.data);
                    resolve(resp);
                })
                .catch(err => {
                    reject(err)
                })
        })
    },
}

const mutations = {
    ADD_UPDATES: (state, updates) => {
        state.updates = state.updates.concat(updates);
    },
}

export default {
    state,
    getters,
    actions,
    mutations,
}
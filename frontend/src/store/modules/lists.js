import Api from '@/services/api.js';

const state = {
    lists: []
};

const getters = {
    lists: (state) =>
        // directly exposing the array means it is modified directly.
        // therefore we need to deep copy it
        JSON.parse(JSON.stringify(state.lists))
}

const actions = {
    loadLists({ commit }) {
        return new Promise((resolve, reject) => {
            Api().get("lists")
                .then(resp => {
                    commit('SET_LISTS', resp.data);
                    resolve(resp);
                })
                .catch(err => {
                    reject(err)
                })
        })
    },
    createList({ commit }, list) {
        return new Promise((resolve, reject) => {
            Api().put("lists", {
                name: list.name
            }).then(resp => {
                list.id = resp.data.id;
                list.channels = [];
                commit('ADD_LIST', list);
                resolve(list);
            }).catch(err => {
                reject(err)
            });
        })
    },
    addChannel({ commit }, { list, channel }) {
        commit('ADD_CHANNEL', { list, channel });
        Promise.resolve(channel);
        // todo impl api :)
    },
}

const mutations = {
    SET_LISTS: (state, lists) => {
        state.lists = lists;
    },
    ADD_LIST: (state, list) => {
        state.lists.unshift(list);
    },
    ADD_CHANNEL: (state, { list, channel }) => {
        const index = state.lists.findIndex(x => x.id == list.id);
        state.lists[index].channels.unshift(channel);
    }
}

export default {
    state,
    getters,
    actions,
    mutations,
}
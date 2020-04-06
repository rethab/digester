import Api from '@/services/api.js';

const state = {
    lists: []
};

const getters = {
    lists: (state) => state.lists,
    channelsByListId: (state) => (id) => state.lists.find(l => l.id == id).channels
}

const actions = {
    loadLists({ commit }) {
        return new Promise((resolve, reject) => {
            Api().get("lists?own=true")
                .then(resp => {
                    commit('SET_LISTS', resp.data);
                    resolve(resp);
                })
                .catch(err => {
                    reject(err)
                })
        })
    },
    loadList({ commit }, id) {
        return new Promise((resolve, reject) => {
            Api().get(`lists/${id}`)
                .then(resp => {
                    commit('SET_LISTS', [resp.data]);
                    resolve(resp.data);
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
                const newList = resp.data;
                commit('ADD_LIST', newList);
                resolve(newList);
            }).catch(err => {
                reject(err)
            });
        })
    },
    deleteList({ commit }, list) {
        return new Promise((resolve, reject) => {
            Api().delete(`lists/${list.id}`).then(() => {
                commit('REMOVE_LIST', list);
                resolve();
            }).catch(err => reject(err))
        })
    },
    addChannel({ commit }, { list, channel }) {
        return new Promise((resolve, reject) => {
            Api().post(`lists/${list.id}/add_channel`, {
                id: channel.id,
            }).then(() => {
                commit('ADD_CHANNEL', { list, channel });
                resolve(channel);
            }).catch(err => {
                reject(err)
            });
        })
    },
    removeChannel({ commit }, { list, channel }) {
        return new Promise((resolve, reject) => {
            Api().post(`lists/${list.id}/remove_channel`, {
                id: channel.id,
            }).then(() => {
                commit('REMOVE_CHANNEL', { list, channel });
                resolve(channel);
            }).catch(err => {
                reject(err)
            });
        })
    },
}

const mutations = {
    SET_LISTS: (state, lists) => {
        state.lists = lists;
    },
    ADD_LIST: (state, list) => {
        state.lists.unshift(list);
    },
    REMOVE_LIST: (state, list) => {
        state.lists = state.lists.filter(l => l.id != list.id);
    },
    ADD_CHANNEL: (state, { list, channel }) => {
        const index = state.lists.findIndex(l => l.id == list.id);
        state.lists[index].channels = [channel, ...state.lists[index].channels];
    },
    REMOVE_CHANNEL: (state, { list, channel }) => {
        const listIndex = state.lists.findIndex(l => l.id == list.id);
        const channels = state.lists[listIndex].channels;
        state.lists[listIndex].channels = channels.filter(c => c.id != channel.id);
    }
}

export default {
    state,
    getters,
    actions,
    mutations,
}
const state = {
    offline: false,
};

const getters = {
    isOffline: (state) => state.offline
}

const actions = {
    setOnline({ commit }) {
        commit("SET_OFFLINE", false)
    },
    setOffline({ commit }) {
        commit("SET_OFFLINE", true)
    },
}

const mutations = {
    "SET_OFFLINE": (state, offline) => {
        state.offline = offline;
    },
}

export default {
    state,
    getters,
    actions,
    mutations,
}
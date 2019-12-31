import Api from '@/services/api.js';

const state = {
    timezone: null,
};

const getters = {
    timezone: (state) => state.timezone
}

const actions = {
    setTimezone({ commit }, timezone) {
        return new Promise((resolve, reject) => {
            Api().post("/settings/", { timezone: timezone })
                .then(resp => {
                    commit("SET_TIMEZONE", timezone);
                    resolve(resp);
                }).catch(err => {
                    reject(err);
                });
        });
    },
    loadTimezone({ commit }) {
        return new Promise((resolve, reject) => {
            Api().get("/settings/")
                .then(resp => {
                    if (resp.data.timezone) {
                        commit("SET_TIMEZONE", resp.data.timezone);
                    }
                    resolve(resp);
                }).catch(err => {
                    reject(err);
                });
        });
    },
}

const mutations = {
    "SET_TIMEZONE": (state, timezone) => {
        state.timezone = timezone;
    },
}

export default {
    state,
    getters,
    actions,
    mutations,
}
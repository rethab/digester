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
    loadTimezone({ commit, getters }) {
        return new Promise((resolve, reject) => {
            // only load timezone if we don't know it already
            if (getters.timezone === null) {
                Api().get("/settings/")
                    .then(resp => {
                        if (resp.data.timezone) {
                            commit("SET_TIMEZONE", resp.data.timezone);
                        }
                        resolve(resp);
                    }).catch(err => {
                        reject(err);
                    });
            } else {
                // fake server response
                resolve({ data: { timezone: getters.timezone } });
            }
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
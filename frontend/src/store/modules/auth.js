import Api from '@/services/api.js';

const state = {
    // best effort guess on whether user is logged in
    authTimestamp: localStorage.getItem('auth-ts') || null,
    username: localStorage.getItem('username') || null
};

const getters = {
    isAuthenticated: state => !!state.authTimestamp,
    hoursSinceLastAuth: state => {
        const seconds = (Date.now() / 1000) - (state.authTimestamp / 1000);
        return seconds / (60 * 60);
    },
    username: (state, getters) =>
        getters.isAuthenticated ? state.username : "Anonymous"
}

const actions = {
    authenticate({ commit, dispatch }, payload) {
        const vueAuth = payload.vueAuth,
            provider = payload.provider;

        return new Promise((resolve, reject) => {
            vueAuth.authenticate(provider)
                .then(resp => {
                    const username = resp.data.username;
                    localStorage.setItem('username', username);
                    commit("AUTHENTICATED", {
                        username: username
                    });
                    dispatch("updateAuthTimestamp");
                    resolve(resp);
                }).catch(err => {
                    localStorage.removeItem('auth-ts');
                    localStorage.removeItem('username');
                    reject(err);
                });
        });
    },
    unauthenticate({ getters, dispatch }) {
        return new Promise((resolve, reject) => {
            if (!getters.isAuthenticated) {
                resolve({});
            } else {
                Api().post("/auth/logout")
                    .then(() => {
                        dispatch("unauthenticated")
                        resolve({});
                    })
                    .catch(err => {
                        reject(err)
                    });
            }
        }
        )
    },
    refreshAuth({ dispatch }) {
        Api().get("/auth/me")
            .then(() => {
                dispatch("updateAuthTimestamp");
            })
    },
    updateAuthTimestamp({ commit }) {
        const ts = Date.now();
        commit("SET_AUTH_TIMESTAMP", ts);
        localStorage.setItem('auth-ts', ts);
    },
    unauthenticated({ commit }) {
        localStorage.removeItem('auth-ts');
        localStorage.removeItem('username');
        commit('UNAUTHENTICATED');
    }

}

const mutations = {
    "AUTHENTICATED": (state, user) => {
        state.username = user.username;
    },
    "UNAUTHENTICATED": (state) => {
        state.authTimestamp = null;
        state.username = null;
    },
    "SET_AUTH_TIMESTAMP": (state, ts) => {
        state.authTimestamp = ts;
    }
}

export default {
    state,
    getters,
    actions,
    mutations,
}
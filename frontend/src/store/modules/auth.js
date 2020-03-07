import Api from '@/services/api.js';

const state = {
    // best effort guess on whether user is logged in
    authTimestamp: localStorage.getItem('auth-ts') || null,
    username: localStorage.getItem('username') || null,
    userId: localStorage.getItem('userId') || null
};

const getters = {
    isAuthenticated: state => !!state.authTimestamp,
    hoursSinceLastAuth: state => {
        const seconds = (Date.now() / 1000) - (state.authTimestamp / 1000);
        return seconds / (60 * 60);
    },
    userId: (state, getters) => getters.isAuthenticated ? parseInt(state.userId) : null,
    username: (state, getters) => getters.isAuthenticated ? state.username : "Anonymous",
}

const actions = {
    authenticate({ commit, dispatch }, payload) {
        const vueAuth = payload.vueAuth,
            provider = payload.provider;

        return new Promise((resolve, reject) => {
            vueAuth.authenticate(provider)
                .then(resp => {
                    const username = resp.data.username;
                    const userId = resp.data.userId;
                    localStorage.setItem('username', username);
                    localStorage.setItem('userId', userId);
                    commit("AUTHENTICATED", {
                        username: username,
                        userId: userId,
                    });
                    dispatch("updateAuthTimestamp");
                    resolve(resp);
                }).catch(err => {
                    localStorage.removeItem('auth-ts');
                    localStorage.removeItem('username');
                    localStorage.removeItem('userId');
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
    refreshAuth({ dispatch, getters, commit }) {
        Api().get("/auth/me")
            .then(resp => {
                if (isNaN(getters.userId)) {
                    // temporary to automatically set the userId for users that were signed in before the userId was there
                    const username = resp.data.username;
                    const userId = resp.data.userId;
                    localStorage.setItem('username', username);
                    localStorage.setItem('userId', userId);
                    commit("AUTHENTICATED", {
                        username: username,
                        userId: userId,
                    });
                }
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
        localStorage.removeItem('userId');
        commit('UNAUTHENTICATED');
    }

}

const mutations = {
    "AUTHENTICATED": (state, user) => {
        state.username = user.username;
        state.userId = user.userId;
    },
    "UNAUTHENTICATED": (state) => {
        state.authTimestamp = null;
        state.username = null;
        state.userId = NaN;
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
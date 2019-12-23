import Api from '@/services/api.js';

const state = {
    // best effort guess on whether user is logged in
    guessAuth: localStorage.getItem('guess-auth') || false,
    username: localStorage.getItem('username') || null
};

const getters = {
    isAuthenticated: state => state.guessAuth,
    username: (state, getters) =>
        getters.isAuthenticated ? state.username : "Anonymous"
}

const actions = {
    authenticate({ commit }, payload) {
        const vueAuth = payload.vueAuth,
            provider = payload.provider;

        return new Promise((resolve, reject) => {
            console.log("Inside promise: " + vueAuth + " provider: " + provider);
            vueAuth.authenticate(provider)
                .then(resp => {
                    const username = resp.data.username;
                    localStorage.setItem('guess-auth', true);
                    localStorage.setItem('username', username);
                    commit("AUTHENTICATED", {
                        username: username
                    });
                    console.log("resolving promise..");
                    resolve(resp);
                }).catch(err => {
                    localStorage.removeItem('guess-auth');
                    localStorage.removeItem('username');
                    reject(err);
                });
        });
    },
    unauthenticate({ getters, dispatch }) {
        if (!getters.isAuthenticated) {
            return;
        }
        Api().post("/auth/logout").then(() => {
            dispatch("unauthenticated")
        });
    },
    unauthenticated({ commit }) {
        localStorage.removeItem('guess-auth');
        localStorage.removeItem('username');
        commit('UNAUTHENTICATED');
    }

}

const mutations = {
    "AUTHENTICATED": (state, user) => {
        state.guessAuth = true;
        state.username = user.username;
    },
    "UNAUTHENTICATED": (state) => {
        state.guessAuth = false;
        state.username = null;
    }
}

export default {
    state,
    getters,
    actions,
    mutations,
}
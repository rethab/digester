import Api from '@/services/api.js';

const state = {
    lists: [
        {
            name: 'Top of Scala',
            channels: [
                { name: 'Scala Programming Blog', channelType: 'RssFeed' },
                { name: 'lightbend/scalac', channelType: 'GithubRelease' },
                { name: 'Propsensive\' Thoughts on Reality', channelType: 'RssFeed' },
            ]
        },
        {
            name: 'Android Dev',
            channels: [
                { name: 'Jetbrains Android Studio Blog', channelType: 'RssFeed' },
                { name: 'google/android', channelType: 'GithubRelease' },
                { name: 'Andy Rubyn Daily', channelType: 'RssFeed' },
            ]
        },
    ]
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
    }
}

const mutations = {
    SET_LISTS: (state, lists) => {
        state.lists = lists;
    }
}

export default {
    state,
    getters,
    actions,
    mutations,
}
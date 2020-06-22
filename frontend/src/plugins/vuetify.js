import Vue from 'vue';
import Vuetify from 'vuetify/lib';

Vue.use(Vuetify);

export default new Vuetify({
    icons: {
        iconfont: 'mdiSvg',
    },
    theme: {
        themes: {
            light: {
                primary: '#86DBEB',
                secondary: '#619FAB',
            },
        },
        options: {
            cspNonce: 'dc77ae858e'
        }
    },
});

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
                primary: '#73A2BF',
                secondary: '#5D7CA6',
            },
        },
        options: {
            cspNonce: 'dc77ae858e'
        }
    },
});

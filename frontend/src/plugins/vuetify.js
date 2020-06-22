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
                primary: '#7EB1D0',
                secondary: '#5D7CA6',
            },
        },
        options: {
            cspNonce: 'dc77ae858e'
        }
    },
});

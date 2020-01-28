import axios from 'axios';
import store from '@/store/index.js';
import router from '../router';

export default () => {
    const instance = axios.create({
        baseURL: process.env.VUE_APP_API_HOST,
        withCredentials: true,
        timeout: 4000,
        headers: {
            Accept: "application/json",
            "Content-Type": "application/json"
        }
    });


    instance.interceptors.response.use(resp => {
        // do nothing with success
        return resp;
    }, err => {
        if (!err.response) {
            // no response could mean: cors issue, offline, timeout, etc..
            // (for the timeout, we could match on the error message)
            // more info: https://github.com/axios/axios/issues/383
            store.dispatch("setOffline");
        } else if (err.response.status === 401) {
            store.dispatch("unauthenticated").then(() => {
                router.push({ name: 'auth-login', query: { sessionExpired: true } });
            });
        }
        return Promise.reject(err);
    });


    return instance;
}

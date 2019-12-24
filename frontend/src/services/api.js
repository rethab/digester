import axios from 'axios';
import store from '@/store/index.js';
import router from '../router';

export default () => {
    const instance = axios.create({
        baseURL: process.env.VUE_APP_API_HOST,
        withCredentials: true,
        headers: {
            Accept: "application/json",
            "Content-Type": "application/json"
        }
    });


    instance.interceptors.response.use(resp => {
        // do nothing with success
        return resp;
    }, err => {
        if (err.response.status === 401) {
            store.dispatch("unauthenticated").then(() => {
                router.push({ name: 'auth-login', query: { sessionExpired: true } });
            });
        }
        return Promise.reject(err);
    });


    return instance;
}

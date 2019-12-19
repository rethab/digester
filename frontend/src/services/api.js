import axios from 'axios';

export default () => {
    return axios.create({
        baseURL: process.env.VUE_APP_API_HOST,
        withCredentials: true,
        headers: {
            Accept: "application/json",
            "Content-Type": "application/json"
        }
    });
}

import axios from 'axios';

export default () => {
    return axios.create({
        baseURL: 'http://localhost:8000/',
        withCredentials: false,
        headers: {
            Accept: "application/json",
            "Content-Type": "application/json"
        }
    });
}
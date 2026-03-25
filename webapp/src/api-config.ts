import axios from 'axios';
import {Configuration} from "./api-client";

const axiosInstance = axios.create();

axiosInstance.interceptors.request.use(config => {
    const token = localStorage.getItem('auth_token');
    if (token) {
        config.headers['Authorization'] = `Bearer ${token}`;
    }
    return config;
});

axiosInstance.interceptors.response.use(
    response => response,
    error => {
        if (error.response?.status === 401) {
            localStorage.removeItem('auth_token');
            window.location.reload();
        }
        return Promise.reject(error);
    }
);

export const apiConfig = new Configuration({
    basePath: "/api",
    axios: axiosInstance,
});

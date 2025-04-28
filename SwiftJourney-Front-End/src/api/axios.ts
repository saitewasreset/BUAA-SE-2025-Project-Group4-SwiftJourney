import { useUserStore } from '@/stores/user';
import axios from 'axios';

const instance = axios.create({
  baseURL: import.meta.env.VITE_API_URL,
  timeout: 10000,
});

instance.interceptors.request.use(
  (config) => {
    const userStore = useUserStore();
    const token = userStore.token;
    if (token) {
      config.headers['Authorization'] = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

const request = (config: Object) => {
    return instance.request(config);
}

export const getRequest = (url: string, params?: Object) => {
    return request({
        url,
        method: 'get',
        params
    });
};

export const postRequest = (url: string, data?: Object) => {
    return request({
        url,
        method: 'post',
        data
    });
};

export const postBlobRequest = (url: string, data?: Object) => {
    return request({
        url,
        method: 'post',
        data,
        responseType: 'blob'
    });
};

export const putRequest = (url: string, data?: Object) => {
    return request({
        url,
        method: 'put',
        data
    });
};
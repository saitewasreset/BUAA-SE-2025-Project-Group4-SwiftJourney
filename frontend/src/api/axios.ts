import axios from 'axios';

const instance = axios.create({
//   baseURL: import.meta.env.VITE_API_URL,
  timeout: 10000,
});

instance.interceptors.request.use(
  (config) => {
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

const request = (config: object) => {
    return instance.request(config);
}

export const getRequest = (url: string, params?: object) => {
    return request({
        url,
        method: 'get',
        params
    });
};

export const postRequest = (url: string, data?: object) => {
    return request({
        url,
        method: 'post',
        data
    });
};

export const postBlobRequest = (url: string, data?: object) => {
    return request({
        url,
        method: 'post',
        data,
        responseType: 'blob'
    });
};

export const putRequest = (url: string, data?: object) => {
    return request({
        url,
        method: 'put',
        data
    });
};
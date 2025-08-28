import { getRequest } from "../axios";
import { postRequest } from "../axios";

export const userApi = {
    userLogin: (params: object) => {
        return postRequest('/api/user/login', params);
    },
    setUserInfo: (params: object) => {
        return postRequest('/api/user/user_info', params);
    },
    updatePassword: (params: object) => {
        return postRequest('/api/user/update_password', params);
    },
    getUserInfo: () => {
        return getRequest('/api/user/user_info');
    },
    userRegister: (params: object) => {
        return postRequest('/api/user/register', params);
    },
    userLogout: () => {
        return postRequest('/api/user/logout');
    },
    queryUserBalance: () => {
        return getRequest('/api/payment/balance');
    },
    queryPersonalInfo: () => {
        return getRequest('/api/user/personal_info');
    },
    setPersonalInfo: (params: object) => {
        return postRequest('/api/user/personal_info', params);
    }
};
import { getRequest } from "../axios";
import { postRequest } from "../axios";
import { postBlobRequest } from "../axios";

export const userApi = {
    userLogin: (params: Object) => {
        return postRequest('/api/user/login', params);
    },
    getUserInfo: () => {
        return getRequest('/api/user/user_info');
    },
    userRegister: (params: Object) => {
        return postRequest('/api/user/register', params);
    },
    userLogout: () => {
        return postRequest('/api/user/logout');
    }
};
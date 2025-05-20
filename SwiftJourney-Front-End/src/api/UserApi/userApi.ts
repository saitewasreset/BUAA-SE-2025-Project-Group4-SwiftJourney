import { getRequest } from "../axios";
import { postRequest } from "../axios";
import { postBlobRequest } from "../axios";

export const userApi = {
    userLogin: (params: Object) => {
        return postRequest('/api/user/login', params);
    },
    getUserInfo: (params: Object) => {
        return postRequest('/api/user/user_info', params);
    },
    userRegister: (params: Object) => {
        return postRequest('/api/user/register', params);
    },
    userLogout: (params: Object) => {
        return postRequest('/api/user/logout', params);
    }
};
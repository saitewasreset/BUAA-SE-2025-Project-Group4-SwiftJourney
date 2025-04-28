import { getRequest } from "../axios";
import { postRequest } from "../axios";
import { postBlobRequest } from "../axios";

export const userApi = {
    userLogin: (params: Object) => {
        return postRequest('/api/user/login', params);
    }
};
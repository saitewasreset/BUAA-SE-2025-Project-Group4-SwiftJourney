import { getRequest } from "../axios";

export const orderApi = {
    orderList: () => {
        return getRequest('/api/order/list');
    }
};
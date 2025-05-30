import { getRequest, postRequest } from "../axios";

export const orderApi = {
    orderList: () => {
        return getRequest('/api/order/list');
    },
    orderCancel: (id: string) => {
        return postRequest('/api/order/cancel', {orderId: id});
    }
};
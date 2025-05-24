import { getRequest, postRequest } from "../axios";

export const hotelApi = {
    hotelQuery: (params: Object) => {
        return postRequest('/api/hotel/query', params);
    },
    hotelInfo: (hotel_id: string) => {
        return getRequest('/api/hotel/info/' + hotel_id);
    },
    hotelOrderInfo: (params: Object) => {
        return postRequest('/api/hotel/order_info', params);
    }
};
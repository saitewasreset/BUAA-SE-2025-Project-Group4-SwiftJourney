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
    },
    hotelOrder: (params: Object) => {
        return postRequest('api/hotel/order', params);
    },
    hotelQuota: (hotel_id: string) => {
        return getRequest('api/hotel/quota/' + hotel_id);
    },
    hotelComment: (params: Object) => {
        return postRequest('/api/hotel/comment', params);
    }
};
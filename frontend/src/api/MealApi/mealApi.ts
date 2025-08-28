import { postRequest } from "../axios";

export const mealApi = {
    dishQuery: (params: object) => {
        return postRequest('/api/dish/query', params);
    },
    dishOrder: (params: object) => {
        return postRequest('/api/dish/order', params);
    },
}
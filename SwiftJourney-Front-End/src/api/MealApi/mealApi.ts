import { postRequest} from "../axios";

export const mealApi = {
    dishQuery: (params: Object) => {
        return postRequest('/api/dish/query', params);
    },
}
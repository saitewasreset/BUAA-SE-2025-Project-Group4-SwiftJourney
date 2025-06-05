import { getRequest} from "../axios";

export const generalApi = {
    getCity: () => {
        return getRequest('/api/general/city');
    }
}
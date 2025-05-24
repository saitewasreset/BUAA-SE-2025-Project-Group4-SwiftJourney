import { postRequest } from "../axios";

export const paymentApi = {
    setPaymentPassword: (params: Object) => {
        return postRequest('api/payment/payment_password', params);
    }
}
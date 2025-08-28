import { postRequest } from "../axios";

export const paymentApi = {
    setPaymentPassword: (params: object) => {
        return postRequest('/api/payment/payment_password', params);
    },
    recharge: (params: object) => {
        return postRequest('/api/payment/recharge', params);
    },
    pay: (transactionId: string, params: object) => {
        return postRequest('/api/payment/pay/' + transactionId, params);
    }
}
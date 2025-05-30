import { postRequest } from "../axios";

export const paymentApi = {
    setPaymentPassword: (params: Object) => {
        return postRequest('/api/payment/payment_password', params);
    },
    recharge: (params: Object) => {
        return postRequest('/api/payment/recharge', params);
    },
    pay: (transactionId: string, params: Object) => {
        return postRequest('/api/payment/pay/' + transactionId, params);
    }
}
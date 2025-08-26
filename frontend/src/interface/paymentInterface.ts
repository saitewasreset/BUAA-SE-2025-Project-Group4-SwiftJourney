export interface PaymentApiResponseData {
  code: number;
  message: string;
  data: Object;
}
export interface RechargeRequest {
  amount: number;
  // 由于无需访问外部支付系统，故`externalPaymentId`设置为`null`即可。
  externalPaymentId: null;
}
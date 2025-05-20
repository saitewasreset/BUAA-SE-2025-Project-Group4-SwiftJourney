export interface UserInfo {
  username: string;
  gender?: "male" | "female";
  age?: number;
  phone: string;
  email?: string;
  // 当前用户是否设置了支付密码
  havePaymentPasswordSet: boolean;
  // 姓名
  name: string;
  // 身份证号
  identityCardId: string;
}
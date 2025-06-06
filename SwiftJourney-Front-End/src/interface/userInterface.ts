export interface UserApiResponseData {
  code: number;
  message: string;
  data: Object;
}
export interface UserApiBalanceData {
  code: number;
  message: string;
  data: {balance: number};
}
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

export interface PersonalInfo {
  // 该用户身份的 UUID
  personalId: string;
  // 姓名
  name: string;
  // 身份证号
  identityCardId: string;
  // 偏好座位位置
  preferredSeatLocation?: "A" | "B" | "C" | "D" | "F";
  // 是否为默认个人资料，即，当前用户的身份
  default: boolean;
}
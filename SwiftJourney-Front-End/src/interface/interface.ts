
export type ResponseData = TransactionData[];

export interface TransactionData {
  transactionId: string;
  status: "unpaid" | "paid";
  createTime: string;
  payTime?: string;
  orders: OrderInfo[];
  amount: number;
}

export interface OrderInfo {
  orderId: string;
  status: "unpaid" | "paid" | "ongoing" | "active" | "completed" | "failed" | "canceled";
  unitPrice: number;
  amount: number;
  orderType: "train" | "hotel" | "dish" | "takeaway";
  canCancel: boolean;
  reason?: string;
}

export interface SeatLocationInfo {
  carriage: number;
  row: number;
  location: string;
  type: string;
}

export interface TrainOrderInfo extends OrderInfo {
  // 车次，例如：“G53”
  trainNumber: string;
  // 始发站
  departureStation: string;
  // 终到站
  terminalStation: string;
  // 离开始发站日期时间
  departureTime: string;
  // 到达终到站的日期时间
  terminalTime: string;
  // 乘车人姓名
  name: string;
  // 人类可读的座位号
  seat: SeatLocationInfo;
}

export interface HotelOrderInfo extends OrderInfo {
  // 酒店名称
  hotelName: string;
  // 酒店 UUID
  hotelId: string;
  // 订房人姓名
  name: string;
  // 人类可读的房间类型，例如：“大床房”
  roomType: string;
  // 住宿开始日期
  beginDate: string;
  // 住宿结束日期
  endDate: string;
}

export interface DishOrderInfo extends OrderInfo {
  // 车次，例如：“G53”
  trainNumber: string;
  // 离开起始站日期时间
  depatureTime: string;
  // 用餐“时间”
  dishTime: "lunch" | "dinner";
  // 用餐人姓名
  name: string;
  // 人类可读的餐品名称
  dishName: string;
}

export interface TakeawayOrderInfo extends OrderInfo {
  // 车次，例如：“G53”
  trainNumber: string;
  // 离开起始站日期时间
  depatureTime: string;
  // 车站
  station: string;
  // 用餐时间（到达车站的时间）
  dishTime: string;
  // 店铺名称
  shopName: string;
  // 用餐人姓名
  name: string;
  // 人类可读的外卖餐品名称
  takeawayName: string;
}


export interface OrderInform {
    id: string, //订单编号
    status: string, //订单状态
    type: string, //订单类型
    money: string, //订单金额
    canCanceled: boolean, //是否可以被取消
    reason?: string,
}

export interface TransactionDetail {
    id: string, // 交易编号
    status: string, // 交易状态
    time: string, // 交易时间
    payTime?: string, //付款时间
    money: string, // 交易金额
    orderInfo: OrderInform[], // 订单表
}

export interface OrderDetail {
    id: string, //订单号
    name: string, //姓名
}

export interface TrainOrderDetail extends OrderDetail {
    depatureStation: string, //出发车站
    reachStation: string, //到达车站
    trainNumber: string, //车次
    date: string, //日期
    depatureTime: string, //出发时间
    seatInfo: string, //座位号 
}

export interface HotelOrderDetail extends OrderDetail {
    hotelName: string, //酒店名
    roomType: string, //房型
    beginDate: string, //入住日期
    endDate: string, //退房日期
    number: number, //数量
}

export interface FoodOrderDetail extends OrderDetail {
    shopName: string, //店铺名
    foodName: string, //食物名
    trainNumber: string, //车次
    station: string, //送餐车站(外卖)
    date: string, //日期
    time: string, //送餐时间
}

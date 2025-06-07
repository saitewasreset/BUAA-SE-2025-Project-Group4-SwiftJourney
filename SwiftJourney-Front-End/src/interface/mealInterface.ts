export interface DishInfo {
  // 该火车餐在哪些时段提供？
  availableTime: Array<"lunch" | "dinner">;
  // 火车餐名称
  name: string;
  // 火车餐类别，例如：主食、饮料、零食
  type: string;
  // 火车餐图片，URL
  picture: string;
  // 价格
  price: number;
}

export interface TakeawayDishInfo {
  // 餐品名称
  name: string;
  // 餐品图片，URL
  picture: string;
  // 价格
  price: number;
  // 该火车餐在哪些时段提供？
  availableTime?: Array<"lunch" | "dinner">;
  // 火车餐类别，例如：主食、饮料、零食
  type?: string;
}

export interface Takeaway {
  // 店铺名称
  shopName: string;
  station?: string;

  dishes: TakeawayDishInfo[];
}

export interface TrainDishInfo {
  // 车次
  trainNumber: string;
  // 离开“始发站”的日期时间
  originDepartureTime: string;
  // 到达“终到站”的日期时间
  terminalArrivalTime: string;

  dishes: TakeawayDishInfo[];

  // 车站名称 -> 可点的外卖列表
  takeaway: { [station: string]: Takeaway[] };

  // 能否预订
  canBooking: boolean;
  // 不能预订原因
  reason?: string;
}


export interface MealInfo {
    // 车次
  trainNumber: string;
  // 离开“始发站”的日期时间
  originDepartureTime: string;
  // 到达“终到站”的日期时间
  terminalArrivalTime: string;

  dishes: Takeaway[];

  // 能否预订
  canBooking: boolean;
  // 不能预订原因
  reason?: string;
}


export interface DishOrder {
  // 火车餐名称
  name: string;
  // 用餐人 UUID
  personalId: string;
  // 份数
  amount: number;
  // 用餐时间
  dishTime: "lunch" | "dinner";
}

export interface TakeawayOrder {
  // 车站名称
  station: string;
  // 店铺名称
  shopName: string;
  // 餐品名称
  name: string;
  // 用餐人 UUID
  personalId: string;
  // 份数
  amount: number;
}

export interface TrainDishOrderRequest {
  // 车次
  trainNumber: string;
  // 离开“始发站”的日期时间
  originDepartureTime: string;

  // 要预订的火车餐列表
  dishes: DishOrder[];
  // 要预订的外卖列表
  takeaway: TakeawayOrder[];
}

export interface MealOrder {
    // 车站名称
  station?: string;
  // 店铺名称
  shopName: string;
  // 餐品名称
  name: string;
  // 用餐人 UUID
  personalId: string;
  // 份数
  amount: number;
  // 用餐时间
  dishTime?: "lunch" | "dinner";
  price: number;
}

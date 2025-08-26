// -------------------- Interface 定义 --------------------
export type scheduleRequest = scheduleQuery

export interface scheduleQuery {
  departureStation?: string
  arrivalStation?: string
  departureCity?: string
  arrivalCity?: string
  // departureDate：YYYY-MM-DD
  departureDate: string
}

export type directScheduleResponseData = directScheduleInfo[]

// 站点停靠信息
export interface stoppingStationInfo {
  stationName: string
  // 到达该站点的日期时间，若为始发站，不包含该属性
  arrivalTime?: string
  // 离开该站点的日期时间，若为终到站，不包含该属性
  departureTime?: string
}

export interface seatTypeInfo {
  seatType: string
  // 该类型座位剩余量
  left: number
  // 这种座位的价格
  price: number
}

export interface directScheduleInfo {
  departureStation: string
  // 离开“起始站”的日期时间
  departureTime: string
  arrivalStation: string
  // 到达“到达站”的日期时间
  arrivalTime: string
  originStation: string
  // 离开“始发站”的日期时间
  originDepartureTime: string
  terminalStation: string
  // 到达“终到站”的日期时间
  terminalArrivalTime: string
  // 车次号，例如：“G53”
  trainNumber: string
  // 行程时间：到达“到达站”的时间 - 离开“起始站”的时间，单位：秒
  travelTime: number
  price: number
  // 车次经停车站信息
  route: stoppingStationInfo[]
  // 座席类别，如：二等座 -> SeatTypeInfo
  seatInfo: Map<string, seatTypeInfo>
}

export type indirectScheduleResponseData = indirectScheduleInfo[];

export interface indirectScheduleInfo {
  // 中转乘车第一程的信息
  first_ride: directScheduleInfo;
  // 中转乘车第二程的信息
  second_ride: directScheduleInfo;
  // 中间换乘可用的时间，单位：秒
  relaxing_time: number;
}

export type trainTransactionRequest = OrderPack[];

export interface OrderPack {
  // 原子操作，若为 true，则`orderList`中任意订单失败将回滚已成功的订单
  atomic: boolean;
  orderList: TrainOrderRequest[];
}

export interface TrainOrderRequest {
  // 车次号，例如：“G53”
  trainNumber: string;
  // 离开“始发站”的日期时间
  originDepartureTime: string;

  // 起始站
  departureStation: string;
  // 到达站
  arrivalStation: string;

  // 乘车人 Id（见`PersonalInfo`）
  personalId: string;
  // 座位类别，如：二等座
  seatType: string;
}

// -------------------- Type 定义 --------------------
// -------------------- 筛选相关 --------------------
export type CheckBoxGroup = {
  options: string[]
  checkedList: string[]
  indeterminate: boolean
  checkAll: boolean
}

export enum CheckType {
  TrainType = 0,
  SeatType = 1,
  DepartureStation = 2,
  TransferStation = 3,
  ArrivalStation = 4,
}

// -------------------- 排序相关 --------------------
export enum SortType {
  DepartureTime = 0, // 出发时间
  TravelTime = 1, // 运行时长
  Price = 2, // 价格
}

// -------------------- 查询相关 --------------------
export type QueryMode = 'direct' | 'indirect';


export interface TrainInfoQuery {
  // 车次号，例如：“G53”
  trainNumber: string;
  // 离开“始发站”的日期
  // departureDate：YYYY-MM-DD
  departureDate: string;
}

// 站点停靠信息
export interface StoppingStationInfo {
  stationName: string;
  // 到达该站点的日期时间，若为始发站，不包含该属性
  arrivalTime?: string;
  // 离开该站点的日期时间，若为终到站，不包含该属性
  departureTime?: string;
}

export interface TrainScheduleInfo {
  originStation: string;
  originDepartureTime: string;
  terminalStation: string;
  terminalArrivalTime: string;
  // departureDate：YYYY-MM-DD
  departureDate: string;
  // 车次经停车站信息
  route: StoppingStationInfo[];
}

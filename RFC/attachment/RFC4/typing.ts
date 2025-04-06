interface APIResponse<T> {
  code: number;
  message: string;
  data: T;
}

interface UserRegisterRequest {
  // 手机号
  phone: string;
  username: string;
  // 明文密码
  password: string;
}

interface UserLoginRequest {
  phone: string;
  // 明文密码
  password: string;
}
interface UserLoginInfo {
  // 是否是第一次登录
  isFirstLogin: boolean;
}

interface UserInfo {
  username: string;
  gender?: "male" | "female";
  age?: number;
  phone?: string;
  email?: string;
  // 当前用户是否设置了支付密码
  havePaymentPasswordSet: boolean;
}

interface UserUpdateInfo {
  username: string;
  gender?: "male" | "female";
  age?: number;
  phone: string;
  email: string;
}

interface PersonalInfo {
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

interface UpdatePersonalInfo {
  // 姓名
  name?: string;
  // 身份证号
  identityCardId: string;
  // 偏好座位位置
  preferredSeatLocation?: "A" | "B" | "C" | "D" | "F";
  // 是否为默认个人资料，即，当前用户的身份
  default?: boolean;
}

interface RechargeInfo {
  amount: number;
  // 由于无需访问外部支付系统，故`externalPaymentId`设置为`null`即可。
  externalPaymentId: null;
}

interface BalanceInfo {
  balance: number;
}

interface TransactionInfo {
  transactionId: string;
  amount: number;
  status: "unpaid" | "paid";
}

interface PaymentPasswordInfo {
  // 修改支付密码时，需要传入用户密码进行验证
  userPassword: string;
  paymentPassword: number;
}

interface PaymentConfirmation {
  userPassword?: string;
  paymentPassword?: number;
}

interface TrainScheduleQuery {
  depatureStation?: string;
  arrivalStation?: string;
  depatureCity?: string;
  arrivalCity?: string;
  // deparuteDate：YYYY-MM-DD
  deparuteDate: string;
}

// 站点停靠信息
interface StoppingStationInfo {
  stationName: string;
  // 到达该站点的日期时间，若为始发站，不包含该属性
  arrivalTime?: string;
  // 离开该站点的日期时间，若为终到站，不包含该属性
  depatureTime?: string;
}

interface SeatTypeInfo {
  // 该类型座位总计容量
  capacity: number;
  // 该类型座位剩余量
  remainCount: number;
  // 这种座位的价格
  price: number;
}

interface TransactionGenerateRequest {
  amount: number;
}

interface TrainScheduleInfo {
  depatureStation: string;
  // 离开“起始站”的日期时间
  depatureTime: string;
  arrivalStation: string;
  // 到达“到达站”的日期时间
  arrivalTime: string;
  originStation: string;
  // 离开“始发站”的日期时间
  originDepatureTime: string;
  terminalStation: string;
  // 到达“终到站”的日期时间
  terminalArrivalTime: string;
  // 车次号，例如：“G53”
  trainNumber: string;
  // 行程时间：到达“到达站”的时间 - 离开“起始站”的时间，单位：秒
  travelTime: number;
  price: number;
  // 车次经停车站信息
  route: StoppingStationInfo[];
  // 座位类型，如：二等座 -> SeatTypeInfo
  seatInfo: Map<string, SeatTypeInfo>;
}

interface IndirectTrainScheduleInfo {
  // 中转乘车第一程的信息
  firstRide: TrainScheduleInfo;
  // 中转乘车第二程的信息
  secondRide: TrainScheduleInfo;
  // 中间换乘可用的时间，单位：秒
  relaxingTime: number;
}

interface OrderPack {
  // 原子操作，若为 true，则`orderList`中任意订单失败将回滚已成功的订单
  atomic: boolean;
  orderList: TrainOrderRequest[];
}

interface TrainOrderRequest {
  // 车次号，例如：“G53”
  trainNumber: string;
  // 离开“始发站”的日期时间
  originDepatureTime: string;

  // 起始站
  depatureStation: string;
  // 到达站
  arrivalStation: string;

  // 乘车人 Id（见`PersonalInfo`）
  personalId: string;
  // 座位类别，如：二等座
  seatType: string;
}

interface OrderInfo {
  // 订单的 UUID
  orderId: string;
  // 订单对应的交易的 UUID
  transactionId: string;
  // 订单状态：详见 RFC3“关于订单状态的约定”
  status: "Unpaid" | "Paid" | "Ongoing" | "Active" | "Completed" | "Failed" | "Canceled";
  // 支付日期时间
  payTime?: string;
  // 订单金额
  amount: number;
  // 订单类型
  orderType: "train" | "hotel" | "dish" | "takeaway";
  // 订单是否能够取消
  canCancel: boolean;
  // 人类可读的不能取消订单的原因（若适用）
  reason?: string;
}

interface SeatLocationInfo {
  // 车厢号，例如：“03 车 12A 二等座”中的“3”
  carriage: number;
  // 座位行数，例如：“03 车 12A 二等座”中的“12”
  row: number;
  // 座位位置代码，例如：“03 车 12A 二等座”中的“A”
  location: string;
  // 座位类型，例如：“03 车 12A 二等座”中的“二等座”
  type: string;
}

interface TrainOrderInfo extends OrderInfo {
  // 车次，例如：“G53”
  trainNumber: string;
  // 离开起始站日期时间
  depatureTime: string;
  // 乘车人姓名
  name: string;
  // 人类可读的座位号
  seat: SeatLocationInfo;
}

interface HotelOrderInfo extends OrderInfo {
  // 酒店名称
  hotelName: string;
  // 酒店 UUID
  hotelId: string;
  // 旅客姓名
  name: string;
  // 人类可读的房间类型，例如：“大床房”
  roomType: string;
  // 住宿开始日期
  beginDate: string;
  // 住宿结束日期
  endDate: string;
}

interface DishOrderInfo extends OrderInfo {
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

interface TakeawayOrderInfo extends OrderInfo {
  // 车次，例如：“G53”
  trainNumber: string;
  // 离开起始站日期时间
  depatureTime: string;
  // 车站
  station: string;
  // 店铺名称
  shopName: string;
  // 用餐人姓名
  name: string;
  // 人类可读的外卖餐品名称
  takeawayName: string;
}

interface CancelOrderInfo {
  // 订单的 UUID
  orderId: string;
}

interface HotelQuery {
  // 目标城市/火车站，由`target_type`属性指定
  target: string;
  target_type: "city" | "station";
  // 通过酒店名称进行匹配，可不存在
  search?: string;
  // 入住日期
  beginDate?: string;
  // 离开日期
  endDate?: string;
}

// 酒店的总体信息
interface HotelGeneralInfo {
  // 酒店 UUID
  hotelId: string;
  // 酒店名称
  name: string;
  // 酒店图片，URL
  picture?: string;
  // 酒店评分
  rating: number;
  // 评分人数
  ratingCount: number;
  // 累计预订人次
  totalBookings: number;
  price: number;
}

// 用户评价
interface HotelComment {
  // 用户名
  username: string;
  // 留言日期时间
  commentTime: string;
  // 评分
  rating: number;
  // 留言内容
  comment: string;
}

// 酒店的总体信息
interface HotelDetailInfo {
  // 酒店 UUID
  hotelId: string;
  // 酒店名称
  name: string;
  // 酒店详细地址
  address: string;
  // 联系电话
  phone: string[];

  // 酒店图片列表，URL
  picture?: string[];
  // 酒店评分
  rating: number;
  // 评分人数
  ratingCount: number;
  // 累计预订人次
  totalBookings: number;
  // 用户留言
  comments: HotelComment[];
}

interface HotelOrderQuery {
  // 欲查询酒店的 UUID
  hotelId: string;
  // 入住日期
  beginDate?: string;
  // 离开日期
  endDate?: string;
}

// 房间预订信息
interface HotelRoomDetailInfo {
  // 该类型房间总可入住人数
  capacity: number;
  // 该类型房间剩余容量
  remainCount: number;
  // 价格
  price: number;
}

interface HotelOrderRequest {
  // 欲预订酒店的 UUID
  hotelId: string;

  // 人类可读的房间类型
  roomType: string;

  // 入住日期
  beginDate?: string;
  // 离开日期
  endDate?: string;

  // 旅客 UUID（见`PersonalInfo`）
  personalId: string;
}

interface HotelCommentQuota {
  // 用户可为该酒店撰写评价数量配额（该酒店的已完成订单数量）
  quota: number;
  // 已使用配额
  used: number;
}

interface NewHotelComment {
  // 欲评价酒店的 UUID
  hotelId: string;
  // 评分
  rating: number;
  // 留言内容
  comment: string;
}

interface DishQuery {
  // 车次
  trainNumber: string;
  // 离开“始发站”的日期时间
  originDepatureTime: string;
}

interface DishInfo {
  // 该火车餐在哪些时段提供？
  availableTime: Array<"launch" | "dinner">;
  // 火车餐名称
  name: string;
  // 火车餐图片，URL
  picture: string;
  // 价格
  price: number;
}

interface TrainDishInfo {
  // 车次
  trainNumber: string;
  // 离开“始发站”的日期时间
  originDepatureTime: string;
  // 到达“终到站”的日期时间
  terminalArrivalTime: string;

  dishes: DishInfo[];

  // 车站名称 -> 可点的外卖列表
  takeaway: Map<string, Takeaway[]>;

  // 能否预订
  canBooking: boolean;
  // 不能预订原因
  reason?: string;
}

interface DishStationQuery {
  // 起始站
  depatureStation: string;
  // 到达站
  arrivalStation: string;
  // 查询日期
  targetDate: string;
}

interface DishInfo {
  // 该火车餐在哪些时段提供？
  availableTime: Array<"launch" | "dinner">;
  // 火车餐名称
  name: string;
  // 火车餐类别，例如：主食、饮料、零食
  type: string;
  // 火车餐图片，URL
  picture: string;
  // 价格
  price: number;
}

interface TakeawayDishInfo {
  // 餐品名称
  name: string;
  // 餐品图片，URL
  picture: string;
  // 价格
  price: number;
}

interface Takeaway {
  // 店铺名称
  shopName: string;

  dishes: TakeawayDishInfo[];
}

interface FullTrainDishInfo {
  // 车次
  trainNumber: string;

  depatureStation: string;
  // 离开“起始站”的日期时间
  depatureTime: string;
  arrivalStation: string;
  // 到达“到达站”的日期时间
  arrivalTime: string;
  originStation: string;
  // 离开“始发站”的日期时间
  originDepatureTime: string;
  terminalStation: string;
  // 到达“终到站”的日期时间
  terminalArrivalTime: string;

  dishes: DishInfo[];
  // 车站名称 -> 可点的外卖列表
  takeaway: Map<string, Takeaway[]>;

  // 能否预订
  canBooking: boolean;
  // 不能预订原因
  reason?: string;
}

interface DishOrder {
  // 火车餐名称
  name: string;
  // 用餐人 UUID
  personalId: string;
  // 份数
  amount: number;
  // 用餐时间
  dishTime: "launch" | "dinner";
}

interface TakeawayOrder {
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

interface TrainDishOrderRequest {
  // 车次
  trainNumber: string;
  // 离开“始发站”的日期时间
  originDepatureTime: string;

  // 要预订的火车餐列表
  dishes: DishOrder[];
  // 要预订的外卖列表
  takeaway: TakeawayOrder[];
}

interface Message<T> {
  // 标识该消息的类型，即`T`，由于 WebSocket 在同一个端点中传递多种类型的消息，需要通过`type`确定收到的消息类型。
  type: string;
  data: T;
}

interface Notify {
  // 消息标题
  title: string;
  // 发送的日期时间
  messageTime: string;
  // 提醒类型
  type: "order" | "trip";
}

// OrderInfo 定义详见“订单列表、订单详情”
interface OrderNotify extends Notify {
  order: OrderInfo;
}

interface TripNotify extends Notify {
  // 车次，例如：“G53”
  trainNumber: string;
  // 离开起始站日期时间
  depatureTime: string;
  // 起始站
  depatureStation: string;
  // 到达站
  arrivalStation: string;
}

interface DishTakeawayInfo {
  dishes: DishInfo[];
  // 车站 -> Takeaway[]
  takeaway: Map<string, Takeaway[]>;
}

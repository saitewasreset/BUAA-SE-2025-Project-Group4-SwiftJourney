export interface HotelQuery {
  // 目标城市/火车站，由`target_type`属性指定
  target: string;
  targetType: "city" | "station";
  // 通过酒店名称进行匹配，可不存在
  search?: string;
  // 入住日期
  beginDate?: string;
  // 离开日期
  endDate?: string;
}

// 酒店的总体信息
export interface HotelGeneralInfo {
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
  info: string;
}

// 用户评价
export interface HotelComment {
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
export interface HotelDetailInfo {
  // 酒店 UUID
  hotelId: string;
  // 酒店名称
  name: string;
  // 酒店详细地址
  address: string;
  // 联系电话
  phone: string[];
  info: string;

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

export interface HotelOrderQuery {
  // 欲查询酒店的 UUID
  hotelId: string;
  // 入住日期
  beginDate?: string;
  // 离开日期
  endDate?: string;
}

export interface HotelRoomDetailInfo {
  // 该类型房间总可入住人数
  capacity: number;
  // 该类型房间剩余容量
  remainCount: number;
  // 价格
  price: number;
}

export interface HotelGInfoWRoom extends HotelGeneralInfo {
  roomTypeMap: Map<string, HotelRoomDetailInfo>,
}

export interface HotelOrderRequest {
  // 欲预订酒店的 UUID
  hotelId: string;

  // 人类可读的房间类型
  roomType: string;

  // 入住日期
  beginDate: string;
  // 离开日期
  endDate: string;

  // 预订人 UUID（见`PersonalInfo`）
  personalId: string;
  // 预订数量
  amount: number;
}

export interface HotelRoomInfo extends HotelRoomDetailInfo {
    roomType: string,
}

export interface HotelOrderInfo extends HotelOrderRequest {
  name: string,
  maxCount: number,
  price: number,
}

export interface HotelCommentQuota {
  // 用户可为该酒店撰写评价数量配额（该酒店的已完成订单数量）
  quota: number;
  // 已使用配额
  used: number;
}

export interface NewHotelComment {
  // 欲评价酒店的 UUID
  hotelId: string;
  // 评分
  rating: number;
  // 留言内容
  comment: string;
}
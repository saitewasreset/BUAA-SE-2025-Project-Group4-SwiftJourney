export interface HotelQuery {
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
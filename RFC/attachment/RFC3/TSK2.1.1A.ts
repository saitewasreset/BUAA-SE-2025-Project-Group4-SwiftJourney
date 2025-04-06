type HotelSchema = HotelInfo[];

interface HotelInfo {
  // 酒店名称
  name: string;

  // 人类可读的酒店地址
  address: string;
  // 酒店所在城市，该城市必须在`CitySchema`（见`TSK1.2.1A.ts`）中出现过
  city: string;
  // 酒店临近的火车站，该车站必须在`StationSchema`（见`TSK1.2.1A.ts`）中出现过
  station?: string;
  // 酒店的图片，使用相对 JSON 文件的路径，例如：`./images/a.png`
  images: string[];
  // 酒店联系电话
  phone: string[];
  // 酒店简介
  info: string;
  // 酒店房间信息：人类可读的房间类型 -> HotelRoomType
  room_info: Map<string, HotelRoomType>;
  // 酒店评价
  comments: HotelComment[];
}

interface HotelRoomType {
  // 该酒店中该类房间的总计容量
  capacity: number;
  // 该酒店中该类房间的价格
  price: number;
}

interface HotelComment {
  // 留言的日期时间，格式参见 RFC4
  time: string;
  // 评分，为 [0.0, 5.0] 间的实数
  rating: number;
  // 留言内容
  text: string;
}

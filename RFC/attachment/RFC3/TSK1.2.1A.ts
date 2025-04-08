// 搜集的数据需要满足下文中`xxxSchema`规定的类型

// 城市

type CitySchema = string[];

// 车站

type StationSchema = StationInfo[];

interface StationInfo {
  name: string;
  // 车站位于哪个城市，该城市必须在`CitySchema`中出现过
  city: string;
}

// 车型及座位信息

type TrainTypeSchema = TrainTypeInfo[];

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

interface SeatInfo {
  // 人类可读的座位信息
  description: SeatLocationInfo;
  // 座位价格
  price: number;
}

interface TrainTypeInfo {
  // 例如："G"、"D"
  id: string;
  // 例如：高铁，动车，特快
  name: string;
  // 座位类型 -> 座位位置 -> 座位信息 []
  seat: Map<string, Map<string, SeatInfo[]>>;
}

// 车次信息

type TrainNumberSchema = TrainNumberInfo[];

interface RouteStationInfo {
  // 顺序编号
  order: number;
  // 站点名称，该站点必须在`StationSchema`中出现过
  station: string;
  // 到达该站点时间，用“距离始发时间的秒数”记录，始发站为 0
  arrivalTime: number;
  // 离开该站点时间，用“距离始发时间的秒数”记录，始发站为 0，终到站为 0
  depatureTime: number;
}

interface TrainNumberInfo {
  // 车次名，例如：G53
  train_number: string;
  // 车型名，例如：G，该车型必须在`TrainTypeSchema`中出现过
  train_type: string;
  // 从始发站出发的时间，不含日期，采用从 00:00:00 开始的秒数
  originDepatureTime: number;
  // 该车次经停站
  route: RouteStationInfo[];
}

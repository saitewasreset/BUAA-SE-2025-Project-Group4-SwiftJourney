type DishTakeawaySchema = DishTakeawayPack[];

interface DishTakeawayPack {
  // 车次，必须在`TrainNumberSchema`（见`TSK1.2.1A.ts`）中出现过
  trainNumber: string;

  dishInfo: DishInfo[];
  // 车站 -> 商户名 -> 提供的外卖列表
  // 车站必须在`StationSchema`（见`TSK1.2.1A.ts`）中出现过
  takeawayInfo: Map<string, Map<string, TakeawayDishInfo[]>>;
}

interface DishInfo {
  // 该火车餐在哪些时段提供？
  availableTime: Array<"launch" | "dinner">;
  // 火车餐名称
  name: string;
  // 火车餐类别，例如：主食、饮料、零食
  type: string;
  // 火车餐图片，使用相对 JSON 文件的路径，例如：`./images/a.png`
  picture: string;
  // 价格
  price: number;
}

interface TakeawayDishInfo {
  // 餐品名称
  name: string;
  // 餐品图片，使用相对 JSON 文件的路径，例如：`./images/a.png`
  picture: string;
  // 价格
  price: number;
}

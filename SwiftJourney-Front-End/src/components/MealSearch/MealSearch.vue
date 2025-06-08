<template>
  <div class="meal-search">
    <!-- 现代化搜索卡片 - 始终在顶部 -->
    <el-card shadow="hover" class="search-card">
      <!-- 搜索标题区域 -->
      <div class="search-title-section">
        <h2 class="search-title">餐食预订</h2>
        <p class="search-subtitle">查询列车餐食服务，享受旅途美食</p>
      </div>

      <!-- 主要搜索区域 -->
      <div class="search-container">
        <!-- 车次输入区域 -->
        <div class="train-input-section">
          <label class="input-label">车次号码</label>
          <a-input
            class="train-input"
            v-model:value="trainNumber"
            :bordered="false"
            size="large"
            placeholder="请输入车次号（如：G1234）"
          />
        </div>

        <!-- 日期选择区域 -->
        <div class="date-selection">
          <label class="input-label">出发日期</label>
          <a-date-picker
            suffix-icon=""
            id="MealDatePicker"
            class="date-picker"
            v-model:value="originDepartureTime"
            size="large"
            :locale="locale"
            :format="dateFormat"
            :bordered="false"
            :allow-clear="false"
            :disabled-date="disabledDate"
            placeholder="选择列车出发日期"
          />
        </div>

        <!-- 搜索按钮 -->
        <div class="search-button-wrapper">
          <a-button type="primary" size="large" class="search-button" @click="getTrainInfo">
            <template #icon>
              <SearchOutlined />
            </template>
            <span class="search-text">查询餐食</span>
          </a-button>
        </div>
      </div>
    </el-card>

    <!-- 站点筛选区域 -->
    <div v-if="!isHeadPage" class="StationSelected">
      <div class="station-filters">
        <el-checkbox
          v-for="(station, index) in stations"
          :key="index"
          v-model="stationsShow[station]"
          class="station-checkbox"
        >
          {{ station }}{{ station == '餐车' ? '' : '站' }}
        </el-checkbox>
      </div>
    </div>

    <!-- 结果展示区域 -->
    <div v-if="!isHeadPage" class="Grid">
      <div class="order-card-container" id="details-section"><SelectCard /></div>
      <el-scrollbar height="600px" class="DishInfo">
        <div v-if="false" class="HotelUnFind">
          <img class="UnfindImage" src="../../assets/unfind.jpg" alt="unfind" />
          <p style="text-align: center">没有搜索到符合条件的餐食，请重新输入</p>
        </div>
        <div v-for="(info, index) in dishInfo?.dishes" :key="index">
          <el-card
            v-if="info.station ? stationsShow[info.station] : stationsShow['餐车']"
            class="DishInfoCard"
            shadow="always"
          >
            <div class="card-header">
              <p class="ShopName">{{ info.shopName }}</p>
            </div>
            <el-table class="DishInfoTable" :data="info.dishes" border>
              <el-table-column label="图片" width="100">
                <template #default="scope">
                  <div class="food-image-container">
                    <img
                      class="FoodImage"
                      :src="scope.row.picture"
                      alt="food-image"
                      @click="showImage(scope.row.picture)"
                    />
                  </div>
                </template>
              </el-table-column>
              <el-table-column prop="name" label="餐品名称" min-width="120"></el-table-column>
              <el-table-column v-if="info.shopName == '餐车'" label="提供时段" width="100">
                <template #default="scope">
                  <div class="time-tag">{{ lunchChange(scope.row.availableTime) }}</div>
                </template>
              </el-table-column>
              <el-table-column
                v-if="info.shopName == '餐车'"
                prop="type"
                label="类别"
                width="80"
              ></el-table-column>
              <el-table-column prop="price" label="价格" sortable width="80">
                <template #default="scope">
                  <div class="price-tag">SC {{ scope.row.price }}</div>
                </template>
              </el-table-column>
              <el-table-column label="操作" width="80">
                <template #default="scope">
                  <el-tooltip :content="dishInfo?.reason" :disabled="dishInfo?.canBooking">
                    <el-button
                      class="OrderButton"
                      type="primary"
                      size="small"
                      :disabled="!dishInfo?.canBooking"
                      @click="
                        info.shopName == '餐车'
                          ? orderDish(info.shopName, scope.row)
                          : orderMeal(info.shopName, scope.row, info.station)
                      "
                    >
                      订购
                    </el-button>
                  </el-tooltip>
                </template>
              </el-table-column>
            </el-table>
          </el-card>
        </div>
      </el-scrollbar>
    </div>

    <!-- 无搜索结果时的占位内容 -->
    <div v-if="isHeadPage" class="empty-state">
      <div class="empty-content">
        <h3 class="empty-title">开始您的餐食预订之旅</h3>
        <p class="empty-subtitle">输入车次和日期，发现列车上的美食世界</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { h, ref, nextTick } from 'vue'
import { SearchOutlined } from '@ant-design/icons-vue'
import { mealApi } from '@/api/MealApi/mealApi'
import { TicketServiceApi } from '@/api/TicketServiceApi/TicketServiceApi'
import locale from 'ant-design-vue/es/date-picker/locale/zh_CN'
import dayjs from 'dayjs'
import 'dayjs/locale/zh-cn'
import { ElMessage, ElMessageBox, ElOption, ElSelect } from 'element-plus'
import type { TrainDishInfo, MealInfo, Takeaway, TakeawayDishInfo } from '@/interface/mealInterface'
import type { TrainScheduleInfo } from '@/interface/ticketServiceInterface'
import SelectCard from './MealOrderCard.vue'
import { useMealOrderStore } from '@/stores/mealOrder'

const mealOrderStore = useMealOrderStore()

const today = dayjs()
dayjs.locale('zh-cn')

const isHeadPage = ref(true)
const trainNumber = ref('')
const originDepartureTime = ref(today)

const dateFormat = 'YYYY-MM-DD(dddd)'

function disabledDate(current: any) {
  return (
    current &&
    (current < dayjs().startOf('day').subtract(2, 'day') ||
      current > dayjs().startOf('day').add(15, 'day'))
  )
}

async function getTrainInfo() {
  if (!checkInput()) return

  await TicketServiceApi.trainSchedule({
    trainNumber: trainNumber.value.trim(),
    departureDate: originDepartureTime.value.format('YYYY-MM-DD'),
  })
    .then((res: any) => {
      if (res.status == 200) {
        if (res.data.code == 200) {
          getMail(res.data.data)
        } else if (res.data.code == 404) {
          ElMessage.error('查询的车次不存在')
        } else if (res.data.code == 403) {
          ElMessage.error('会话无效')
        }
      }
    })
    .catch((err: any) => {
      ElMessage.error(err)
    })
}
function checkInput(): boolean {
  if (trainNumber.value.trim() === '') {
    ElMessage.error('请输入您要查询车次')
    return false
  }
  return true
}

const departureTime = ref('')
const stations = ref<string[]>([])
const stationsShow = ref<{ [stations: string]: boolean }>({})

async function getMail(trainInfo: TrainScheduleInfo) {
  departureTime.value = trainInfo.originDepartureTime
  const tepStations: string[] = []
  const tepStationsMap: { [stations: string]: boolean } = {}
  trainInfo.route.forEach((value) => {
    tepStations.push(value.stationName)
    tepStationsMap[value.stationName] = true
  })
  tepStations.push('餐车')
  tepStationsMap['餐车'] = true
  stations.value = tepStations
  stationsShow.value = tepStationsMap

  await mealApi
    .dishQuery({
      trainNumber: trainNumber.value.trim(),
      originDepartureTime: departureTime.value,
    })
    .then((res: any) => {
      if (res.status == 200) {
        if (res.data.code == 200) {
          successGetMeal(res.data.data)
        } else if (res.data.code == 404) {
          ElMessage.error('查询的车次不存在')
        } else if (res.data.code == 403) {
          ElMessage.error('会话无效')
        }
      }
    })
    .catch((err: any) => {
      ElMessage.error(err)
    })
}
const dishInfo = ref<MealInfo>()
function successGetMeal(trainDishInfo: TrainDishInfo) {
  const tepInfo: MealInfo = {
    trainNumber: trainDishInfo.trainNumber,
    originDepartureTime: trainDishInfo.originDepartureTime,
    terminalArrivalTime: trainDishInfo.terminalArrivalTime,
    canBooking: trainDishInfo.canBooking,
    reason: trainDishInfo.reason,
    dishes: [],
  }

  const canche: Takeaway = {
    shopName: '餐车',
    dishes: trainDishInfo.dishes,
  }
  tepInfo.dishes.push(canche)

  for (const station in trainDishInfo.takeaway) {
    trainDishInfo.takeaway[station].forEach((value) => {
      const tepShopInfo: Takeaway = {
        ...value,
        station: station,
      }
      tepInfo.dishes.push(tepShopInfo)
    })
  }

  mealOrderStore.deleteAll()
  dishInfo.value = tepInfo
  isHeadPage.value = false
  nextTick(() => {
    const detailsSection = document.getElementById('details-section');
    if (detailsSection) {
      detailsSection.scrollIntoView({ behavior: 'smooth' });
    }
  });
}

const orderDish = (shopName: string, food: TakeawayDishInfo) => {
  if (food.availableTime?.length == 1) {
    orderMeal(shopName, food, undefined, food.availableTime[0])
  } else {
    const select = ref<'lunch' | 'dinner'>('lunch')
    ElMessageBox({
      title: '请选择用餐时段',
      message: () =>
        h(
          ElSelect,
          {
            modelValue: select.value,
            'onUpdate:modelValue': (val: 'lunch' | 'dinner') => {
              select.value = val
            },
            style: { width: '100px' },
          },
          [
            h(ElOption, {
              key: 'lunch',
              label: '午餐',
              value: 'lunch',
            }),
            h(ElOption, {
              key: 'dinner',
              label: '晚餐',
              value: 'dinner',
            }),
          ],
        ),
      type: 'info',
      confirmButtonText: '确定',
    })
      .then(() => {
        orderMeal(shopName, food, undefined, select.value)
      })
      .catch((err) => {
        console.log(err)
      })
  }
}

const orderMeal = (
  shopName: string,
  food: TakeawayDishInfo,
  station?: string,
  dishTime?: 'lunch' | 'dinner',
) => {
  if (
    !mealOrderStore.add(trainNumber.value, departureTime.value, shopName, food, station, dishTime)
  ) {
    ElMessage.error('不可在同一订单为不同车次预订餐品')
  } else {
    ElMessage.success('加入预订列表成功，可在预定列表中修改数量')
  }
}

const lunchChangeTab = {
  lunch: '午餐',
  dinner: '晚餐',
}

const lunchChange = (time: ('lunch' | 'dinner')[]) => {
  let str = ''
  time.forEach((value) => {
    str = str + lunchChangeTab[value] + ' '
  })
  return str
}

const showImage = (src: string) => {
  window.open(src, '_blank')
}
</script>

<style scoped>
.meal-search {
  width: 100%;
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  overflow: auto;
}

/* 搜索卡片 - 始终在顶部 */
.search-card {
  border-radius: 16px;
  border: none;
  background: linear-gradient(135deg, #ffffff 0%, #f8f9ff 100%);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
  transition: all 0.3s ease;
  position: relative;
  margin-bottom: 20px;
}

.search-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.12);
}

.search-card :deep(.el-card__body) {
  padding: 32px 40px;
}

/* 搜索标题区域 */
.search-title-section {
  margin-bottom: 32px;
  text-align: center;
  padding-bottom: 20px;
  border-bottom: 1px solid #e4e7ed;
}

.search-title {
  font-size: 28px;
  font-weight: 700;
  color: #303133;
  margin: 0 0 8px 0;
  background: linear-gradient(135deg, #409eff, #67c23a);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.search-subtitle {
  font-size: 16px;
  color: #909399;
  margin: 0;
  font-weight: 500;
}

/* 主搜索容器 */
.search-container {
  display: flex;
  align-items: flex-end;
  gap: 24px;
  flex-wrap: wrap;
  justify-content: center;
}

/* 车次输入区域 */
.train-input-section {
  min-width: 240px;
  background: #f8f9fa;
  border-radius: 12px;
  padding: 24px 20px 20px;
  transition: all 0.3s ease;
  flex: 1;
  max-width: 300px;
}

.train-input-section:hover {
  background: #f0f2f5;
  transform: translateY(-1px);
}

.input-label {
  display: block;
  font-size: 14px;
  color: #909399;
  margin-bottom: 8px;
  font-weight: 500;
}

.train-input {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  background: transparent;
}

.train-input :deep(.ant-input) {
  font-size: 16px;
  font-weight: 600;
  background: transparent;
  color: #303133;
}

.train-input :deep(.ant-input::placeholder) {
  color: #c0c4cc;
  font-weight: 400;
}

.train-input:focus {
  color: #409eff;
}

/* 日期选择区域 */
.date-selection {
  min-width: 240px;
  background: #f8f9fa;
  border-radius: 12px;
  padding: 24px 20px 20px;
  transition: all 0.3s ease;
  flex: 1;
  max-width: 300px;
}

.date-selection:hover {
  background: #f0f2f5;
  transform: translateY(-1px);
}

.date-picker {
  width: 100%;
  background: transparent;
}

.date-picker :deep(.ant-picker-input) {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.date-picker :deep(.ant-picker-input input) {
  font-weight: 600;
  background: transparent;
}

.date-picker :deep(.ant-picker-input input::placeholder) {
  color: #c0c4cc;
  font-weight: 400;
}

/* 搜索按钮区域 */
.search-button-wrapper {
  display: flex;
  align-items: center;
}

.search-button {
  height: 88px;
  width: 120px;
  border-radius: 12px;
  font-size: 16px;
  font-weight: 600;
  background: linear-gradient(135deg, #409eff 0%, #67c23a 100%);
  border: none;
  box-shadow: 0 4px 16px rgba(64, 158, 255, 0.3);
  transition: all 0.3s ease;
}

.search-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(64, 158, 255, 0.4);
  background: linear-gradient(135deg, #337ecc 0%, #5daf34 100%);
}

.search-button:active {
  transform: translateY(0);
}

.search-text {
  margin-left: 8px;
  font-weight: 600;
}

/* 空状态占位内容 */
.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
}

.empty-content {
  text-align: center;
  max-width: 400px;
  padding: 40px 20px;
}

.empty-image {
  width: 200px;
  height: auto;
  margin-bottom: 24px;
  opacity: 0.8;
  filter: drop-shadow(0 4px 16px rgba(64, 158, 255, 0.1));
}

.empty-title {
  font-size: 24px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 12px 0;
  background: linear-gradient(135deg, #409eff, #67c23a);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.empty-subtitle {
  font-size: 16px;
  color: #909399;
  margin: 0;
  line-height: 1.5;
}

/* 站点筛选区域美化 */
.StationSelected {
  margin-bottom: 20px;
  padding: 20px;
  background: linear-gradient(135deg, #ffffff 0%, #f8f9ff 100%);
  border-radius: 16px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
  border: 1px solid #e4e7ed;
}

.station-filters {
  display: flex;
  justify-content: center;
  flex-wrap: wrap;
  gap: 12px;
}

.station-checkbox {
  background: rgba(64, 158, 255, 0.08);
  padding: 10px 16px;
  border-radius: 20px;
  border: 1px solid rgba(64, 158, 255, 0.2);
  transition: all 0.3s ease;
}

.station-checkbox:hover {
  background: rgba(64, 158, 255, 0.12);
  border-color: #409eff;
  transform: translateY(-1px);
}

::v-deep(.station-checkbox .el-checkbox__label) {
  color: #303133;
  font-weight: 500;
  font-size: 14px;
}

::v-deep(.station-checkbox .el-checkbox__input.is-checked + .el-checkbox__label) {
  color: #409eff;
  font-weight: 600;
}

::v-deep(.station-checkbox .el-checkbox__input.is-checked .el-checkbox__inner) {
  background: linear-gradient(135deg, #409eff, #67c23a);
  border-color: #409eff;
}

::v-deep(.station-checkbox .el-checkbox__inner) {
  border: 1px solid #c0c4cc;
  border-radius: 3px;
  transition: all 0.3s ease;
}

::v-deep(.station-checkbox .el-checkbox__inner:hover) {
  border-color: #409eff;
}

/* 网格布局美化 */
.Grid {
  display: flex;
  justify-content: flex-start;
  gap: 15px;
  width: 100%;
  flex: 1;
  min-height: 0;
}

.order-card-container {
  width: 300px;
  flex-shrink: 0;
}

.DishInfo {
  flex: 1;
  min-width: 0;
  background: linear-gradient(135deg, #ffffff 0%, #f8f9ff 100%);
  border-radius: 16px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
  border: 1px solid #e4e7ed;
  overflow: hidden;
}

/* 餐食卡片美化 */
.DishInfoCard {
  margin-bottom: 20px;
  border-radius: 16px;
  background: linear-gradient(135deg, #ffffff 0%, #f8f9ff 100%);
  border: 1px solid #e4e7ed;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
  transition: all 0.3s cubic-bezier(0.23, 1, 0.32, 1);
  overflow: hidden;
}

.DishInfoCard:hover {
  transform: translateY(-3px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  border-color: #409eff;
}

.card-header {
  background: linear-gradient(135deg, #409eff, #67c23a);
  margin: -20px -20px 15px -20px;
  padding: 15px 20px;
}

.ShopName {
  text-align: center;
  font-size: 1.3rem;
  font-weight: 700;
  color: white;
  margin: 0;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

/* 表格美化 */
.DishInfoTable {
  border-radius: 12px;
  overflow: hidden;
  border: none;
}

::v-deep(.DishInfoTable .el-table__header) {
  background: linear-gradient(135deg, #f8f9ff, #f0f2f5);
}

::v-deep(.DishInfoTable .el-table__header th) {
  background: linear-gradient(135deg, #f8f9ff, #f0f2f5);
  color: #303133;
  font-weight: 600;
  border-color: rgba(64, 158, 255, 0.1);
  text-align: center;
  padding: 12px 8px;
}

::v-deep(.DishInfoTable .el-table__body tr) {
  transition: all 0.3s ease;
}

::v-deep(.DishInfoTable .el-table__body tr:hover) {
  background: rgba(64, 158, 255, 0.05);
}

::v-deep(.DishInfoTable .el-table__body td) {
  border-color: rgba(64, 158, 255, 0.1);
  text-align: center;
  padding: 8px;
  color: #303133;
  font-weight: 500;
}

::v-deep(.el-table .el-table__cell) {
  text-align: center;
  padding: 8px;
}

::v-deep(.el-table .cell) {
  padding: 4px;
}

/* 图片容器美化 */
.food-image-container {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 60px;
  height: 60px;
  margin: 0 auto;
  border-radius: 12px;
  overflow: hidden;
  border: 2px solid rgba(64, 158, 255, 0.2);
  transition: all 0.3s ease;
}

.food-image-container:hover {
  border-color: #409eff;
  box-shadow: 0 4px 15px rgba(64, 158, 255, 0.3);
}

.FoodImage {
  width: 100%;
  height: 100%;
  object-fit: cover;
  cursor: pointer;
  transition: transform 0.3s cubic-bezier(0.23, 1, 0.32, 1);
  filter: contrast(1.1) saturate(1.1);
}

.FoodImage:hover {
  transform: scale(1.1) rotate(1deg);
}

/* 时间标签美化 */
.time-tag {
  background: linear-gradient(135deg, #67c23a, #409eff);
  color: white;
  padding: 4px 8px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
  display: inline-block;
  box-shadow: 0 2px 8px rgba(103, 194, 58, 0.3);
}

/* 价格标签美化 */
.price-tag {
  background: linear-gradient(135deg, #e6a23c, #f56c6c);
  color: white;
  padding: 4px 8px;
  border-radius: 12px;
  font-size: 13px;
  font-weight: 700;
  display: inline-block;
  box-shadow: 0 2px 8px rgba(230, 162, 60, 0.3);
}

/* 订购按钮美化 */
.OrderButton {
  background: linear-gradient(135deg, #409eff, #67c23a);
  border: none;
  border-radius: 20px;
  color: white;
  font-weight: 600;
  padding: 6px 16px;
  box-shadow: 0 4px 15px rgba(64, 158, 255, 0.3);
  transition: all 0.3s cubic-bezier(0.23, 1, 0.32, 1);
}

.OrderButton:hover {
  background: linear-gradient(135deg, #337ecc, #5daf34);
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(64, 158, 255, 0.4);
}

.OrderButton:disabled {
  background: #c0c4cc;
  box-shadow: none;
  transform: none;
}

/* 未找到结果美化 */
.HotelUnFind {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  height: 400px;
  color: #909399;
  text-align: center;
}

.UnfindImage {
  width: 200px;
  height: auto;
  margin-bottom: 20px;
  opacity: 0.8;
  filter: contrast(1.1) saturate(1.1);
}

.HotelUnFind p {
  font-size: 18px;
  font-weight: 500;
  color: #909399;
  margin: 0;
}

/* 响应式设计 */
@media (max-width: 1024px) {
  .search-container {
    flex-direction: column;
    align-items: stretch;
  }

  .train-input-section,
  .date-selection {
    max-width: none;
  }

  .search-button-wrapper {
    align-self: center;
  }

  .search-button {
    width: 200px;
  }
}

@media (max-width: 768px) {
  .meal-search {
    padding: 16px;
  }

  .search-card :deep(.el-card__body) {
    padding: 24px 20px;
  }

  .Grid {
    flex-direction: column;
  }

  .order-card-container {
    width: 100%;
  }
}
</style>

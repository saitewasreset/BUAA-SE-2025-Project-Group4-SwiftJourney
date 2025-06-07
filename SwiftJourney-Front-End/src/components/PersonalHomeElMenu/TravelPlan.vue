<template>
  <div class="travel-plan-container">
    <div class="travel-plan-card">
      <!-- 头部标题区域 -->
      <div class="card-header">
        <div class="header-content">
          <h1 class="page-title">我的行程</h1>
          <p class="page-subtitle">查看和管理您的旅行计划</p>
        </div>
        <!-- 添加统计信息 -->
        <div class="stats-info">
          <span class="stats-text">共 {{ filteredOrderList.length }} 个行程</span>
        </div>
      </div>

      <!-- 筛选器区域 -->
      <div class="filter-section">
        <div class="filter-grid">
          <div class="filter-group">
            <label class="filter-label">行程类型</label>
            <div class="checkbox-group">
              <el-checkbox v-model="selectedTrain" label="火车订票" />
              <el-checkbox v-model="selectedHotel" label="酒店预订" />
              <el-checkbox v-model="selectedFood" label="火车餐预订" />
            </div>
          </div>
          
          <div class="filter-group">
            <label class="filter-label">行程状态</label>
            <div class="checkbox-group">
              <el-checkbox v-model="showUpcoming" label="未出行" />
              <el-checkbox v-model="showCompleted" label="已出行" />
            </div>
          </div>
          
          <div class="filter-group">
            <label class="filter-label">时间范围</label>
            <a-range-picker 
                v-model:value="selectedDate" 
                start-placeholder="开始日期" 
                end-placeholder="结束日期" 
                @change="handleDateChange"
                class="date-picker"
                :locale="locale"
            />
          </div>
        </div>
      </div>

      <!-- 分割线 -->
      <div class="divider"></div>

      <!-- 行程列表内容 -->
      <div class="travel-content">
        <!-- 加载状态 -->
        <div v-if="loading" class="loading-state">
          <div class="skeleton-list">
            <div 
              v-for="n in pageSize" 
              :key="n" 
              class="skeleton-card"
              :style="{ animationDelay: `${(n - 1) * 0.1}s` }"
            >
              <!-- 骨架屏头部 -->
              <div class="skeleton-header">
                <div class="skeleton-info">
                  <div class="skeleton-type">
                    <div class="skeleton-icon"></div>
                    <div class="skeleton-text skeleton-type-text"></div>
                  </div>
                  <div class="skeleton-id">
                    <div class="skeleton-text skeleton-id-label"></div>
                    <div class="skeleton-text skeleton-id-value"></div>
                  </div>
                </div>
                <div class="skeleton-status">
                  <div class="skeleton-badge"></div>
                </div>
              </div>

              <!-- 骨架屏内容 -->
              <div class="skeleton-content">
                <div class="skeleton-main-info">
                  <div class="skeleton-text skeleton-title"></div>
                  <div class="skeleton-text skeleton-subtitle"></div>
                </div>
                <div class="skeleton-details">
                  <div class="skeleton-detail-item">
                    <div class="skeleton-text skeleton-detail-label"></div>
                    <div class="skeleton-text skeleton-detail-value"></div>
                  </div>
                  <div class="skeleton-detail-item">
                    <div class="skeleton-text skeleton-detail-label"></div>
                    <div class="skeleton-text skeleton-detail-value"></div>
                  </div>
                  <div class="skeleton-detail-item">
                    <div class="skeleton-text skeleton-detail-label"></div>
                    <div class="skeleton-text skeleton-detail-value"></div>
                  </div>
                  <div class="skeleton-detail-item">
                    <div class="skeleton-text skeleton-detail-label"></div>
                    <div class="skeleton-text skeleton-detail-value"></div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-else-if="!loading && filteredOrderList.length === 0" class="empty-state">
          <div class="empty-icon">
            <el-icon size="64"><Calendar /></el-icon>
          </div>
          <h3 class="empty-title">暂无行程安排</h3>
          <p class="empty-subtitle">当前筛选条件下没有找到相关行程</p>
        </div>

        <!-- 实际数据 -->
        <div v-else>
          <!-- 行程列表 -->
          <div class="trips-list">
            <div 
              v-for="(order, index) in paginatedOrderList" 
              :key="order.id"
              class="trip-item"
              :style="{ animationDelay: `${index * 0.1}s` }"
            >
              <TripCard :order="order" :isUpcoming="isUpcomingTrip(order)" @cancel-order="cancelOrder" />
            </div>
          </div>

          <!-- 分页组件 -->
          <div class="pagination-section" v-if="filteredOrderList.length > pageSize">
            <el-pagination
              v-model:current-page="currentPage"
              v-model:page-size="pageSize"
              :page-sizes="[5, 10, 20, 50]"
              :total="filteredOrderList.length"
              layout="total, sizes, prev, pager, next, jumper"
              @size-change="handlePageSizeChange"
              @current-change="handleCurrentPageChange"
              class="custom-pagination"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import dayjs from 'dayjs';
import 'dayjs/locale/zh-cn';
import { orderApi } from "@/api/orderApi/orderApi";
import type { 
  ResponseData, 
  TrainOrderInfo, 
  HotelOrderInfo, 
  DishOrderInfo, 
  TakeawayOrderInfo,
  SeatLocationInfo 
} from '@/interface/interface';
import { ElMessage, ElMessageBox } from 'element-plus';
import { useUserStore } from '@/stores/user';
import { userApi } from '@/api/UserApi/userApi';
import type { UserApiBalanceData } from '@/interface/userInterface';
import { Calendar } from '@element-plus/icons-vue';
import locale from 'ant-design-vue/es/date-picker/locale/zh_CN';
import type { Moment } from 'moment';
import TripCard from './TripCard.vue';

dayjs.locale('zh-cn');

// 状态映射
const statusChangeTab = {
  unpaid: "未支付",
  paid: "已支付", 
  ongoing: "未出行",
  active: "行程中",
  completed: "已完成",
  failed: "失败",
  cancelled: "已取消",
};

const typeChangeTab = {
  train: "火车订票",
  hotel: "酒店预订", 
  dish: "火车餐预订",
  takeaway: "火车餐预订",
};

// 响应式数据
const selectedTrain = ref(true);
const selectedHotel = ref(true);
const selectedFood = ref(true);
const showUpcoming = ref(true);
const showCompleted = ref(true);
const selectedDate = ref<Moment[]>([]);
const selectedStartDate = ref("");
const selectedEndDate = ref("");

const orderList = ref<OrderDetail[]>([]);
const currentPage = ref(1);
const pageSize = ref(10);
const loading = ref(true); // 添加加载状态

const user = useUserStore();

// 订单详情接口
interface OrderDetail {
  id: string;
  type: string;
  status: string;
  money: string;
  departureTime?: string;
  checkInTime?: string;
  canCanceled: boolean;
  reason: string;
  details: any;
}

// 计算属性
const filteredOrderList = computed(() => {
  return orderList.value.filter(order => {
    // 类型筛选
    const typeMatch = (
      (order.type === '火车订票' && selectedTrain.value) ||
      (order.type === '酒店预订' && selectedHotel.value) ||
      (order.type === '火车餐预订' && selectedFood.value)
    );

    // 状态筛选
    const isUpcoming = isUpcomingTrip(order);
    const statusMatch = (
      (isUpcoming && showUpcoming.value) ||
      (!isUpcoming && showCompleted.value)
    );

    // 时间筛选
    const timeMatch = !selectedDate.value || 
      selectedDate.value.length === 0 || 
      isDateInRange(order);

    return typeMatch && statusMatch && timeMatch;
  });
});

// 分页后的订单列表
const paginatedOrderList = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  const end = start + pageSize.value;
  return filteredOrderList.value.slice(start, end);
});

// 方法
const isUpcomingTrip = (order: OrderDetail): boolean => {
  const now = dayjs();
  const tripTime = dayjs(order.departureTime || order.checkInTime);
  return tripTime.isAfter(now) && order.status !== '已取消' && order.status !== '失败';
};

const isDateInRange = (order: OrderDetail): boolean => {
  if (!selectedStartDate.value || !selectedEndDate.value) return true;
  
  const orderTime = dayjs(order.departureTime || order.checkInTime);
  const start = dayjs(selectedStartDate.value);
  const end = dayjs(selectedEndDate.value);
  
  return orderTime.isBetween(start, end, 'day', '[]');
};

const handleDateChange = (dateRange: any) => {
  if (dateRange) {
    const [startDate, endDate] = selectedDate.value;
    selectedStartDate.value = dayjs(startDate.toDate()).format('YYYY-MM-DD');
    selectedEndDate.value = dayjs(endDate.toDate()).format('YYYY-MM-DD');
  } else {
    selectedDate.value = [];
    selectedStartDate.value = "";
    selectedEndDate.value = "";
  }
};

const cancelOrder = async (orderId: string, canCancel: boolean, reason: string) => {
  if (!canCancel) {
    ElMessage.error('不可取消该订单 ' + reason);
    return;
  }

  try {
    await ElMessageBox.confirm(
      '确认取消该订单？',
      '警告！',
      {
        confirmButtonText: '确认',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );

    const res = await orderApi.orderCancel(orderId);
    if (res.status === 200) {
      if (res.data.code === 200) {
        ElMessage.success('成功取消该订单');
        await updateBalance();
        await initOrderList();
      } else {
        handleApiError(res.data.code);
      }
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('取消订单失败');
    }
  }
};

const handleApiError = (code: number) => {
  const errorMessages = {
    403: '会话无效',
    404: '订单号不存在，或没有权限访问该订单',
    14001: '订单已被取消',
    14002: '订单不满足取消条件'
  };
  ElMessage.error(errorMessages[code] || '操作失败');
};

const updateBalance = async () => {
  try {
    const balRes: UserApiBalanceData = (await userApi.queryUserBalance()).data;
    if (balRes.code === 200) {
      user.setUserBalance(balRes.data.balance);
    }
  } catch (error) {
    console.error('更新余额失败:', error);
  }
};

const getSeat = (seat: SeatLocationInfo): string => {
  return (seat.carriage < 10 ? `0${seat.carriage}` : `${seat.carriage}`) + 
         "车" + seat.row + seat.location + " " + seat.type;
};

const initOrderList = async () => {
  try {
    loading.value = true; // 开始加载
    const res = await orderApi.orderList();
    if (res.status === 200 && res.data.code === 200) {
      const resData: ResponseData = res.data.data;
      console.log(resData);
      processOrderData(resData);
    } else {
      throw new Error(res.statusText);
    }
  } catch (error) {
    console.error('获取订单列表失败:', error);
    ElMessage.error('获取订单列表失败');
  } finally {
    loading.value = false; // 结束加载
  }
};

const processOrderData = (resData: ResponseData) => {
  const orders: OrderDetail[] = [];
  
  for (const transactionData of resData) {
    for (const orderInfo of transactionData.orders) {
      // 只处理已支付的订单
      if (orderInfo.status === 'unpaid' || orderInfo.status === 'cancelled') continue;

      const order: OrderDetail = {
        id: orderInfo.orderId,
        type: typeChangeTab[orderInfo.orderType],
        status: statusChangeTab[orderInfo.status],
        money: 'SC ' + String(orderInfo.unitPrice),
        canCanceled: orderInfo.canCancel,
        reason: orderInfo.reason || '',
        details: {}
      };

      // 处理不同类型的订单详情
      switch (orderInfo.orderType) {
        case "train":
          const trainOrder = orderInfo as TrainOrderInfo;
          order.departureTime = dayjs(trainOrder.departureTime).format("YYYY-MM-DD HH:mm");
          order.details = {
            trainNumber: trainOrder.trainNumber,
            departureStation: trainOrder.departureStation,
            terminalStation: trainOrder.terminalStation,
            seat: getSeat(trainOrder.seat),
            name: trainOrder.name,
            date: dayjs(trainOrder.departureTime).format("YYYY-MM-DD"),
            time: dayjs(trainOrder.departureTime).format("HH:mm")
          };
          break;

        case "hotel":
          const hotelOrder = orderInfo as HotelOrderInfo;
          order.checkInTime = dayjs(hotelOrder.beginDate).format("YYYY-MM-DD");
          order.details = {
            hotelName: hotelOrder.hotelName,
            roomType: hotelOrder.roomType,
            beginDate: dayjs(hotelOrder.beginDate).format("YYYY-MM-DD"),
            endDate: dayjs(hotelOrder.endDate).format("YYYY-MM-DD"),
            name: hotelOrder.name,
            amount: hotelOrder.amount
          };
          break;

        case "dish":
          const dishOrder = orderInfo as DishOrderInfo;
          order.departureTime = dayjs(dishOrder.depatureTime).format("YYYY-MM-DD HH:mm");
          order.details = {
            dishName: dishOrder.dishName,
            trainNumber: dishOrder.trainNumber,
            dishTime: dishOrder.dishTime === "dinner" ? "晚餐" : "午餐",
            name: dishOrder.name,
            date: dayjs(dishOrder.depatureTime).format("YYYY-MM-DD")
          };
          break;

        case "takeaway":
          const takeawayOrder = orderInfo as TakeawayOrderInfo;
          order.departureTime = dayjs(takeawayOrder.depatureTime).format("YYYY-MM-DD HH:mm");
          order.details = {
            takeawayName: takeawayOrder.takeawayName,
            shopName: takeawayOrder.shopName,
            trainNumber: takeawayOrder.trainNumber,
            station: takeawayOrder.station,
            dishTime: dayjs(takeawayOrder.dishTime).format("HH:mm"),
            name: takeawayOrder.name,
            date: dayjs(takeawayOrder.depatureTime).format("YYYY-MM-DD")
          };
          break;
      }

      orders.push(order);
    }
  }

  // 按时间排序
  orders.sort((a, b) => {
    const timeA = a.departureTime || a.checkInTime || '';
    const timeB = b.departureTime || b.checkInTime || '';
    return dayjs(timeB).valueOf() - dayjs(timeA).valueOf();
  });

  orderList.value = orders;
};

// 分页相关方法
const handlePageSizeChange = (newPageSize: number) => {
  pageSize.value = newPageSize;
  currentPage.value = 1;
};

const handleCurrentPageChange = (newCurrentPage: number) => {
  currentPage.value = newCurrentPage;
  scrollToTop();
};

const resetPagination = () => {
  currentPage.value = 1;
};

const scrollToTop = () => {
  document.querySelector('.travel-content')?.scrollIntoView({ 
    behavior: 'smooth', 
    block: 'start' 
  });
};

// 监听器
watch([selectedTrain, selectedHotel, selectedFood, showUpcoming, showCompleted, selectedDate], () => {
  resetPagination();
});

// 生命周期
onMounted(() => {
  initOrderList();
});
</script>

<style scoped>
/* 基础容器样式 */
.travel-plan-container {
  min-height: 50vh;
  padding: 16px;
  display: flex;
  justify-content: center;
}

.travel-plan-card {
  width: 100%;
  max-width: 1200px;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border-radius: 12px;
  box-shadow: 
    0 6px 20px rgba(0, 0, 0, 0.06),
    0 0 0 1px rgba(255, 255, 255, 0.2);
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* 头部样式 */
.card-header {
  padding: 20px 28px 14px;
  background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
}

.header-content {
  flex: 1;
}

.page-title {
  font-size: 26px;
  font-weight: 700;
  color: #1a202c;
  margin: 0 0 4px 0;
  letter-spacing: -0.2px;
  line-height: 1.2;
}

.page-subtitle {
  font-size: 16px;
  color: #64748b;
  margin: 0;
  font-weight: 400;
  line-height: 1.3;
}

.stats-info {
  display: flex;
  align-items: center;
}

.stats-text {
  font-size: 14px;
  color: #64748b;
  font-weight: 500;
  padding: 6px 12px;
  background: rgba(255, 255, 255, 0.8);
  border-radius: 6px;
  border: 1px solid rgba(226, 232, 240, 0.5);
  line-height: 1.2;
}

/* 筛选器样式 */
.filter-section {
  padding: 16px 28px;
  background: rgba(248, 250, 252, 0.5);
  border-bottom: 1px solid rgba(226, 232, 240, 0.5);
}

.filter-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
  align-items: start;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.filter-label {
  font-size: 15px;
  font-weight: 600;
  color: #374151;
  margin-bottom: 4px;
  line-height: 1.2;
}

.checkbox-group {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.checkbox-group :deep(.el-checkbox) {
  margin-right: 0;
  height: auto;
}

.checkbox-group :deep(.el-checkbox__label) {
  font-size: 14px;
  color: #4b5563;
  font-weight: 500;
  line-height: 1.3;
  padding-left: 8px;
}

.date-picker {
  width: 100%;
  max-width: 280px;
}

.date-picker :deep(.el-input__wrapper) {
  height: 36px;
  border-radius: 6px;
  transition: all 0.2s;
}

.divider {
  height: 1px;
  background: linear-gradient(90deg, transparent, #e2e8f0, transparent);
  margin: 0 28px;
}

/* 内容区域 */
.travel-content {
  padding: 20px 28px;
}

/* 加载状态 */
.loading-state {
  padding: 0;
}

.skeleton-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.skeleton-card {
  background: #fff;
  border-radius: 8px;
  border: 1px solid #f1f5f9;
  overflow: hidden;
  animation: fadeInUp 0.6s cubic-bezier(0.4, 0, 0.2, 1) forwards;
  opacity: 0;
}

.skeleton-header {
  padding: 16px 20px 12px;
  border-bottom: 1px solid #f1f5f9;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.skeleton-info {
  display: flex;
  align-items: center;
  gap: 24px;
}

.skeleton-type {
  display: flex;
  align-items: center;
  gap: 8px;
}

.skeleton-icon {
  width: 30px;
  height: 30px;
  background: #f1f5f9;
  border-radius: 6px;
  position: relative;
  overflow: hidden;
}

.skeleton-icon::after {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.6), transparent);
  animation: shimmer 1.5s infinite;
}

.skeleton-id {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.skeleton-text {
  background: #f1f5f9;
  border-radius: 4px;
  position: relative;
  overflow: hidden;
}

.skeleton-text::after {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.6), transparent);
  animation: shimmer 1.5s infinite;
}

.skeleton-type-text {
  width: 80px;
  height: 18px;
}

.skeleton-id-label {
  width: 60px;
  height: 12px;
}

.skeleton-id-value {
  width: 240px;
  height: 16px;
}

.skeleton-status {
  display: flex;
  align-items: center;
}

.skeleton-badge {
  width: 60px;
  height: 24px;
  background: #f1f5f9;
  border-radius: 6px;
  position: relative;
  overflow: hidden;
}

.skeleton-badge::after {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.6), transparent);
  animation: shimmer 1.5s infinite;
}

.skeleton-content {
  padding: 16px 20px;
}

.skeleton-main-info {
  margin-bottom: 16px;
  padding: 12px;
  background: rgba(248, 250, 252, 0.5);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton-title {
  width: 200px;
  height: 20px;
}

.skeleton-subtitle {
  width: 120px;
  height: 16px;
}

.skeleton-details {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 16px;
}

.skeleton-detail-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.skeleton-detail-label {
  width: 80px;
  height: 12px;
}

.skeleton-detail-value {
  width: 100px;
  height: 16px;
}

@keyframes shimmer {
  0% {
    left: -100%;
  }
  100% {
    left: 100%;
  }
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 40px 24px;
}

.empty-icon {
  color: #d1d5db;
  margin-bottom: 16px;
}

.empty-title {
  font-size: 22px;
  font-weight: 600;
  color: #374151;
  margin: 0 0 8px 0;
  line-height: 1.3;
}

.empty-subtitle {
  font-size: 16px;
  color: #6b7280;
  margin: 0;
  line-height: 1.4;
}

/* 行程列表 */
.trips-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.trip-item {
  animation: fadeInUp 0.6s cubic-bezier(0.4, 0, 0.2, 1) forwards;
  opacity: 0;
}

/* 分页样式 */
.pagination-section {
  margin-top: 20px;
  padding: 16px 0;
  border-top: 1px solid rgba(226, 232, 240, 0.3);
  display: flex;
  justify-content: center;
}

.custom-pagination {
  background: rgba(248, 250, 252, 0.5);
  padding: 12px 20px;
  border-radius: 8px;
  border: 1px solid rgba(226, 232, 240, 0.3);
}

.custom-pagination :deep(.el-pagination__total) {
  color: #64748b;
  font-weight: 500;
  font-size: 14px;
  line-height: 1.2;
}

.custom-pagination :deep(.btn-prev:hover),
.custom-pagination :deep(.btn-next:hover) {
  background: #667eea;
  border-color: #667eea;
  color: #fff;
}

.custom-pagination :deep(.el-pager li:hover) {
  background: rgba(102, 126, 234, 0.1);
  border-color: #667eea;
  color: #667eea;
}

.custom-pagination :deep(.el-pager li.is-active) {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-color: #667eea;
  color: #fff;
}

/* 动画 */
@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.trip-item:nth-child(1),
.skeleton-card:nth-child(1) { animation-delay: 0.05s; }
.trip-item:nth-child(2),
.skeleton-card:nth-child(2) { animation-delay: 0.1s; }
.trip-item:nth-child(3),
.skeleton-card:nth-child(3) { animation-delay: 0.15s; }
.trip-item:nth-child(4),
.skeleton-card:nth-child(4) { animation-delay: 0.2s; }
.trip-item:nth-child(5),
.skeleton-card:nth-child(5) { animation-delay: 0.25s; }

/* 响应式设计 */
@media (max-width: 768px) {
  .travel-plan-container {
    padding: 12px;
  }

  .card-header {
    padding: 16px 20px 12px;
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .page-title {
    font-size: 22px;
  }

  .page-subtitle {
    font-size: 14px;
  }

  .filter-section {
    padding: 12px 20px;
  }

  .filter-grid {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .travel-content {
    padding: 16px 20px;
  }

  .skeleton-info {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .skeleton-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .skeleton-details {
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }
}
</style>
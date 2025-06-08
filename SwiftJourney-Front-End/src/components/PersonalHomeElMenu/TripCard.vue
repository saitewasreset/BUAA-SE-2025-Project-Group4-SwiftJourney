<template>
  <div class="trip-card-wrapper" :class="{ 'upcoming': isUpcoming, 'completed': !isUpcoming }">
    <!-- 卡片头部 -->
    <div class="trip-header">
      <div class="trip-info">
        <div class="trip-type">
          <el-icon class="type-icon">
            <House v-if="order.type === '火车订票'" />
            <House v-else-if="order.type === '酒店预订'" />
            <Bowl v-else />
          </el-icon>
          <span class="type-text">{{ order.type }}</span>
        </div>
        <div class="trip-id">
          <span class="id-label">订单编号</span>
          <span class="id-value">{{ order.id }}</span>
        </div>
      </div>
      <div class="trip-status">
        <span class="status-badge" :class="getStatusClass(order.status)">
          {{ order.status }}
        </span>
      </div>
    </div>

    <!-- 行程内容 -->
    <div class="trip-content">
      <!-- 火车订票 -->
      <div v-if="order.type === '火车订票'" class="train-trip">
        <div class="route-info">
          <div class="station departure">
            <div class="station-name">{{ order.details.departureStation }}</div>
            <div class="station-time">{{ order.details.time }} 出发</div>
          </div>
          <div class="route-line">
            <div class="train-number">{{ order.details.trainNumber }}</div>
            <div class="line"></div>
          </div>
          <div class="station arrival">
            <div class="station-name">{{ order.details.terminalStation }}</div>
            <div class="station-time">预计到达</div>
          </div>
        </div>
        <div class="trip-details">
          <div class="detail-item">
            <span class="detail-label">出行日期</span>
            <span class="detail-value">{{ order.details.date }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">座位信息</span>
            <span class="detail-value">{{ order.details.seat }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">乘车人</span>
            <span class="detail-value">{{ order.details.name }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">订单金额</span>
            <span class="detail-value amount">{{ order.money }}</span>
          </div>
        </div>
      </div>

      <!-- 酒店预订 -->
      <div v-else-if="order.type === '酒店预订'" class="hotel-trip">
        <div class="hotel-main-info">
          <div class="hotel-info">
            <div class="hotel-name">{{ order.details.hotelName }}</div>
            <div class="room-type">{{ order.details.roomType }}</div>
          </div>
          <div class="date-range">
            <div class="check-in">
              <span class="date-label">入住</span>
              <span class="date-value">{{ order.details.beginDate }}</span>
            </div>
            <div class="stay-duration">
              <el-icon><Right /></el-icon>
            </div>
            <div class="check-out">
              <span class="date-label">离店</span>
              <span class="date-value">{{ order.details.endDate }}</span>
            </div>
          </div>
        </div>
        <div class="trip-details">
          <div class="detail-item">
            <span class="detail-label">房间数量</span>
            <span class="detail-value">{{ order.details.amount }} 间</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">预订人</span>
            <span class="detail-value">{{ order.details.name }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">订单金额</span>
            <span class="detail-value amount">{{ order.money }}</span>
          </div>
        </div>
      </div>

      <!-- 火车餐预订 -->
      <div v-else class="food-trip">
        <div class="food-main-info">
          <div class="food-info">
            <div class="dish-name">{{ order.details.dishName || order.details.takeawayName }}</div>
            <div class="shop-name">{{ order.details.shopName || '餐车' }}</div>
          </div>
          <div class="train-info">
            <div class="info-item">
              <span class="info-label">车次</span>
              <span class="info-value">{{ order.details.trainNumber }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">出行日期</span>
              <span class="info-value">{{ order.details.date }}</span>
            </div>
          </div>
        </div>
        <div class="trip-details">
          <div class="detail-item">
            <span class="detail-label">用餐时间</span>
            <span class="detail-value">{{ order.details.dishTime }}</span>
          </div>
          <div class="detail-item" v-if="order.details.station">
            <span class="detail-label">车站</span>
            <span class="detail-value">{{ order.details.station }}站</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">订餐人</span>
            <span class="detail-value">{{ order.details.name }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">订单金额</span>
            <span class="detail-value amount">{{ order.money }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 卡片底部操作 -->
    <div class="trip-footer" v-if="isUpcoming && order.canCanceled">
      <el-button 
        text 
        type="danger" 
        size="small"
        @click="handleCancel"
        class="cancel-btn"
      >
        取消订单
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { House, Bowl, Right } from '@element-plus/icons-vue';

// Props
interface Props {
  order: any;
  isUpcoming: boolean;
}

const props = defineProps<Props>();

// Emits
const emit = defineEmits<{
  cancelOrder: [orderId: string, canCancel: boolean, reason: string];
}>();

// Methods
const getStatusClass = (status: string) => {
  const statusClasses: { [key: string]: string } = {
    '未出行': 'status-upcoming',
    '行程中': 'status-active',
    '已完成': 'status-completed',
    '已取消': 'status-cancelled',
    '失败': 'status-failed'
  };
  return statusClasses[status] || 'status-default';
};

const handleCancel = () => {
  emit('cancelOrder', props.order.id, props.order.canCanceled, props.order.reason);
};
</script>

<style scoped>
.trip-card-wrapper {
  width: 100%;
  background: #fff;
  border-radius: 12px;
  border: 1px solid #f1f5f9;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
}

.trip-card-wrapper::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  transition: transform 0.3s;
  transform: scaleX(0);
  transform-origin: left;
}

.trip-card-wrapper.upcoming::before {
  background: linear-gradient(135deg, #3b82f6 0%, #1d4ed8 100%);
}

.trip-card-wrapper.completed::before {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
}

.trip-card-wrapper:hover {
  border-color: #e2e8f0;
  transform: translateY(-1px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}

.trip-card-wrapper:hover::before {
  transform: scaleX(1);
}

/* 头部 */
.trip-header {
  padding: 16px 20px 12px;
  border-bottom: 1px solid #f1f5f9;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.trip-info {
  display: flex;
  align-items: center;
  gap: 24px;
}

.trip-type {
  display: flex;
  align-items: center;
  gap: 8px;
}

.type-icon {
  font-size: 18px;
  color: #667eea;
  padding: 6px;
  background: rgba(102, 126, 234, 0.1);
  border-radius: 6px;
}

.type-text {
  font-size: 15px;
  font-weight: 600;
  color: #374151;
}

.trip-id {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.id-label {
  font-size: 12px;
  color: #6b7280;
  font-weight: 500;
}

.id-value {
  font-size: 14px;
  color: #1a202c;
  font-weight: 600;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
}

.status-badge {
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.status-upcoming {
  background: #dbeafe;
  color: #1d4ed8;
  border: 1px solid #3b82f6;
}

.status-active {
  background: #d1fae5;
  color: #047857;
  border: 1px solid #10b981;
}

.status-completed {
  background: #f3f4f6;
  color: #6b7280;
  border: 1px solid #9ca3af;
}

.status-cancelled {
  background: #fef2f2;
  color: #b91c1c;
  border: 1px solid #f87171;
}

/* 内容区域 */
.trip-content {
  padding: 16px 20px;
}

/* 火车行程 */
.route-info {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  gap: 20px;
  align-items: center;
  margin-bottom: 16px;
  padding: 12px;
  background: rgba(59, 130, 246, 0.05);
  border-radius: 8px;
}

.station {
  text-align: center;
}

.station-name {
  font-size: 18px;
  font-weight: 700;
  color: #1a202c;
  margin-bottom: 4px;
}

.station-time {
  font-size: 13px;
  color: #6b7280;
  font-weight: 500;
}

.route-line {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.train-number {
  padding: 6px 12px;
  background: linear-gradient(135deg, #3b82f6 0%, #1d4ed8 100%);
  color: white;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
}

.line {
  width: 60px;
  height: 2px;
  background: linear-gradient(90deg, #3b82f6, #8b5cf6);
  border-radius: 1px;
  position: relative;
}

.line::after {
  content: '→';
  position: absolute;
  right: -8px;
  top: -6px;
  color: #8b5cf6;
  font-size: 14px;
  font-weight: bold;
}

/* 酒店行程 */
.hotel-main-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding: 12px;
  background: rgba(16, 185, 129, 0.05);
  border-radius: 8px;
}

.hotel-info {
  flex: 1;
}

.hotel-name {
  font-size: 18px;
  font-weight: 700;
  color: #1a202c;
  margin-bottom: 4px;
}

.room-type {
  font-size: 14px;
  color: #6b7280;
  font-weight: 500;
}

.date-range {
  display: flex;
  align-items: center;
  gap: 16px;
}

.check-in,
.check-out {
  text-align: center;
}

.date-label {
  display: block;
  font-size: 12px;
  color: #6b7280;
  font-weight: 500;
  margin-bottom: 4px;
}

.date-value {
  font-size: 14px;
  font-weight: 600;
  color: #374151;
}

.stay-duration {
  color: #10b981;
  font-size: 18px;
}

/* 餐食行程 */
.food-main-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding: 12px;
  background: rgba(245, 158, 11, 0.05);
  border-radius: 8px;
}

.food-info {
  flex: 1;
}

.dish-name {
  font-size: 18px;
  font-weight: 700;
  color: #1a202c;
  margin-bottom: 4px;
}

.shop-name {
  font-size: 14px;
  color: #6b7280;
  font-weight: 500;
}

.train-info {
  display: flex;
  gap: 20px;
}

.info-item {
  text-align: center;
}

.info-label {
  display: block;
  font-size: 12px;
  color: #6b7280;
  font-weight: 500;
  margin-bottom: 4px;
}

.info-value {
  font-size: 14px;
  font-weight: 600;
  color: #374151;
}

/* 通用详情 */
.trip-details {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 16px;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.detail-label {
  font-size: 12px;
  color: #6b7280;
  font-weight: 500;
}

.detail-value {
  font-size: 14px;
  color: #374151;
  font-weight: 600;
}

.detail-value.amount {
  font-size: 16px;
  font-weight: 700;
  color: #667eea;
}

/* 底部操作 */
.trip-footer {
  padding: 12px 20px;
  border-top: 1px solid #f1f5f9;
  background: rgba(248, 250, 252, 0.5);
  display: flex;
  justify-content: flex-end;
}

.cancel-btn {
  font-size: 12px;
  padding: 6px 12px;
  border-radius: 6px;
}

/* 已完成行程的特殊样式 */
.trip-card-wrapper.completed {
  background: rgba(248, 250, 252, 0.5);
}

.trip-card-wrapper.completed .trip-content {
  opacity: 0.85;
}

.trip-card-wrapper.completed .detail-value.amount {
  color: #6b7280;
}

/* 响应式 */
@media (max-width: 768px) {
  .trip-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .route-info {
    grid-template-columns: 1fr;
    gap: 12px;
    text-align: center;
  }

  .hotel-main-info,
  .food-main-info {
    flex-direction: column;
    gap: 12px;
    align-items: flex-start;
  }

  .date-range {
    align-self: stretch;
    justify-content: space-around;
  }

  .trip-details {
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }

  .train-info {
    flex-direction: column;
    gap: 8px;
  }
}
</style>
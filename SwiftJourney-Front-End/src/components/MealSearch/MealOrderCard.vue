<template>
  <el-card class="order-info-card">
    <!-- 标题区域 -->
    <div class="card-header">
      <h3 class="card-title">已选餐品</h3>
      <div class="order-summary">
        <span class="item-count">{{ mealOrderStore.mealOrderInfoList.length }}项</span>
        <span class="total-amount">合计: SC {{ totalMoney }}</span>
      </div>
    </div>

    <!-- 订单列表区域 -->
    <div v-if="mealOrderStore.mealOrderInfoList.length === 0" class="empty-state">
      <div class="empty-icon">🍽️</div>
      <p class="empty-text">您还没有选择任何餐品</p>
      <p class="empty-hint">选择心仪的餐食开始您的美味之旅</p>
    </div>
    <div v-else class="order-items">
      <div
        v-for="(item, index) in mealOrderStore.mealOrderInfoList"
        :key="index"
        class="order-item-container"
        @mouseenter="showDeleteIcon(index)"
        @mouseleave="hideDeleteIcon(index)"
      >
        <div class="order-item-card">
          <!-- 餐品信息 -->
          <div class="item-info">
            <div class="shop-badge">{{ item.shopName }}</div>
            <h4 class="item-name">{{ item.name }}</h4>
            <div class="item-details">
              <span v-if="item.dishTime" class="meal-time">
                {{ lunchChangeTab[item.dishTime] }}
              </span>
              <span class="item-price">SC {{ item.price }}/份</span>
            </div>
          </div>

          <!-- 数量控制 -->
          <div class="quantity-control">
            <el-input-number
              v-model="item.amount"
              :min="1"
              :max="10"
              size="small"
              class="quantity-input"
            />
          </div>

          <!-- 总价显示 -->
          <div class="item-total">
            <span class="total-price">SC{{ item.amount * item.price }}</span>
          </div>

          <!-- 删除按钮 -->
          <div
            v-if="deleteIconsVisible[index]"
            class="delete-button"
            @click="deleteRoomFromOrder(item.shopName, item.name, item.dishTime)"
          >
            <el-icon class="delete-icon">
              <CircleCloseFilled />
            </el-icon>
          </div>
        </div>
      </div>
    </div>

    <!-- 底部操作区域 -->
    <div class="card-footer">
      <div class="footer-summary">
        <div class="total-info">
          <span class="total-label">订单总计:</span>
          <span class="total-value">SC{{ totalMoney }}</span>
        </div>
      </div>
      <el-button
        class="order-submit-button"
        type="primary"
        size="large"
        :disabled="mealOrderStore.mealOrderInfoList.length === 0"
        @click="createTransaction"
      >
        <template #icon>
          <el-icon><ShoppingCartFull /></el-icon>
        </template>
        生成订单
      </el-button>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useMealOrderStore } from '@/stores/mealOrder'
import { useRouter } from 'vue-router'
const mealOrderStore = useMealOrderStore()

const lunchChangeTab = {
  lunch: '午餐',
  dinner: '晚餐',
}

const totalMoney = computed(() => {
  let sum = 0
  mealOrderStore.mealOrderInfoList.forEach((key) => {
    sum += key.amount * key.price
  })
  return sum
})

const deleteIconsVisible = ref(mealOrderStore.mealOrderInfoList.map(() => false))

function showDeleteIcon(index: number) {
  deleteIconsVisible.value[index] = true
}

function hideDeleteIcon(index: number) {
  deleteIconsVisible.value[index] = false
}

function deleteRoomFromOrder(shopName: string, foodName: string, time?: 'lunch' | 'dinner') {
  ElMessageBox.confirm('是否取消选择' + shopName + '的' + foodName, '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning',
  }).then(() => {
    mealOrderStore.delete(shopName, foodName, time)
    ElMessage.success('成功取消选择' + shopName + '的' + foodName)
  })
}

//---------------------------------生成订单-----------------------------------
import { useUserStore } from '@/stores/user'

const nowUser = useUserStore()

function createTransaction() {
  ElMessageBox.confirm(
    '您选择的餐品总价为 SC' + totalMoney.value + '，核对无误后请点击确定',
    '确认生成订单',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    },
  ).then(() => {
    confirmCreateTransaction()
  })
}

import type { TrainDishOrderRequest, TakeawayOrder, DishOrder } from '@/interface/mealInterface'
import { mealApi } from '@/api/MealApi/mealApi'
import type { TransactionInfo } from '@/interface/interface'

async function confirmCreateTransaction() {
  const trainDishOrderRequest: TrainDishOrderRequest = {
    trainNumber: mealOrderStore.trainNumber,
    originDepartureTime: mealOrderStore.originDepartureTime,
    takeaway: [],
    dishes: [],
  }

  mealOrderStore.mealOrderInfoList.forEach((value: any) => {
    if (value.shopName == '餐车') {
      const tepInfo: DishOrder = {
        name: value.name,
        personalId: value.personalId,
        amount: value.amount,
        dishTime: value.dishTime as 'lunch' | 'dinner',
      }
      trainDishOrderRequest.dishes.push(tepInfo)
    } else {
      const tepInfo: TakeawayOrder = {
        station: value.station as string,
        shopName: value.shopName,
        name: value.name,
        personalId: value.personalId,
        amount: value.amount,
      }
      trainDishOrderRequest.takeaway.push(tepInfo)
    }
  })

  await mealApi
    .dishOrder(trainDishOrderRequest)
    .then((res: any) => {
      if (res.status == 200) {
        if (res.data.code == 200) {
          successCreateTransaction(res.data.data as TransactionInfo)
        } else if (res.data.code == 22006) {
          ElMessage.error(
            '没有对应的车次订单/对应的车次订单未支付/对应的车次订单已完成（失败/已取消）',
          )
        } else {
          throw new Error(res.data.message)
        }
      }
    })
    .catch((error: any) => {
      ElMessage.error('生成订单失败 ' + error)
    })
}

function successCreateTransaction(transactionInfo: TransactionInfo) {
  mealOrderStore.deleteAll()
  ElMessageBox.confirm(
    '您的订单号为 ' +
      transactionInfo.transactionId +
      ' ,总价 SC' +
      transactionInfo.amount +
      '，可在订单系统中查看具体信息，是否立即支付',
    '生成订单成功',
    {
      confirmButtonText: '立即支付',
      cancelButtonText: '稍后支付',
      type: 'success',
    },
  ).then(() => {
    //处理支付逻辑
    goToPay(transactionInfo.transactionId, 'SC ' + transactionInfo.amount)
  })
}

const router = useRouter()

function goToPay(transactionId: string, money: string) {
  router.push({
    name: 'paypage',
    params: { transactionId: transactionId },
    query: {
      money: money,
    },
  })
}

import type { ScrollbarInstance } from 'element-plus'
import { watch, nextTick } from 'vue'
// 滚动条组件的引用
const innerRef = ref<HTMLDivElement>()
const scrollbar = ref<ScrollbarInstance>()
// 获取 hotelOrderInfoList
const hotelOrderInfoListLength = computed(() => mealOrderStore.mealOrderInfoList.length);
// 监听 hotelOrderInfoList 的长度变化
watch(hotelOrderInfoListLength, (newLength: number, oldLength: number) => {
    if (newLength > oldLength) {
        nextTick(() => {
            scrollbar.value!.scrollTo({ top: innerRef.value!.clientHeight, behavior: 'smooth' });
        });
    }
});
</script>

<style scoped>
.order-info-card {
  width: 300px;
  border-radius: 16px;
  border: none;
  background: linear-gradient(135deg, #ffffff 0%, #f8f9ff 100%);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
  transition: all 0.3s ease;
  overflow: hidden;
}

.order-info-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.12);
}

.order-info-card :deep(.el-card__body) {
  padding: 0;
  height: 600px;
  display: flex;
  flex-direction: column;
}

/* 卡片标题区域 */
.card-header {
  padding: 20px 20px 16px;
  background: linear-gradient(135deg, #409eff, #67c23a);
  color: white;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.card-title {
  font-size: 18px;
  font-weight: 700;
  margin: 0 0 8px 0;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.order-summary {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 14px;
  opacity: 0.9;
}

.item-count {
  background: rgba(255, 255, 255, 0.2);
  padding: 2px 8px;
  border-radius: 12px;
  font-weight: 500;
}

.total-amount {
  font-weight: 600;
  font-size: 15px;
}

/* 订单列表容器 */
.order-list-container {
  flex-grow: 1;
  padding: 16px 20px 0;
  background: rgba(248, 250, 252, 0.5);
  overflow: auto;
}

.order-list-container :deep(.el-scrollbar__view) {
  padding-bottom: 16px;
}

/* 空状态样式 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 280px;
  color: #909399;
  text-align: center;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.6;
}

.empty-text {
  font-size: 16px;
  font-weight: 500;
  margin: 0 0 8px 0;
  color: #606266;
}

.empty-hint {
  font-size: 14px;
  margin: 0;
  color: #909399;
  line-height: 1.4;
}

/* 订单项目样式 */
.order-items {
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow: auto;
  padding: 1rem;
  gap: 0.5em;
}

.order-item-container {
  position: relative;
}

.order-item-card {
  background: white;
  border-radius: 12px;
  padding: 16px;
  border: 2px solid rgba(64, 158, 255, 0.1);
  transition: all 0.3s ease;
  position: relative;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.order-item-card:hover {
  border-color: #409eff;
  box-shadow: 0 4px 16px rgba(64, 158, 255, 0.15);
  transform: translateY(-1px);
}

/* 餐品信息区域 */
.item-info {
  margin-bottom: 12px;
}

.shop-badge {
  display: inline-block;
  background: linear-gradient(135deg, #409eff, #67c23a);
  color: white;
  font-size: 12px;
  font-weight: 600;
  padding: 4px 8px;
  border-radius: 8px;
  margin-bottom: 8px;
  box-shadow: 0 2px 6px rgba(64, 158, 255, 0.3);
}

.item-name {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 8px 0;
  line-height: 1.4;
}

.item-details {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.meal-time {
  background: rgba(103, 194, 58, 0.1);
  color: #67c23a;
  font-size: 12px;
  font-weight: 500;
  padding: 2px 6px;
  border-radius: 6px;
  border: 1px solid rgba(103, 194, 58, 0.2);
}

.item-price {
  color: #909399;
  font-size: 13px;
  font-weight: 500;
}

/* 数量控制和总价区域 */
.quantity-control {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.quantity-input {
  width: 120px;
}

.quantity-input :deep(.el-input-number__decrease),
.quantity-input :deep(.el-input-number__increase) {
  border-color: #409eff;
  color: #409eff;
}

.quantity-input :deep(.el-input__inner) {
  text-align: center;
  font-weight: 600;
  border-color: rgba(64, 158, 255, 0.3);
}

.item-total {
  text-align: right;
}

.total-price {
  font-size: 16px;
  font-weight: 700;
  color: #409eff;
  background: linear-gradient(135deg, #409eff, #67c23a);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

/* 删除按钮 */
.delete-button {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: rgba(245, 108, 108, 0.1);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.3s ease;
  opacity: 0.8;
}

.delete-button:hover {
  background: rgba(245, 108, 108, 0.2);
  transform: scale(1.1);
  opacity: 1;
}

.delete-icon {
  color: #f56c6c;
  font-size: 16px;
}

/* 底部操作区域 */
.card-footer {
  padding: 16px 20px 20px;
  background: rgba(248, 250, 252, 0.8);
  border-top: 1px solid rgba(64, 158, 255, 0.1);
  flex-shrink: 0;
}

.footer-summary {
  margin-bottom: 16px;
}

.total-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background: white;
  border-radius: 8px;
  border: 2px solid rgba(64, 158, 255, 0.1);
}

.total-label {
  font-size: 14px;
  color: #606266;
  font-weight: 500;
}

.total-value {
  font-size: 18px;
  font-weight: 700;
  background: linear-gradient(135deg, #409eff, #67c23a);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

/* 提交按钮 */
.order-submit-button {
  width: 100%;
  height: 48px;
  font-size: 16px;
  font-weight: 600;
  background: linear-gradient(135deg, #409eff 0%, #67c23a 100%);
  border: none;
  border-radius: 12px;
  box-shadow: 0 4px 16px rgba(64, 158, 255, 0.3);
  transition: all 0.3s ease;
}

.order-submit-button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(64, 158, 255, 0.4);
  background: linear-gradient(135deg, #337ecc 0%, #5daf34 100%);
}

.order-submit-button:disabled {
  background: #c0c4cc;
  box-shadow: none;
  transform: none;
  cursor: not-allowed;
}

.order-submit-button :deep(.el-icon) {
  margin-right: 8px;
  font-size: 18px;
}

/* 滚动条美化 */
.order-list-container :deep(.el-scrollbar__bar) {
  opacity: 0.3;
  transition: opacity 0.3s ease;
}

.order-list-container :deep(.el-scrollbar__bar):hover,
.order-list-container:hover :deep(.el-scrollbar__bar) {
  opacity: 0.6;
}

.order-list-container :deep(.el-scrollbar__thumb) {
  background: linear-gradient(135deg, #409eff, #67c23a);
  border-radius: 6px;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .order-info-card {
    width: 100%;
    margin-bottom: 20px;
    display: flex;
    flex-direction: column;
  }

  .order-item-card {
    padding: 12px;
  }

  .item-details {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }

  .quantity-control {
    flex-direction: column;
    gap: 8px;
    align-items: stretch;
  }

  .quantity-input {
    width: 100%;
  }
}

/* 动画效果 */
@keyframes slideInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.order-item-card {
  animation: slideInUp 0.3s ease forwards;
}

/* 加载状态 */
.order-info-card.loading {
  pointer-events: none;
}

.order-info-card.loading::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(2px);
  z-index: 10;
}
</style>

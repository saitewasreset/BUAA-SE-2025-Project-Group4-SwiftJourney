<template>
  <div class="train-transaction">
    <div class="transaction-container">
      <!-- 车次信息卡片 -->
      <div class="schedule-card">
        <!-- 顶部信息区 -->
        <div class="card-header">
          <div class="header-content">
            <h1 class="page-title">火车票预订</h1>
            <p class="page-subtitle">确认车次信息，完成您的出行预订</p>
          </div>
          <div class="header-actions">
            <div class="query-type-badge" :class="ticketServiceStore.queryMode">
              {{ ticketServiceStore.queryMode === 'direct' ? '直达' : '中转' }}
            </div>
          </div>
        </div>

        <!-- 分隔线 -->
        <div class="divider"></div>

        <!-- 直达车次信息 -->
        <div v-if="ticketServiceStore.queryMode === 'direct'" class="content-layout">
          <div class="route-card">
            <div class="route-content">
              <div class="segment-label">{{ formatDate(ticketServiceStore.queryDate) }}</div>

              <!-- 车次信息和座位信息并排显示 -->
              <div class="route-and-seat-container">
                <!-- 左侧：车次路线信息 -->
                <div class="route-content-section">
                  <div class="station-block departure">
                    <div class="time-medium">
                      {{ formatTime(directPreOrderSchedule.departureTime) }}
                    </div>
                    <div class="station-name">{{ directPreOrderSchedule.departureStation }}</div>
                  </div>

                  <div class="journey-middle">
                    <div class="train-duration-row">
                      <!-- 车次信息气泡 -->
                      <a-popover
                        :title="directPreOrderSchedule.trainNumber + ' 车次信息'"
                        trigger="click"
                      >
                        <template #content>
                          <a-table
                            size="small"
                            :columns="columns"
                            :data-source="processedDirectRoute"
                            :pagination="false"
                            :customRow="handleCustomRow"
                            :scroll="{ y: 200 }"
                            style="width: 400px"
                          />
                        </template>
                        <div
                          class="train-number-badge"
                          :class="{ small: directPreOrderSchedule.trainNumber.length > 5 }"
                        >
                          {{ directPreOrderSchedule.trainNumber }}
                        </div>
                      </a-popover>
                      <div class="duration-text">
                        {{ formatTravelTime(directPreOrderSchedule.travelTime) }}
                      </div>
                    </div>
                    <div class="journey-arrow">
                      <div class="arrow-line"></div>
                    </div>
                  </div>

                  <div class="station-block arrival">
                    <div>
                      <div class="time-medium">
                        {{ formatTime(directPreOrderSchedule.arrivalTime) }}
                      </div>
                      <div class="station-name">{{ directPreOrderSchedule.arrivalStation }}</div>
                    </div>
                    <div>
                      <div v-if="directOverDateFlag" class="over-date-badge">
                        +{{ directOverDateNum }}
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <!-- 右侧：座位类型信息 -->
            <div class="seat-types-section-inline">
              <div class="seat-types-title-inline">座位类型</div>
              <div class="seat-types-grid-inline">
                <div
                  v-for="(seatInfo, index) in sortedDirectSeatInfo"
                  :key="index"
                  class="seat-type-item-inline"
                >
                  <div class="seat-type-name">{{ seatInfo.seatType }}</div>
                  <div class="seat-type-info">
                    <div
                      class="seat-availability"
                      :class="{
                        rich: getleftType(seatInfo.left) === 'rich',
                        few: getleftType(seatInfo.left) === 'few',
                        little: getleftType(seatInfo.left) === 'little',
                        none: getleftType(seatInfo.left) === 'none',
                      }"
                    >
                      {{ formatleft(getleftType(seatInfo.left), seatInfo.left) }}
                    </div>
                    <div class="seat-price">SC {{ seatInfo.price }}</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 中转车次信息 -->
        <div v-else class="content-layout">
          <div class="transfer-info">
            <!-- 第一程 -->
            <div class="route-card transfer">
              <div class="route-content transfer">
                <div class="segment-label">
                  第1程 {{ formatDate(ticketServiceStore.queryDate) }}
                </div>

                <!-- 车次信息和座位信息并排显示 -->
                <div class="route-and-seat-container">
                  <!-- 左侧：车次路线信息 -->
                  <div class="route-content-section">
                    <div class="station-block departure">
                      <div class="time-medium">
                        {{ formatTime(indirectPreOrderSchedule.first_ride.departureTime) }}
                      </div>
                      <div class="station-name">
                        {{ indirectPreOrderSchedule.first_ride.departureStation }}
                      </div>
                    </div>

                    <div class="journey-middle">
                      <div class="train-duration-row">
                        <!-- 车次信息气泡 -->
                        <a-popover
                          :title="indirectPreOrderSchedule.first_ride.trainNumber + ' 车次信息'"
                          trigger="click"
                        >
                          <template #content>
                            <a-table
                              size="small"
                              :columns="columns"
                              :data-source="processedFirstRideRoute"
                              :pagination="false"
                              :customRow="handleCustomRow"
                              :scroll="{ y: 200 }"
                              style="width: 400px"
                            />
                          </template>
                          <div
                            class="train-number-badge"
                            :class="{
                              small: indirectPreOrderSchedule.first_ride.trainNumber.length > 5,
                            }"
                          >
                            {{ indirectPreOrderSchedule.first_ride.trainNumber }}
                          </div>
                        </a-popover>
                        <div class="duration-text">
                          {{ formatTravelTime(indirectPreOrderSchedule.first_ride.travelTime) }}
                        </div>
                      </div>
                      <div class="journey-arrow">
                        <div class="arrow-line"></div>
                      </div>
                    </div>

                    <div class="station-block transfer">
                      <div class="time-medium">
                        {{ formatTime(indirectPreOrderSchedule.first_ride.arrivalTime) }}
                      </div>
                      <div class="station-name">
                        {{ indirectPreOrderSchedule.first_ride.arrivalStation }}
                      </div>
                    </div>
                  </div>
                </div>
              </div>
              <!-- 右侧：座位类型信息 -->
              <div class="seat-types-section-inline">
                <div class="seat-types-title-inline">第1程座位类型</div>
                <div class="seat-types-grid-inline">
                  <div
                    v-for="(seatInfo, index) in sortedFirstRideSeatInfo"
                    :key="index"
                    class="seat-type-item-inline"
                  >
                    <div class="seat-type-name">{{ seatInfo.seatType }}</div>
                    <div class="seat-type-info">
                      <div
                        class="seat-availability"
                        :class="{
                          rich: getleftType(seatInfo.left) === 'rich',
                          few: getleftType(seatInfo.left) === 'few',
                          little: getleftType(seatInfo.left) === 'little',
                          none: getleftType(seatInfo.left) === 'none',
                        }"
                      >
                        {{ formatleft(getleftType(seatInfo.left), seatInfo.left) }}
                      </div>
                      <div class="seat-price">¥{{ seatInfo.price }}</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <!-- 第二程 -->
            <div class="route-card transfer">
              <div class="route-content transfer">
                <div class="segment-label">
                  第2程 {{ formatNextDate(indirectPreOrderSchedule.second_ride.departureTime) }}
                </div>

                <!-- 车次信息和座位信息并排显示 -->
                <div class="route-and-seat-container">
                  <!-- 左侧：车次路线信息 -->
                  <div class="route-content-section">
                    <div class="station-block departure">
                      <div class="time-medium">
                        {{ formatTime(indirectPreOrderSchedule.second_ride.departureTime) }}
                      </div>
                      <div class="station-name">
                        {{ indirectPreOrderSchedule.second_ride.departureStation }}
                      </div>
                    </div>

                    <div class="journey-middle">
                      <div class="train-duration-row">
                        <!-- 车次信息气泡 -->
                        <a-popover
                          :title="indirectPreOrderSchedule.second_ride.trainNumber + ' 车次信息'"
                          trigger="click"
                        >
                          <template #content>
                            <a-table
                              size="small"
                              :columns="columns"
                              :data-source="processedSecondRideRoute"
                              :pagination="false"
                              :customRow="handleCustomRow"
                              :scroll="{ y: 200 }"
                              style="width: 400px"
                            />
                          </template>
                          <div
                            class="train-number-badge"
                            :class="{
                              small: indirectPreOrderSchedule.second_ride.trainNumber.length > 5,
                            }"
                          >
                            {{ indirectPreOrderSchedule.second_ride.trainNumber }}
                          </div>
                        </a-popover>
                        <div class="duration-text">
                          {{ formatTravelTime(indirectPreOrderSchedule.second_ride.travelTime) }}
                        </div>
                      </div>
                      <div class="journey-arrow">
                        <div class="arrow-line"></div>
                      </div>
                    </div>

                    <div class="station-block arrival">
                      <div>
                        <div class="time-medium">
                          {{ formatTime(indirectPreOrderSchedule.second_ride.arrivalTime) }}
                        </div>
                        <div class="station-name">
                          {{ indirectPreOrderSchedule.second_ride.arrivalStation }}
                        </div>
                      </div>
                      <div>
                        <div v-if="indirectOverDateFlag" class="over-date-badge">
                          +{{ indirectOverDateNum }}
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
              <!-- 右侧：座位类型信息 -->
              <div class="seat-types-section-inline">
                <div class="seat-types-title-inline">第2程座位类型</div>
                <div class="seat-types-grid-inline">
                  <div
                    v-for="(seatInfo, index) in sortedSecondRideSeatInfo"
                    :key="index"
                    class="seat-type-item-inline"
                  >
                    <div class="seat-type-name">{{ seatInfo.seatType }}</div>
                    <div class="seat-type-info">
                      <div
                        class="seat-availability"
                        :class="{
                          rich: getleftType(seatInfo.left) === 'rich',
                          few: getleftType(seatInfo.left) === 'few',
                          little: getleftType(seatInfo.left) === 'little',
                          none: getleftType(seatInfo.left) === 'none',
                        }"
                      >
                        {{ formatleft(getleftType(seatInfo.left), seatInfo.left) }}
                      </div>
                      <div class="seat-price">¥{{ seatInfo.price }}</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 乘车人管理卡片 -->
      <div class="passenger-management-card">
        <div class="passenger-management-container">
          <!-- 左侧：添加乘车人表单 -->
          <div class="passenger-form-section">
            <div class="form-header">
              <div class="header-left">
                <h2 class="form-title">添加乘车人</h2>
                <p class="form-subtitle">请填写乘车人信息</p>
              </div>
              <div class="header-right">
                <el-button
                  class="import-btn"
                  :icon="Download"
                  @click="showImportDialog"
                >
                  从预填信息导入
                </el-button>
              </div>
            </div>

            <div class="passenger-form">
              <div class="form-grid">
                <div class="form-item">
                  <label class="form-label">姓名 <span class="required">*</span></label>
                  <el-input
                    v-model="passengerForm.name"
                    placeholder="请输入真实姓名"
                    maxlength="20"
                    clearable
                  />
                </div>

                <div class="form-item">
                  <label class="form-label">身份证号 <span class="required">*</span></label>
                  <el-input
                    v-model="passengerForm.identityCardId"
                    placeholder="请输入18位身份证号"
                    maxlength="18"
                    :class="{ 'error-input': identityCardIdError }"
                    clearable
                    @input="checkIdentityCardId"
                    @change="checkIdentityCardId"
                  />
                  <div v-if="identityCardIdError" class="error-message">
                    {{ identityCardIdErrorMsg }}
                  </div>
                </div>

                <!-- 直达模式座位选择 -->
                <div v-if="ticketServiceStore.queryMode === 'direct'" class="form-item full-width">
                  <label class="form-label">座位类型 <span class="required">*</span></label>
                  <div class="seat-type-selection">
                    <div
                      v-for="(seatInfo, index) in sortedDirectSeatInfo"
                      :key="index"
                      class="seat-type-option"
                      :class="{ selected: passengerForm.seatType === seatInfo.seatType }"
                      @click="passengerForm.seatType = seatInfo.seatType"
                    >
                      <div class="seat-type-name">{{ seatInfo.seatType }}</div>
                      <div class="seat-type-price">¥{{ seatInfo.price }}</div>
                      <div
                        class="seat-availability"
                        :class="{
                          rich: getleftType(seatInfo.left) === 'rich',
                          few: getleftType(seatInfo.left) === 'few',
                          little: getleftType(seatInfo.left) === 'little',
                          none: getleftType(seatInfo.left) === 'none',
                        }"
                      >
                        {{ formatleft(getleftType(seatInfo.left), seatInfo.left) }}
                      </div>
                    </div>
                  </div>
                </div>

                <!-- 中转模式座位选择 -->
                <div v-else class="form-item full-width">
                  <label class="form-label">第一程座位类型 <span class="required">*</span></label>
                  <div class="seat-type-selection">
                    <div
                      v-for="(seatInfo, index) in sortedFirstRideSeatInfo"
                      :key="index"
                      class="seat-type-option"
                      :class="{ selected: passengerForm.firstRideSeatType === seatInfo.seatType }"
                      @click="passengerForm.firstRideSeatType = seatInfo.seatType"
                    >
                      <div class="seat-type-name">{{ seatInfo.seatType }}</div>
                      <div class="seat-type-price">¥{{ seatInfo.price }}</div>
                      <div
                        class="seat-availability"
                        :class="{
                          rich: getleftType(seatInfo.left) === 'rich',
                          few: getleftType(seatInfo.left) === 'few',
                          little: getleftType(seatInfo.left) === 'little',
                          none: getleftType(seatInfo.left) === 'none',
                        }"
                      >
                        {{ formatleft(getleftType(seatInfo.left), seatInfo.left) }}
                      </div>
                    </div>
                  </div>

                  <label class="form-label" style="margin-top: 20px"
                    >第二程座位类型 <span class="required">*</span></label
                  >
                  <div class="seat-type-selection">
                    <div
                      v-for="(seatInfo, index) in sortedSecondRideSeatInfo"
                      :key="index"
                      class="seat-type-option"
                      :class="{ selected: passengerForm.secondRideSeatType === seatInfo.seatType }"
                      @click="passengerForm.secondRideSeatType = seatInfo.seatType"
                    >
                      <div class="seat-type-name">{{ seatInfo.seatType }}</div>
                      <div class="seat-type-price">¥{{ seatInfo.price }}</div>
                      <div
                        class="seat-availability"
                        :class="{
                          rich: getleftType(seatInfo.left) === 'rich',
                          few: getleftType(seatInfo.left) === 'few',
                          little: getleftType(seatInfo.left) === 'little',
                          none: getleftType(seatInfo.left) === 'none',
                        }"
                      >
                        {{ formatleft(getleftType(seatInfo.left), seatInfo.left) }}
                      </div>
                    </div>
                  </div>
                </div>

                <!-- 偏好座位选择 -->
                <div class="form-item full-width">
                  <label class="form-label">偏好座位 (可选)</label>
                  <div class="seat-preference-selection">
                    <div class="seat-row">
                      <div
                        v-for="seat in ['A', 'B', 'C']"
                        :key="seat"
                        class="seat clickable"
                        :class="{
                          selected: passengerForm.preferredSeatLocation === seat,
                          window: seat === 'A',
                          aisle: seat === 'C',
                        }"
                        @click="
                          passengerForm.preferredSeatLocation = seat as 'A' | 'B' | 'C' | 'D' | 'F'
                        "
                      >
                        {{ seat }}
                      </div>
                      <!-- 过道间距 -->
                      <div class="aisle-gap"></div>
                      <div
                        v-for="seat in ['D', 'F']"
                        :key="seat"
                        class="seat clickable"
                        :class="{
                          selected: passengerForm.preferredSeatLocation === seat,
                          window: seat === 'F',
                          aisle: seat === 'D',
                        }"
                        @click="
                          passengerForm.preferredSeatLocation = seat as 'A' | 'B' | 'C' | 'D' | 'F'
                        "
                      >
                        {{ seat }}
                      </div>
                    </div>
                    <div class="seat-description">
                      <div class="desc-item">
                        <span class="seat-type window-type">窗</span>
                        <span class="desc-text">靠窗 (A, F)</span>
                      </div>
                      <div class="desc-item">
                        <span class="seat-type aisle-type">道</span>
                        <span class="desc-text">靠过道 (C, D)</span>
                      </div>
                      <div class="desc-item">
                        <span class="seat-type middle-type">中</span>
                        <span class="desc-text">中间 (B)</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <div class="form-actions">
                <el-button class="add-passenger-btn" type="primary" @click="addPassenger">
                  添加乘车人
                </el-button>
                <el-button class="clear-form-btn" @click="clearForm"> 清空表单 </el-button>
              </div>
            </div>
          </div>

          <!-- 右侧：乘车人列表 -->
          <div class="passenger-list-section">
            <div class="list-header">
              <h2 class="list-title">乘车人列表</h2>
              <span class="passenger-count">{{ passengers.length }} 人</span>
            </div>

            <div v-if="passengers.length === 0" class="empty-passenger-list">
              <div class="empty-icon">
                <el-icon size="48"><User /></el-icon>
              </div>
              <p class="empty-text">暂无乘车人</p>
              <p class="empty-subtext">请在左侧添加乘车人信息</p>
            </div>

            <div
              v-else
              class="passenger-list"
              :class="{ transfer: ticketServiceStore.queryMode === 'indirect' }"
            >
              <div v-for="(passenger, index) in passengers" :key="index" class="passenger-item">
                <div class="passenger-info">
                  <div class="passenger-name">{{ passenger.name }}</div>
                  <div class="passenger-id">{{ desensitizeIdCard(passenger.identityCardId) }}</div>

                  <!-- 直达模式显示 -->
                  <div v-if="ticketServiceStore.queryMode === 'direct'" class="seat-info">
                    <div class="seat-type-tag">{{ passenger.seatType }}</div>
                    <div v-if="passenger.preferredSeatLocation" class="preferred-seat">
                      偏好: {{ getSeatLocationText(passenger.preferredSeatLocation) }}
                    </div>
                  </div>

                  <!-- 中转模式显示 -->
                  <div v-else class="seat-info transfer">
                    <div class="transfer-seat-info">
                      <span class="transfer-label">第1程:</span>
                      <div class="seat-type-tag">{{ passenger.firstRideSeatType }}</div>
                    </div>
                    <div class="transfer-seat-info">
                      <span class="transfer-label">第2程:</span>
                      <div class="seat-type-tag">{{ passenger.secondRideSeatType }}</div>
                    </div>
                    <div v-if="passenger.preferredSeatLocation" class="preferred-seat">
                      偏好: {{ getSeatLocationText(passenger.preferredSeatLocation) }}
                    </div>
                  </div>
                </div>

                <div class="passenger-actions">
                  <el-button
                    class="delete-passenger-btn"
                    :icon="Delete"
                    size="small"
                    text
                    @click="removePassenger(index)"
                  >
                    删除
                  </el-button>
                </div>
              </div>
            </div>

            <!-- 总价显示 -->
            <div v-if="passengers.length > 0" class="total-price-section">
              <div class="price-breakdown">
                <div class="price-item">
                  <span class="price-label">乘车人数</span>
                  <span class="price-value">{{ passengers.length }} 人</span>
                </div>
                <div class="price-item total">
                  <span class="price-label">预估总价</span>
                  <span class="price-value">¥{{ calculateTotalPrice() }}</span>
                </div>
              </div>
              <el-button class="proceed-btn" type="primary" size="large">
                继续预订 ({{ passengers.length }}人)
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 预填信息导入对话框 -->
    <el-dialog
      v-model="importDialogVisible"
      title="选择预填信息"
      width="500px"
      class="import-dialog"
      :close-on-click-modal="false"
    >
      <div class="import-content">
        <div v-if="prefilledInfos.length === 0" class="empty-import-state">
          <div class="empty-icon">
            <el-icon size="48"><User /></el-icon>
          </div>
          <p class="empty-text">暂无预填信息</p>
          <p class="empty-subtext">请先在个人中心添加预填信息</p>
        </div>

        <div v-else class="prefilled-list">
          <div
            v-for="(info, index) in prefilledInfos"
            :key="info.personalId"
            class="prefilled-item"
            :class="{ selected: selectedPrefilledInfo?.personalId === info.personalId }"
            @click="selectPrefilledInfo(info)"
          >
            <div class="prefilled-info">
              <div class="name-section">
                <span class="prefilled-name">{{ info.name }}</span>
                <span v-if="info.default" class="self-badge">本人</span>
              </div>
              <div class="prefilled-id">{{ desensitizeIdCard(info.identityCardId) }}</div>
              <div class="prefilled-seat">
                偏好座位: {{ getSeatLocationText(info.preferredSeatLocation) }}
              </div>
            </div>
            <div class="selection-indicator">
              <el-icon v-if="selectedPrefilledInfo?.personalId === info.personalId" class="check-icon">
                <Check />
              </el-icon>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <div class="import-dialog-footer">
          <el-button @click="importDialogVisible = false">取消</el-button>
          <el-button
            type="primary"
            :disabled="!selectedPrefilledInfo"
            @click="importPrefilledInfo"
          >
            导入选中信息
          </el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script lang="ts" setup>
import type {
  directScheduleInfo,
  indirectScheduleInfo,
  stoppingStationInfo,
} from '@/interface/ticketServiceInterface'
import { useTicketServiceStore } from '@/stores/ticketService'
import { computed, ref, reactive, onMounted } from 'vue'
import { Delete, User, Download, Check } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { userApi } from '@/api/UserApi/userApi'

const ticketServiceStore = useTicketServiceStore()

const directPreOrderSchedule = computed(() => {
  return ticketServiceStore.preOrderSchedule as directScheduleInfo
})
const indirectPreOrderSchedule = computed(() => {
  return ticketServiceStore.preOrderSchedule as indirectScheduleInfo
})

// 新增计算属性和方法
type leftType = 'rich' | 'few' | 'little' | 'none'

// 格式化日期
function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    weekday: 'long',
  })
}

// 格式化时间
function formatTime(dateTime: string): string {
  if (!dateTime) return '--:--'
  const date = new Date(dateTime)
  return date.toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
  })
}

// 格式化运行时间
function formatTravelTime(seconds: number): string {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60 + 0.5)
  if (hours > 0) {
    return `${hours}时${minutes}分`
  }
  return `${minutes}分`
}

// 修改余票类型定义，与卡片组件保持一致
// 获取余票类型
function getleftType(left: number): leftType {
  if (left > 30) return 'rich'
  if (left >= 10) return 'few'
  if (left > 0) return 'little'
  return 'none'
}

// 修改函数名和逻辑，与卡片组件保持一致
// 格式化余票信息
function formatleft(type: leftType, count: number): string {
  if (type === 'rich') return '有票'
  if (type === 'few' || type === 'little') return `${count} 张`
  if (type === 'none') return '售罄'
  return '未知'
}

// 直达过夜标志
const directOverDateNum = computed(() => {
  const departureDate = new Date(directPreOrderSchedule.value.departureTime)
  const arrivalDate = new Date(directPreOrderSchedule.value.arrivalTime)
  return arrivalDate.getDate() - departureDate.getDate()
})

const directOverDateFlag = computed(() => directOverDateNum.value > 0)

// 中转过夜标志
const indirectOverDateNum = computed(() => {
  const departureDate = new Date(indirectPreOrderSchedule.value.first_ride.departureTime)
  const arrivalDate = new Date(indirectPreOrderSchedule.value.second_ride.arrivalTime)
  return arrivalDate.getDate() - departureDate.getDate()
})

const indirectOverDateFlag = computed(() => indirectOverDateNum.value > 0)

// 排序座位信息
const sortedDirectSeatInfo = computed(() => {
  return Object.values(directPreOrderSchedule.value.seatInfo).sort((a, b) =>
    a.seatType.localeCompare(b.seatType),
  )
})

const sortedFirstRideSeatInfo = computed(() => {
  return Object.values(indirectPreOrderSchedule.value.first_ride.seatInfo).sort((a, b) =>
    a.seatType.localeCompare(b.seatType),
  )
})

const sortedSecondRideSeatInfo = computed(() => {
  return Object.values(indirectPreOrderSchedule.value.second_ride.seatInfo).sort((a, b) =>
    a.seatType.localeCompare(b.seatType),
  )
})

// 列车信息表格列定义
const columns = [
  {
    title: '途径站点',
    dataIndex: 'stationName',
    key: 'stationName',
  },
  {
    title: '到站时间',
    dataIndex: 'arrivalTime',
    key: 'arrivalTime',
    customRender: ({ text }: { text: string }) => formatTrainInfoTime(text),
  },
  {
    title: '出发时间',
    dataIndex: 'departureTime',
    key: 'departureTime',
    customRender: ({ text }: { text: string }) => formatTrainInfoTime(text),
  },
  {
    title: '停留时间',
    dataIndex: 'stopTime',
    key: 'stopTime',
    customRender: ({ record }: { record: stoppingStationInfo }) => {
      if (record.arrivalTime && record.departureTime) {
        const arrivalDate = new Date(record.arrivalTime)
        const departureDate = new Date(record.departureTime)
        const stopSeconds = (departureDate.getTime() - arrivalDate.getTime()) / 1000
        return formatTravelTime(stopSeconds)
      }
      return '--'
    },
  },
]

// 格式化车次信息时间
function formatTrainInfoTime(text: string): string {
  const time = formatTime(text)
  const date = new Date(text)
  const departureDate = new Date(directPreOrderSchedule.value.departureTime)
  const overNum = date.getDate() - departureDate.getDate()
  if (overNum > 0) {
    return `${time} +${overNum}`
  }
  return time
}

// 应用行样式
const handleCustomRow = (record: any) => ({
  style: record.rowStyle,
})

// 按时间顺序排序直达车次信息
const sortedDirectRoute = computed(() => {
  return directPreOrderSchedule.value.route.sort((a, b) => {
    let aDate = null,
      bDate = null
    if (a.arrivalTime) {
      aDate = new Date(a.arrivalTime).getTime()
    } else {
      aDate = new Date('1970-01-01T00:00:00Z').getTime()
    }
    if (b.arrivalTime) {
      bDate = new Date(b.arrivalTime).getTime()
    } else {
      bDate = new Date('1970-01-01T00:00:00Z').getTime()
    }
    return aDate - bDate
  })
})

// 获取直达起终点站下标
const directStartIndex = computed(() => {
  return sortedDirectRoute.value.findIndex(
    (station) =>
      station.stationName === directPreOrderSchedule.value.departureStation &&
      station.departureTime === directPreOrderSchedule.value.departureTime,
  )
})

const directEndIndex = computed(() => {
  return sortedDirectRoute.value.findIndex(
    (station) =>
      station.stationName === directPreOrderSchedule.value.arrivalStation &&
      station.arrivalTime === directPreOrderSchedule.value.arrivalTime,
  )
})

// 处理直达车次信息
const processedDirectRoute = computed(() => {
  return sortedDirectRoute.value.map((station, index) => {
    const rowStyle: any = {}
    if (index >= directStartIndex.value && index <= directEndIndex.value) {
      rowStyle.backgroundColor = '#f0f5ff'
    }
    if (index === directStartIndex.value || index === directEndIndex.value) {
      rowStyle.color = '#1677ff'
    }
    return {
      ...station,
      rowStyle,
    }
  })
})

// 格式化第二程日期（如果是第二天或更晚）
function formatNextDate(dateTimeStr: string): string {
  const date = new Date(dateTimeStr)
  const queryDate = new Date(ticketServiceStore.queryDate)
  const diffDays = Math.floor((date.getTime() - queryDate.getTime()) / (1000 * 60 * 60 * 24))

  if (diffDays === 0) {
    return formatDate(ticketServiceStore.queryDate)
  } else if (diffDays === 1) {
    return date.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      weekday: 'long',
    })
  } else {
    return date.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      weekday: 'long',
    })
  }
}

// 按时间顺序排序中转车次信息
const sortedFirstRideRoute = computed(() => {
  return indirectPreOrderSchedule.value.first_ride.route.sort((a, b) => {
    let aDate = null,
      bDate = null
    if (a.arrivalTime) {
      aDate = new Date(a.arrivalTime).getTime()
    } else {
      aDate = new Date('1970-01-01T00:00:00Z').getTime()
    }
    if (b.arrivalTime) {
      bDate = new Date(b.arrivalTime).getTime()
    } else {
      bDate = new Date('1970-01-01T00:00:00Z').getTime()
    }
    return aDate - bDate
  })
})

const sortedSecondRideRoute = computed(() => {
  return indirectPreOrderSchedule.value.second_ride.route.sort((a, b) => {
    let aDate = null,
      bDate = null
    if (a.arrivalTime) {
      aDate = new Date(a.arrivalTime).getTime()
    } else {
      aDate = new Date('1970-01-01T00:00:00Z').getTime()
    }
    if (b.arrivalTime) {
      bDate = new Date(b.arrivalTime).getTime()
    } else {
      bDate = new Date('1970-01-01T00:00:00Z').getTime()
    }
    return aDate - bDate
  })
})

// 获取中转起终点站下标
const firstRideStartIndex = computed(() => {
  return sortedFirstRideRoute.value.findIndex(
    (station) =>
      station.stationName === indirectPreOrderSchedule.value.first_ride.departureStation &&
      station.departureTime === indirectPreOrderSchedule.value.first_ride.departureTime,
  )
})

const firstRideEndIndex = computed(() => {
  return sortedFirstRideRoute.value.findIndex(
    (station) =>
      station.stationName === indirectPreOrderSchedule.value.first_ride.arrivalStation &&
      station.arrivalTime === indirectPreOrderSchedule.value.first_ride.arrivalTime,
  )
})

const secondRideStartIndex = computed(() => {
  return sortedSecondRideRoute.value.findIndex(
    (station) =>
      station.stationName === indirectPreOrderSchedule.value.second_ride.departureStation &&
      station.departureTime === indirectPreOrderSchedule.value.second_ride.departureTime,
  )
})

const secondRideEndIndex = computed(() => {
  return sortedSecondRideRoute.value.findIndex(
    (station) =>
      station.stationName === indirectPreOrderSchedule.value.second_ride.arrivalStation &&
      station.arrivalTime === indirectPreOrderSchedule.value.second_ride.arrivalTime,
  )
})

// 处理中转车次信息
const processedFirstRideRoute = computed(() => {
  return sortedFirstRideRoute.value.map((station, index) => {
    const rowStyle: any = {}
    if (index >= firstRideStartIndex.value && index <= firstRideEndIndex.value) {
      rowStyle.backgroundColor = '#f0f5ff'
    }
    if (index === firstRideStartIndex.value || index === firstRideEndIndex.value) {
      rowStyle.color = '#1677ff'
    }
    return {
      ...station,
      rowStyle,
    }
  })
})

const processedSecondRideRoute = computed(() => {
  return sortedSecondRideRoute.value.map((station, index) => {
    const rowStyle: any = {}
    if (index >= secondRideStartIndex.value && index <= secondRideEndIndex.value) {
      rowStyle.backgroundColor = '#f0f5ff'
    }
    if (index === secondRideStartIndex.value || index === secondRideEndIndex.value) {
      rowStyle.color = '#1677ff'
    }
    return {
      ...station,
      rowStyle,
    }
  })
})

// 乘车人相关数据
interface PassengerInfo {
  name: string
  identityCardId: string
  seatType?: string
  firstRideSeatType?: string
  secondRideSeatType?: string
  preferredSeatLocation?: 'A' | 'B' | 'C' | 'D' | 'F'
}

const passengers = ref<PassengerInfo[]>([])

const passengerForm = reactive<PassengerInfo>({
  name: '',
  identityCardId: '',
  seatType: '',
  firstRideSeatType: '',
  secondRideSeatType: '',
  preferredSeatLocation: undefined,
})

// 身份证号验证
const identityCardIdError = ref(false)
const identityCardIdErrorMsg = ref('')

// 身份证号检测
function checkIdentityCardId() {
  identityCardIdError.value = false
  identityCardIdErrorMsg.value = ''

  if (passengerForm.identityCardId === '') {
    return
  }

  const weight = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2]
  const checkCode = ['1', '0', 'X', '9', '8', '7', '6', '5', '4', '3', '2']

  if (passengerForm.identityCardId.length !== 18) {
    identityCardIdError.value = true
    identityCardIdErrorMsg.value = '身份证号码长度应为18位'
    return
  }

  let sum = 0
  for (let i = 0; i < 17; i++) {
    if (!/\d/.test(passengerForm.identityCardId[i])) {
      identityCardIdError.value = true
      identityCardIdErrorMsg.value = '身份证号码前17位应全部为数字'
      return
    }
    sum += parseInt(passengerForm.identityCardId[i], 10) * weight[i]
  }

  const mod = sum % 11
  const expectedCheckCode = checkCode[mod].toUpperCase()
  const actualCheckCode = passengerForm.identityCardId[17].toUpperCase()

  if (actualCheckCode !== expectedCheckCode) {
    identityCardIdError.value = true
    identityCardIdErrorMsg.value = '身份证号码校验失败'
  }
}

// 添加乘车人
function addPassenger() {
  console.log('添加乘车人', passengerForm)
  if (!validatePassengerForm()) {
    return
  }

  // 检查是否已存在相同身份证号的乘车人
  if (passengers.value.some((p) => p.identityCardId === passengerForm.identityCardId)) {
    ElMessage.error('该身份证号已存在')
    return
  }

  passengers.value.push({ ...passengerForm })
  ElMessage.success('添加乘车人成功')
  clearForm()
}

// 删除乘车人
function removePassenger(index: number) {
  passengers.value.splice(index, 1)
  ElMessage.success('删除乘车人成功')
}

// 清空表单
function clearForm() {
  passengerForm.name = ''
  passengerForm.identityCardId = ''
  passengerForm.seatType = ''
  passengerForm.firstRideSeatType = ''
  passengerForm.secondRideSeatType = ''
  passengerForm.preferredSeatLocation = undefined
  identityCardIdError.value = false
  identityCardIdErrorMsg.value = ''
}

// 验证表单
function validatePassengerForm(): boolean {
  if (!passengerForm.name.trim()) {
    ElMessage.error('请输入姓名')
    return false
  }

  if (!passengerForm.identityCardId.trim()) {
    ElMessage.error('请输入身份证号')
    return false
  }

  checkIdentityCardId()
  if (identityCardIdError.value) {
    ElMessage.error(identityCardIdErrorMsg.value)
    return false
  }

  if (ticketServiceStore.queryMode === 'direct') {
    if (!passengerForm.seatType) {
      ElMessage.error('请选择座位类型')
      return false
    }
  } else {
    if (!passengerForm.firstRideSeatType) {
      ElMessage.error('请选择第一程座位类型')
      return false
    }
    if (!passengerForm.secondRideSeatType) {
      ElMessage.error('请选择第二程座位类型')
      return false
    }
  }

  return true
}

// 计算总价
function calculateTotalPrice(): number {
  return passengers.value.reduce((total, passenger) => {
    if (ticketServiceStore.queryMode === 'direct') {
      const seatInfo = sortedDirectSeatInfo.value.find((s) => s.seatType === passenger.seatType)
      return total + (seatInfo?.price || 0)
    } else {
      const firstSeatInfo = sortedFirstRideSeatInfo.value.find(
        (s) => s.seatType === passenger.firstRideSeatType,
      )
      const secondSeatInfo = sortedSecondRideSeatInfo.value.find(
        (s) => s.seatType === passenger.secondRideSeatType,
      )
      return total + (firstSeatInfo?.price || 0) + (secondSeatInfo?.price || 0)
    }
  }, 0)
}

// 身份证号脱敏
function desensitizeIdCard(idCard: string): string {
  if (!idCard || idCard.length < 8) return idCard
  return idCard.slice(0, 4) + '****' + idCard.slice(-4)
}

// 获取座位位置文字描述
function getSeatLocationText(location: 'A' | 'B' | 'C' | 'D' | 'F'): string {
  const locationMap = {
    A: '靠窗 (A)',
    B: '中间 (B)',
    C: '靠过道 (C)',
    D: '靠过道 (D)',
    F: '靠窗 (F)',
  }
  return locationMap[location]
}

// 预填信息相关数据
interface PrefilledInfo {
  personalId: string
  name: string
  identityCardId: string
  preferredSeatLocation?: 'A' | 'B' | 'C' | 'D' | 'F'
  default: boolean
}

const prefilledInfos = ref<PrefilledInfo[]>([])
const importDialogVisible = ref(false)
const selectedPrefilledInfo = ref<PrefilledInfo | null>(null)

// 加载预填信息列表
async function loadPrefilledInfos() {
  try {
    const response = await userApi.queryPersonalInfo()
    if (response.data.code === 200) {
      prefilledInfos.value = response.data.data
    } else {
      console.error('获取预填信息失败:', response.data.message)
    }
  } catch (error) {
    console.error('获取预填信息失败:', error)
  }
}

// 显示导入对话框
function showImportDialog() {
  selectedPrefilledInfo.value = null
  importDialogVisible.value = true
  loadPrefilledInfos()
}

// 选择预填信息
function selectPrefilledInfo(info: PrefilledInfo) {
  selectedPrefilledInfo.value = info
}

// 导入预填信息到表单
function importPrefilledInfo() {
  if (!selectedPrefilledInfo.value) {
    ElMessage.error('请选择要导入的预填信息')
    return
  }

  const selectedInfo = selectedPrefilledInfo.value

  // 检查是否已存在相同身份证号的乘车人
  if (passengers.value.some((p) => p.identityCardId === selectedInfo.identityCardId)) {
    ElMessage.error('该身份证号已存在于乘车人列表中')
    return
  }

  // 填充表单数据
  passengerForm.name = selectedInfo.name
  passengerForm.identityCardId = selectedInfo.identityCardId
  passengerForm.preferredSeatLocation = selectedInfo.preferredSeatLocation

  // 重置验证状态
  identityCardIdError.value = false
  identityCardIdErrorMsg.value = ''

  // 关闭对话框
  importDialogVisible.value = false
  selectedPrefilledInfo.value = null

  ElMessage.success('预填信息导入成功')
}

// 在组件挂载时加载预填信息
onMounted(() => {
  loadPrefilledInfos()
})
</script>

<style lang="css" scoped>
.train-transaction {
  min-height: 100vh;
  display: flex;
  justify-content: center;
  background: #f8fafc;
}

.transaction-container {
  width: 100%;
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.schedule-card {
  width: 100%;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border-radius: 24px;
  box-shadow:
    0 20px 40px rgba(0, 0, 0, 0.1),
    0 0 0 1px rgba(255, 255, 255, 0.2);
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.schedule-card:hover {
  transform: translateY(-4px);
  box-shadow:
    0 32px 64px rgba(0, 0, 0, 0.15),
    0 0 0 1px rgba(255, 255, 255, 0.3);
}

/* 卡片头部 */
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 30px 40px 20px;
  background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
}

.page-title {
  font-size: 32px;
  font-weight: bold;
  color: #1a202c;
  margin: 0 0 8px 0;
  letter-spacing: -0.5px;
}

.page-subtitle {
  font-size: 16px;
  color: #64748b;
  margin: 0;
  font-weight: 400;
}

.header-actions {
  align-items: center;
}

.query-type-badge {
  padding: 12px 20px;
  border-radius: 12px;
  font-weight: 600;
  font-size: 15px;
  border: none;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.query-type-badge.direct {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  color: white;
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3);
}

.query-type-badge.indirect {
  background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
  color: white;
  box-shadow: 0 4px 12px rgba(245, 158, 11, 0.3);
}

.query-type-badge:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4);
}

/* 分割线 */
.divider {
  height: 1px;
  background: linear-gradient(90deg, transparent, #e2e8f0, transparent);
  margin: 0 40px;
}

.content-layout {
  padding: 30px 40px;
}

/* 行程卡片 */
.route-card {
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
  border-radius: 12px;
  padding: 1.2rem;
  border: 1px solid #e2e8f0;
  margin-bottom: 1rem;
  transition: all 0.3s ease;
  display: flex;
  flex-direction: row;
  gap: 5%;
}

.route-card.transfer {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.route-card:hover {
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  transform: translateY(-2px);
}

.route-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.5rem;
  width: 50%;
}

.route-content.transfer {
  width: 100%;
}

.segment-label {
  font-size: 1rem;
  font-weight: bold;
  color: #1677ff;
  text-align: center;
  padding: 0.5rem;
  background: rgba(22, 119, 255, 0.1);
  border-radius: 8px;
  width: 100%;
}

.route-content-section {
  display: flex;
  align-items: center;
  gap: 1.5rem;
}

.station-block {
  text-align: center;
  min-width: 90px;
}

.station-block.arrival {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
}

.time-medium {
  font-size: 2rem;
  font-weight: bold;
  color: #1677ff;
  line-height: 1.2;
  text-shadow: 0 2px 4px rgba(22, 119, 255, 0.1);
}

.station-name {
  font-size: 1rem;
  font-weight: bold;
  color: #334155;
  margin-top: 0.5rem;
}

.journey-middle {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
}

.train-duration-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  justify-content: center;
}

.journey-arrow {
  display: flex;
  align-items: center;
  width: 100%;
  gap: 0.75rem;
  height: 14px;
}

.arrow-line {
  flex: 1;
  height: 3px;
  background: linear-gradient(90deg, #cbd5e1 0%, #1677ff 100%);
  border-radius: 2px;
}

.train-number-badge {
  background: #1677ff;
  color: white;
  padding: 0.3rem 0.8rem;
  border-radius: 12px;
  font-weight: bold;
  font-size: 0.9rem;
  cursor: pointer;
  transition: all 0.3s ease;
}

.train-number-badge:hover {
  background: #0ea5e9;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(22, 119, 255, 0.3);
}

.train-number-badge.small {
  font-size: 0.8rem;
  padding: 0.25rem 0.6rem;
}

.duration-text {
  font-size: 0.9rem;
  color: #48566b;
  font-weight: 600;
  background: rgba(100, 116, 139, 0.1);
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
}

.over-date-badge {
  background: linear-gradient(135deg, #f97316 0%, #ea580c 100%);
  color: white;
  padding: 0.2rem 0.6rem;
  border-radius: 10px;
  font-size: 0.75rem;
  font-weight: 700;
  margin-top: 0.5rem;
  display: inline-block;
  box-shadow: 0 2px 8px rgba(249, 115, 22, 0.3);
}

.seat-types-section-inline {
  flex: 1;
  padding: 1rem;
  background: rgba(22, 119, 255, 0.03);
  border-radius: 8px;
  border: 1px solid rgba(22, 119, 255, 0.1);
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.seat-types-title-inline {
  font-size: 0.9rem;
  font-weight: bold;
  color: #1677ff;
  text-align: center;
  padding: 0.25rem;
  background: rgba(22, 119, 255, 0.1);
  border-radius: 6px;
}

.seat-types-grid-inline {
  display: grid;
  grid-auto-flow: column;
  grid-auto-columns: 1fr;
  gap: 0.5rem;
  justify-items: center;
}

.seat-type-item-inline {
  background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%);
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  padding: 0.5rem;
  transition: all 0.3s ease;
  max-width: 150px;
  width: 100%;
}

.seat-type-item-inline:hover {
  border-color: #1677ff;
  box-shadow: 0 2px 8px rgba(22, 119, 255, 0.15);
  transform: translateY(-1px);
}

.seat-type-name {
  font-size: 0.9rem;
  font-weight: bold;
  color: #1e293b;
  margin-bottom: 0.25rem;
  text-align: center;
}

.seat-type-info {
  display: flex;
  justify-content: space-around;
  align-items: center;
}

.seat-availability {
  font-size: 0.7rem;
  font-weight: 600;
  padding: 0.15rem 0.5rem;
  border-radius: 8px;
  text-align: center;
}

.seat-price {
  font-size: 0.8rem;
  font-weight: 700;
  color: #f97316;
  white-space: nowrap;
}

.seat-availability.rich {
  background: rgba(16, 185, 129, 0.1);
  color: #10b981;
}

.seat-availability.few {
  background: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

.seat-availability.little {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.seat-availability.none {
  background: rgba(156, 163, 175, 0.1);
  color: #9ca3af;
}

/* 中转相关 */
.transfer-info {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1.5rem;
}

/* 乘车人管理卡片 */
.passenger-management-card {
  width: 100%;
  max-width: 1200px;
  margin: 20px auto 0;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border-radius: 24px;
  box-shadow:
    0 20px 40px rgba(0, 0, 0, 0.1),
    0 0 0 1px rgba(255, 255, 255, 0.2);
  overflow: hidden;
}

.passenger-management-container {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 2px;
  background: #e2e8f0;
}

/* 左侧表单区域 */
.passenger-form-section {
  background: #fff;
  padding: 30px;
}

.form-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
}

.header-left {
  flex: 1;
}

.header-right {
  margin-left: 20px;
}

.import-btn {
  background: #f8fafc;
  color: #667eea;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  font-weight: 500;
  transition: all 0.2s;
}

.import-btn:hover {
  background: #667eea;
  color: white;
  border-color: #667eea;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.2);
}

.form-title {
  font-size: 24px;
  font-weight: 700;
  color: #1a202c;
  margin: 0 0 8px 0;
}

.form-subtitle {
  font-size: 14px;
  color: #64748b;
  margin: 0;
}

.passenger-form {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.form-grid {
  display: grid;
  gap: 20px;
}

.form-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-item.full-width {
  grid-column: 1 / -1;
}

.form-label {
  font-size: 14px;
  font-weight: 600;
  color: #374151;
}

.required {
  color: #dc2626;
}

/* 座位类型选择 */
.seat-type-selection {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 12px;
}

.seat-type-option {
  border: 2px solid #e2e8f0;
  border-radius: 12px;
  padding: 16px 12px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s;
  background: #fff;
}

.seat-type-option:hover {
  border-color: #667eea;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.2);
}

.seat-type-option.selected {
  border-color: #667eea;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.1) 0%, rgba(118, 75, 162, 0.1) 100%);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
}

.seat-type-price {
  font-size: 16px;
  font-weight: 700;
  color: #f97316;
  margin-bottom: 4px;
}

/* 偏好座位选择 */
.seat-preference-selection {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 20px;
  background: #f8fafc;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
}

.seat-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.seat.clickable {
  width: 48px;
  height: 48px;
  border: 2px solid #e2e8f0;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  font-weight: 600;
  color: #6b7280;
  background: #fff;
  cursor: pointer;
  transition: all 0.2s;
}

.seat.clickable:hover {
  transform: scale(1.05);
  border-color: #667eea;
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.2);
}

.seat.window {
  border-color: #3b82f6;
  color: #3b82f6;
}

.seat.aisle {
  border-color: #10b981;
  color: #10b981;
}

.seat.selected {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-color: #667eea;
  color: white;
  transform: scale(1.1);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
}

.aisle-gap {
  width: 24px;
  height: 2px;
  background: #d1d5db;
  margin: 0 8px;
  border-radius: 1px;
}

.seat-description {
  display: flex;
  justify-content: center;
  gap: 24px;
  padding: 12px 0;
  border-top: 1px solid #e2e8f0;
}

.desc-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.seat-type {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 600;
}

.window-type {
  background: #dbeafe;
  color: #1e40af;
  border: 1px solid #3b82f6;
}

.aisle-type {
  background: #d1fae5;
  color: #047857;
  border: 1px solid #10b981;
}

.middle-type {
  background: #f3f4f6;
  color: #6b7280;
  border: 1px solid #d1d5db;
}

.desc-text {
  font-size: 12px;
  color: #6b7280;
  font-weight: 500;
}

/* 表单操作按钮 */
.form-actions {
  display: flex;
  gap: 12px;
  padding-top: 20px;
  border-top: 1px solid #f1f5f9;
}

.add-passenger-btn {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  padding: 12px 24px;
  border-radius: 12px;
  font-weight: 600;
}

.clear-form-btn {
  background: #f3f4f6;
  color: #6b7280;
  border: 1px solid #e5e7eb;
  padding: 12px 24px;
  border-radius: 12px;
  font-weight: 600;
}

/* 右侧乘车人列表 */
.passenger-list-section {
  background: #f8fafc;
  padding: 30px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
}

.list-title {
  font-size: 24px;
  font-weight: 700;
  color: #1a202c;
  margin: 0;
}

.passenger-count {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 6px 12px;
  border-radius: 12px;
  font-size: 14px;
  font-weight: 600;
}

/* 空状态 */
.empty-passenger-list {
  text-align: center;
  padding: 40px 20px;
  color: #6b7280;
}

.empty-icon {
  margin-bottom: 16px;
}

.empty-text {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 8px 0;
}

.empty-subtext {
  font-size: 14px;
  margin: 0;
}

/* 乘车人列表 */
.passenger-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex-grow: 1;
  overflow: auto;
  max-height: 390px;
}

.passenger-list.transfer {
  max-height: 565px;
}

.passenger-item {
  background: #fff;
  border-radius: 12px;
  padding: 20px;
  border: 1px solid #e2e8f0;
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  transition: all 0.2s;
}

.passenger-item:hover {
  border-color: #667eea;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.1);
}

.passenger-info {
  flex: 1;
}

.passenger-name {
  font-size: 16px;
  font-weight: 600;
  color: #1a202c;
  margin-bottom: 8px;
}

.passenger-id {
  font-size: 14px;
  color: #6b7280;
  margin-bottom: 12px;
}

.seat-info {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.seat-info.transfer {
  flex-direction: column;
  align-items: flex-start;
  gap: 6px;
}

.transfer-seat-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.transfer-label {
  font-size: 12px;
  color: #6b7280;
  font-weight: 500;
}

.seat-type-tag {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
}

.preferred-seat {
  font-size: 12px;
  color: #6b7280;
  background: #f3f4f6;
  padding: 4px 8px;
  border-radius: 6px;
}

.passenger-actions {
  margin-left: 16px;
}

.delete-passenger-btn {
  color: #dc2626;
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 12px;
}

.delete-passenger-btn:hover {
  background: #fee2e2;
  color: #b91c1c;
}

/* 总价区域 */
.total-price-section {
  border-top: 2px solid #e2e8f0;
  padding-top: 20px;
  margin-top: auto;
  flex-shrink: 0;
}

.price-breakdown {
  background: #fff;
  border-radius: 12px;
  padding: 16px;
  margin-bottom: 16px;
  border: 1px solid #e2e8f0;
}

.price-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
}

.price-item.total {
  border-top: 1px solid #f1f5f9;
  margin-top: 8px;
  font-size: 16px;
  font-weight: 600;
}

.price-label {
  color: #6b7280;
}

.price-value {
  color: #1a202c;
  font-weight: 600;
}

.price-item.total .price-value {
  color: #f97316;
  font-size: 18px;
}

.proceed-btn {
  width: 100%;
  height: 48px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  border-radius: 12px;
  font-weight: 600;
  font-size: 16px;
}

/* 错误样式 */
.error-input :deep(.el-input__wrapper) {
  border-color: #f56565 !important;
  box-shadow: 0 0 0 2px rgba(245, 101, 101, 0.2) !important;
}

.error-message {
  color: #f56565;
  font-size: 12px;
  margin-top: 4px;
  font-weight: 500;
}

/* 导入对话框样式 */
.import-dialog :deep(.el-dialog) {
  border-radius: 16px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
}

.import-dialog :deep(.el-dialog__header) {
  padding: 24px 24px 0;
  border-bottom: 1px solid #f1f5f9;
}

.import-dialog :deep(.el-dialog__title) {
  font-size: 20px;
  font-weight: 600;
  color: #1a202c;
}

.import-dialog :deep(.el-dialog__body) {
  padding: 24px;
  max-height: 500px;
  overflow-y: auto;
}

.import-content {
  min-height: 200px;
}

/* 空状态样式 */
.empty-import-state {
  text-align: center;
  padding: 40px 20px;
  color: #6b7280;
}

.empty-import-state .empty-icon {
  margin-bottom: 16px;
}

.empty-import-state .empty-text {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 8px 0;
}

.empty-import-state .empty-subtext {
  font-size: 14px;
  margin: 0;
}

/* 预填信息列表样式 */
.prefilled-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.prefilled-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border: 2px solid #e2e8f0;
  border-radius: 12px;
  background: #fff;
  cursor: pointer;
  transition: all 0.2s;
}

.prefilled-item:hover {
  border-color: #667eea;
  background: rgba(102, 126, 234, 0.02);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.1);
}

.prefilled-item.selected {
  border-color: #667eea;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.1) 0%, rgba(118, 75, 162, 0.1) 100%);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.2);
}

.prefilled-info {
  flex: 1;
}

.name-section {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.prefilled-name {
  font-size: 16px;
  font-weight: 600;
  color: #1a202c;
}

.self-badge {
  display: inline-block;
  padding: 2px 8px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  font-size: 10px;
  font-weight: 600;
  border-radius: 12px;
  letter-spacing: 0.5px;
}

.prefilled-id {
  font-size: 14px;
  color: #6b7280;
  margin-bottom: 4px;
}

.prefilled-seat {
  font-size: 12px;
  color: #6b7280;
  background: #f3f4f6;
  padding: 4px 8px;
  border-radius: 6px;
  display: inline-block;
}

.selection-indicator {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.check-icon {
  color: #10b981;
  font-size: 18px;
}

/* 导入对话框底部 */
.import-dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 20px 24px 24px;
  border-top: 1px solid #f1f5f9;
}

.import-dialog-footer .el-button {
  padding: 10px 20px;
  border-radius: 8px;
  font-weight: 500;
}

/* 响应式设计 */
@media (max-width: 1024px) {
  .passenger-management-container {
    grid-template-columns: 1fr;
  }

  .seat-type-selection {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .passenger-form-section,
  .passenger-list-section {
    padding: 20px;
  }

  .form-actions {
    flex-direction: column;
  }

  .seat-type-selection {
    grid-template-columns: 1fr;
  }

  .import-dialog-footer {
    flex-direction: column-reverse;
  }

  .import-dialog-footer .el-button {
       width: 100%;
  }
}
</style>

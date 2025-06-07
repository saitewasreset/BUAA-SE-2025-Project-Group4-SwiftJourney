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
    </div>
  </div>
</template>

<script lang="ts" setup>
import type {
  directScheduleInfo,
  indirectScheduleInfo,
  stoppingStationInfo,
} from '@/interface/ticketServiceInterface'
import { useTicketServiceStore } from '@/stores/ticketService'
import { computed } from 'vue'

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
</style>

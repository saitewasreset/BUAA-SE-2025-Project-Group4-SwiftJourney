<template>
  <div class="indirect-schedule-info-card">
    <div class="indirect-root">
      <div class="indirect-main-info">
        <!-- 车次信息 -->
        <div class="indirect-schedule-info">
          <!-- 出发站信息 -->
          <div class="departure-info">
            <!-- 时间 -->
            <div class="schedule-time">{{ formatTime(content.first_ride.departureTime) }}</div>
            <!-- 站名 -->
            <div class="schedule-station">{{ content.first_ride.departureStation }}</div>
          </div>
          <!-- 过程信息 1 -->
          <div class="schedule-process">
            <!-- 车次信息 -->
            <a-popover :title="content.first_ride.trainNumber + ' 车次信息'" trigger="click">
              <template #content>
                <a-table
                  size="small"
                  :columns="columns"
                  :data-source="processedRouteFirstRide"
                  :pagination="false"
                  :customRow="handleCustomRow"
                  :scroll="{ y: 200 }"
                  style="width: 400px"
                />
              </template>
              <div
                class="train-number"
                :class="{ small: content.first_ride.trainNumber.length > 5 }"
              >
                {{ content.first_ride.trainNumber }}
              </div>
            </a-popover>
            <!-- 箭头 -->
            <div class="schedule-arrow">
              <div class="arrow-line">
                <div class="line"></div>
                <svg class="arrow-icon" viewBox="0 0 24 24" fill="none">
                  <path d="M5 12h14m-7-7l7 7-7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              </div>
            </div>
            <!-- 到达时间 -->
            <div class="travel-time">{{ formatTime(content.first_ride.arrivalTime) }} 到达</div>
          </div>
          <!-- 中转信息 -->
          <div class="transfer-info">
            <!-- 全程运行时间 -->
            <div class="travel-time-total">
              全程
              {{
                formatTravelTime(
                  content.first_ride.travelTime +
                    content.second_ride.travelTime +
                    content.relaxing_time,
                )
              }}
            </div>
            <!-- 中转站 -->
            <div class="transfer-station">{{ content.first_ride.arrivalStation }}</div>
            <!-- 换乘时间 -->
            <div class="transfer-time">中转换乘 {{ formatTravelTime(content.relaxing_time) }}</div>
          </div>
          <!-- 过程信息 2 -->
          <div class="schedule-process">
            <!-- 车次信息 -->
            <a-popover :title="content.second_ride.trainNumber + ' 车次信息'" trigger="click">
              <template #content>
                <a-table
                  size="small"
                  :columns="columns"
                  :data-source="processedRouteSecondRide"
                  :pagination="false"
                  :customRow="handleCustomRow"
                  :scroll="{ y: 200 }"
                  style="width: 400px"
                />
              </template>
              <div
                class="train-number"
                :class="{ small: content.second_ride.trainNumber.length > 5 }"
              >
                {{ content.second_ride.trainNumber }}
              </div>
            </a-popover>
            <!-- 箭头 -->
            <div class="schedule-arrow">
              <div class="arrow-line">
                <div class="line"></div>
                <svg class="arrow-icon" viewBox="0 0 24 24" fill="none">
                  <path d="M5 12h14m-7-7l7 7-7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              </div>
            </div>
            <!-- 出发时间 -->
            <div class="travel-time">{{ formatTime(content.second_ride.departureTime) }} 出发</div>
          </div>
          <!-- 到达站信息 -->
          <div class="arrival-info">
            <div class="arrival-info-main">
              <!-- 时间 -->
              <div class="schedule-time">{{ formatTime(content.second_ride.arrivalTime) }}</div>
              <!-- 站名 -->
              <div class="schedule-station">{{ content.second_ride.arrivalStation }}</div>
            </div>
            <!-- 过夜标志 -->
            <div class="over-date-flag">
              <div v-if="overDateFlag" class="over-date-badge">+{{ overDateNum }}</div>
            </div>
          </div>
        </div>
        <!-- 车票信息 -->
        <div class="ticket-info">
          <!-- 第 1 程 -->
          <div class="ticket-info-ride">
            <div class="ride-title">第1程</div>
            <!-- 车票信息 -->
            <div class="seat-cards">
              <div
                v-for="(seatInfo, index) in sortedSeatInfoFirstRide"
                :key="index"
                class="seat-card"
              >
                <!-- 座位类型 -->
                <div class="seat-type-info">{{ seatInfo.seatType }}</div>
                <!-- 余票信息 -->
                <div
                  class="remain-count-info"
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
          <!-- 第 2 程 -->
          <div class="ticket-info-ride">
            <div class="ride-title">第2程</div>
            <!-- 车票信息 -->
            <div class="seat-cards">
              <div
                v-for="(seatInfo, index) in sortedSeatInfoSecondRide"
                :key="index"
                class="seat-card"
              >
                <!-- 座位类型 -->
                <div class="seat-type-info">{{ seatInfo.seatType }}</div>
                <!-- 余票信息 -->
                <div
                  class="remain-count-info"
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
        </div>
      </div>
      <!-- 功能区 -->
      <div class="function-area">
        <!-- 最低价格 -->
        <div class="total-price">SC {{ content.first_ride.price + content.second_ride.price }}</div>
        <a-button
          :disable="!checkBookable"
          type="primary"
          class="book-btn"
          @click="onClickBookTicket"
          >订票</a-button
        >
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeMount } from 'vue'
import type {
  indirectScheduleInfo,
  directScheduleInfo,
  stoppingStationInfo,
  seatTypeInfo,
} from '@/interface/ticketServiceInterface'
import { useTicketServiceStore } from '@/stores/ticketService'
import { useRouter } from 'vue-router'

const ticketServiceStore = useTicketServiceStore()
const router = useRouter()

// -------------------- 接口数据类型 --------------------
const props = withDefaults(
  defineProps<{
    content?: indirectScheduleInfo
  }>(),
  {
    content: () => ({
      first_ride: {
        departureStation: '加载中..',
        departureTime: '2000-01-01T08:00:00+08:00',
        arrivalStation: '加载中..',
        arrivalTime: '2000-01-02T08:00:00+08:00',
        originStation: '加载中..',
        originDepartureTime: '2000-01-01T00:00:00+08:00',
        terminalStation: '加载中..',
        terminalArrivalTime: '2000-01-02T23:00:00+08:00',
        trainNumber: 'D9999',
        travelTime: 24 * 60 * 60,
        price: 9999.99,
        route: [
          {
            stationName: '起点站',
            departureTime: '2000-01-01T00:00:00+08:00',
          },
          {
            stationName: '加载中..',
            departureTime: '2000-01-01T08:00:00+08:00',
            arrivalTime: '2000-01-01T07:00:00+08:00',
          },
          {
            stationName: '加载中..',
            departureTime: '2000-01-01T12:00:00+08:00',
            arrivalTime: '2000-01-01T11:00:00+08:00',
          },
          {
            stationName: '加载中..',
            departureTime: '2000-01-02T09:00:00+08:00',
            arrivalTime: '2000-01-02T08:00:00+08:00',
          },
          {
            stationName: '终点站',
            arrivalTime: '2000-01-02T23:00:00+08:00',
          },
        ] as stoppingStationInfo[],
        seatInfo: new Map<string, seatTypeInfo>([
          ['优选一等座', { seatType: '优选一等座', left: 0, price: 0.0 }],
          ['一等座', { seatType: '一等座', left: 0, price: 0.0 }],
          ['二等座', { seatType: '二等座', left: 0, price: 0.0 }],
          ['无座', { seatType: '无座', left: 0, price: 0.0 }],
        ]),
      },
      second_ride: {
        departureStation: '加载中..',
        departureTime: '2000-01-02T09:05:00+08:00',
        arrivalStation: '加载中..',
        arrivalTime: '2000-01-03T17:00:00+08:00',
        originStation: '加载中..',
        originDepartureTime: '2000-01-02T00:00:00+08:00',
        terminalStation: '加载中..',
        terminalArrivalTime: '2000-01-03T23:00:00+08:00',
        trainNumber: 'D8888',
        travelTime: 31 * 60 * 60 + 55 * 60,
        price: 9999.99,
        route: [
          {
            stationName: '起点站',
            departureTime: '2000-01-02T00:00:00+08:00',
          },
          {
            stationName: '加载中..',
            departureTime: '2000-01-02T09:05:00+08:00',
            arrivalTime: '2000-01-02T09:00:00+08:00',
          },
          {
            stationName: '加载中..',
            departureTime: '2000-01-02T12:00:00+08:00',
            arrivalTime: '2000-01-02T11:00:00+08:00',
          },
          {
            stationName: '加载中..',
            departureTime: '2000-01-03T18:00:00+08:00',
            arrivalTime: '2000-01-03T17:00:00+08:00',
          },
          {
            stationName: '终点站',
            arrivalTime: '2000-01-03T23:00:00+08:00',
          },
        ] as stoppingStationInfo[],
        seatInfo: new Map<string, seatTypeInfo>([
          ['优选一等座', { seatType: '优选一等座', left: 0, price: 0.0 }],
          ['一等座', { seatType: '一等座', left: 0, price: 0.0 }],
          ['二等座', { seatType: '二等座', left: 0, price: 0.0 }],
          ['无座', { seatType: '无座', left: 0, price: 0.0 }],
        ]),
      },
      relaxing_time: 65 * 60,
    }),
  },
)
// -------------------- 类型定义 --------------------
// 定义车票信息类型
type leftType = 'rich' | 'few' | 'little' | 'none'
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
// -------------------- 格式化函数 --------------------
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
    return `${hours.toString()}时 ${minutes.toString()}分`
  }
  return `${minutes.toString()}分`
}
// 格式化余票信息
function formatleft(type: leftType, count: number): string {
  if (type === 'rich') {
    return `有票`
  } else if (type === 'few' || type === 'little') {
    return `${count} 张`
  } else if (type === 'none') {
    return '售罄'
  }
  return '未知'
}
// 格式化车次信息时间
function formatTrainInfoTime(text: string): string {
  const time = formatTime(text)
  const date = new Date(text)
  const departureDate = new Date(props.content.first_ride.departureTime)
  const overNum = date.getDate() - departureDate.getDate()
  if (overNum > 0) {
    return `${time} +${overNum}`
  }
  return time
}
// -------------------- 过夜标志逻辑 --------------------
const overDateNum = computed(() => {
  const departureDate = new Date(props.content.first_ride.departureTime)
  const arrivalDate = new Date(props.content.second_ride.arrivalTime)
  return arrivalDate.getDate() - departureDate.getDate()
})
// 过夜标志
const overDateFlag = computed(() => {
  return overDateNum.value > 0 ? true : false
})
// -------------------- 余票判断逻辑 --------------------
// 根据余票数量和总容量判断余票类型
function getleftType(left: number): leftType {
  if (left > 30) {
    return 'rich'
  } else if (left >= 10) {
    return 'few'
  } else if (left > 0) {
    return 'little'
  } else {
    return 'none'
  }
}
// 订票检查
const checkBookable = computed(() => {
  // 中转车次检查第一程余票
  const firstRideHasTicket = Object.values(props.content.first_ride.seatInfo).some(
    (seatInfo) => seatInfo.left > 0,
  )

  // 中转车次检查第二程余票
  const secondRideHasTicket = Object.values(props.content.second_ride.seatInfo).some(
    (seatInfo) => seatInfo.left > 0,
  )
  // 只有两程都有余票才能订票
  return firstRideHasTicket && secondRideHasTicket
})
// -------------------- 车次信息表格 --------------------
// 应用行样式
const handleCustomRow = (record: any) => ({
  style: record.rowStyle,
})
// 按时间顺序排序车次信息
const sortedRouteFirstRide = computed(() => {
  return props.content.first_ride.route.sort((a, b) => {
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
const sortedRouteSecondRide = computed(() => {
  return props.content.second_ride.route.sort((a, b) => {
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
// 获取起终点站下标
const startIndexFirstRide = computed(() => {
  return sortedRouteFirstRide.value.findIndex(
    (station) =>
      station.stationName === props.content.first_ride.departureStation &&
      station.departureTime === props.content.first_ride.departureTime,
  )
})
const endIndexFirstRide = computed(() => {
  return sortedRouteFirstRide.value.findIndex(
    (station) =>
      station.stationName === props.content.first_ride.arrivalStation &&
      station.arrivalTime === props.content.first_ride.arrivalTime,
  )
})
const startIndexSecondRide = computed(() => {
  return sortedRouteSecondRide.value.findIndex(
    (station) =>
      station.stationName === props.content.second_ride.departureStation &&
      station.departureTime === props.content.second_ride.departureTime,
  )
})
const endIndexSecondRide = computed(() => {
  return sortedRouteSecondRide.value.findIndex(
    (station) =>
      station.stationName === props.content.second_ride.arrivalStation &&
      station.arrivalTime === props.content.second_ride.arrivalTime,
  )
})
// 处理车次信息
const processedRouteFirstRide = computed(() => {
  return sortedRouteFirstRide.value.map((station, index) => {
    const rowStyle: any = {}
    if (index >= startIndexFirstRide.value && index <= endIndexFirstRide.value) {
      rowStyle.backgroundColor = '#f0f5ff'
    }
    if (index === startIndexFirstRide.value || index === endIndexFirstRide.value) {
      rowStyle.color = '#1677ff'
    }
    return {
      ...station,
      rowStyle,
    }
  })
})
const processedRouteSecondRide = computed(() => {
  return sortedRouteSecondRide.value.map((station, index) => {
    const rowStyle: any = {}
    if (index >= startIndexSecondRide.value && index <= endIndexSecondRide.value) {
      rowStyle.backgroundColor = '#f0f5ff'
    }
    if (index === startIndexSecondRide.value || index === endIndexSecondRide.value) {
      rowStyle.color = '#1677ff'
    }
    return {
      ...station,
      rowStyle,
    }
  })
})
// 处理座位信息
const sortedSeatInfoFirstRide = computed(() => {
  return Object.values(props.content.first_ride.seatInfo).sort(
    (a: seatTypeInfo, b: seatTypeInfo) => {
      return a.seatType.localeCompare(b.seatType)
    },
  )
})
const sortedSeatInfoSecondRide = computed(() => {
  return Object.values(props.content.second_ride.seatInfo).sort(
    (a: seatTypeInfo, b: seatTypeInfo) => {
      return a.seatType.localeCompare(b.seatType)
    },
  )
})
// -------------------- 订票按钮逻辑 --------------------
function onClickBookTicket() {
  // 设置预订车次信息
  ticketServiceStore.preOrderSchedule = props.content
  // 跳转到订单页面
  router.push('/trainTransaction')
}
</script>

<style lang="scss" scoped>
.indirect-schedule-info-card {
  padding: 1rem;
  border: 1px solid rgba(0, 0, 0, 0.06);
  border-radius: 16px;
  background: linear-gradient(135deg, #f8fafc 0%, #ffffff 100%);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.08);
  transition: all 0.3s ease;
  
  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 12px 35px rgba(0, 0, 0, 0.12);
  }
}

.indirect-root {
  display: flex;
  flex-direction: row;
  align-items: flex-start;
  gap: 1.5rem;
}

.indirect-main-info {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
  flex: 1;
}

.indirect-schedule-info {
  display: flex;
  flex-direction: row;
  gap: 0.6rem;
  align-items: center;
  text-align: center;
}

.departure-info, .arrival-info-main {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.schedule-time {
  font-size: 1.6rem;
  font-weight: 700;
  margin-bottom: 2px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  letter-spacing: 0.5px;
}

.schedule-station {
  font-size: 0.9rem;
  color: #5a6c7d;
  font-weight: 500;
  letter-spacing: 0.3px;
}

.schedule-process {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 90px;
  gap: 4px;
}

.train-number {
  font-size: 1rem;
  font-weight: 700;
  background: linear-gradient(135deg, #3498db 0%, #2ecc71 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  cursor: pointer;
  transition: all 0.2s ease;
  padding: 2px 6px;
  border-radius: 8px;
  
  &:hover {
    background: linear-gradient(135deg, #2980b9 0%, #27ae60 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    transform: scale(1.05);
  }
  
  &.small {
    font-size: 0.85rem;
  }
}

.schedule-arrow {
  display: flex;
  align-items: center;
  margin: 2px 0;
}

.arrow-line {
  display: flex;
  align-items: center;
  width: 70px;
  
  .line {
    flex: 1;
    height: 2px;
    background: linear-gradient(90deg, #3498db, #2ecc71);
    border-radius: 1px;
  }
  
  .arrow-icon {
    width: 14px;
    height: 14px;
    color: #2ecc71;
    margin-left: 4px;
  }
}

.travel-time {
  font-size: 0.75rem;
  color: #95a5a6;
  font-weight: 500;
  letter-spacing: 0.2px;
}

.transfer-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  width: 100px;
  gap: 2px;
}

.travel-time-total {
  font-size: 0.8rem;
  color: #7f8c8d;
  font-weight: 600;
  letter-spacing: 0.2px;
}

.transfer-station {
  font-size: 1.1rem;
  font-weight: 700;
  background: linear-gradient(135deg, #e67e22 0%, #f39c12 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  letter-spacing: 0.3px;
  margin: 2px 0;
}

.transfer-time {
  font-size: 0.75rem;
  color: #95a5a6;
  font-weight: 500;
  letter-spacing: 0.1px;
}

.arrival-info {
  display: flex;
  flex-direction: row;
  align-items: flex-start;
}

.over-date-flag {
  margin-left: 6px;
  padding-top: 2px;
}

.over-date-badge {
  background: linear-gradient(135deg, #ff6b6b, #ee5a52);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  font-size: 0.8rem;
  font-weight: 700;
  padding: 1px 4px;
  border-radius: 12px;
  border: 1px solid rgba(255, 107, 107, 0.3);
}

.ticket-info {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
  padding: 0.8rem;
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.8) 0%, rgba(248, 250, 252, 0.8) 100%);
  border-radius: 12px;
  border: 1px solid rgba(0, 0, 0, 0.04);
}

.ticket-info-ride {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.ride-title {
  font-size: 0.9rem;
  font-weight: 700;
  background: linear-gradient(135deg, #3498db 0%, #2980b9 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  letter-spacing: 0.3px;
}

.seat-cards {
  display: flex;
  flex-direction: row;
  gap: 0.8rem;
  flex-wrap: wrap;
}

.seat-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 6px 10px;
  border-radius: 10px;
  background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%);
  border: 1px solid rgba(0, 0, 0, 0.05);
  transition: all 0.2s ease;
  min-width: 60px;
  
  &:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  }
}

.seat-type-info {
  font-size: 0.8rem;
  font-weight: 600;
  color: #2c3e50;
  margin-bottom: 2px;
  letter-spacing: 0.2px;
}

.remain-count-info {
  font-size: 0.75rem;
  font-weight: 600;
  padding: 2px 5px;
  border-radius: 6px;
  letter-spacing: 0.1px;
  
  &.rich {
    background: linear-gradient(135deg, rgba(39, 174, 96, 0.1), rgba(46, 204, 113, 0.1));
    color: #27ae60;
    border: 1px solid rgba(39, 174, 96, 0.2);
  }
  
  &.few {
    background: linear-gradient(135deg, rgba(52, 73, 94, 0.1), rgba(44, 62, 80, 0.1));
    color: #34495e;
    border: 1px solid rgba(52, 73, 94, 0.2);
  }
  
  &.little {
    background: linear-gradient(135deg, rgba(231, 76, 60, 0.1), rgba(192, 57, 43, 0.1));
    color: #e74c3c;
    border: 1px solid rgba(231, 76, 60, 0.2);
  }
  
  &.none {
    background: linear-gradient(135deg, rgba(149, 165, 166, 0.1), rgba(127, 140, 141, 0.1));
    color: #95a5a6;
    border: 1px solid rgba(149, 165, 166, 0.2);
  }
}

.function-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.8rem;
  margin-left: auto;
}

.total-price {
  font-size: 1.3rem;
  font-weight: 700;
  background: linear-gradient(135deg, #f39c12, #e67e22);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  letter-spacing: 0.5px;
}

.book-btn {
  padding: 8px 24px;
  height: auto;
  font-size: 0.9rem;
  font-weight: 600;
  border-radius: 12px;
  background: linear-gradient(135deg, #3498db 0%, #2980b9 100%);
  border: none;
  box-shadow: 0 4px 15px rgba(52, 152, 219, 0.3);
  transition: all 0.3s ease;
  letter-spacing: 0.5px;
  
  &:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(52, 152, 219, 0.4);
    background: linear-gradient(135deg, #2980b9 0%, #3498db 100%);
  }
  
  &:disabled {
    background: linear-gradient(135deg, #bdc3c7 0%, #95a5a6 100%);
    box-shadow: none;
    cursor: not-allowed;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .indirect-schedule-info-card {
    padding: 0.8rem;
  }
  
  .indirect-root {
    gap: 1rem;
    flex-direction: column;
  }
  
  .indirect-schedule-info {
    gap: 0.4rem;
  }
  
  .schedule-process {
    width: 70px;
  }
  
  .arrow-line {
    width: 50px;
  }
  
  .transfer-info {
    width: 80px;
  }
  
  .ticket-info {
    padding: 0.6rem;
  }
  
  .seat-cards {
    gap: 0.5rem;
  }
  
  .seat-card {
    min-width: 50px;
    padding: 4px 8px;
  }
  
  .function-area {
    flex-direction: row;
    justify-content: space-between;
    width: 100%;
  }
}
</style>

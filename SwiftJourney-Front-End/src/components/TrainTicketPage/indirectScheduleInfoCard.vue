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
              <img src="@/assets/TicketArrowGrey.svg" />
            </div>
            <!-- 到达时间 -->
            <div class="travel-time">{{ formatTime(content.first_ride.arrivalTime) }} 到达</div>
          </div>
          <!-- 中转信息 -->
          <div>
            <!-- 全程运行时间 -->
            <div class="travel-time">
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
            <div class="schedule-station-mid">{{ content.first_ride.arrivalStation }}</div>
            <!-- 换乘时间 -->
            <div class="travel-time">中转换乘 {{ formatTravelTime(content.relaxing_time) }}</div>
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
              <img src="@/assets/TicketArrowGrey.svg" />
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
              <div v-if="overDateFlag">+{{ overDateNum }}</div>
            </div>
          </div>
        </div>
        <!-- 车票信息 -->
        <div class="ticket-info">
          <!-- 第 1 程 -->
          <div class="ticket-info-ride">
            <div class="ride-title">第1程</div>
            <!-- 车票信息 -->
            <div
              v-for="(seatInfo, index) in sortedSeatInfoFirstRide"
              :key="index"
              class="seat-info"
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
          <!-- 第 2 程 -->
          <div class="ticket-info-ride">
            <div class="ride-title">第2程</div>
            <!-- 车票信息 -->
            <div
              v-for="(seatInfo, index) in sortedSeatInfoSecondRide"
              :key="index"
              class="seat-info"
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
      <!-- 功能区 -->
      <div class="function-area">
        <!-- 最低价格 -->
        <div class="price-info">SC {{ content.first_ride.price + content.second_ride.price }}</div>
        <a-button :disable="!checkBookable" type="primary" class="book-btn">订票</a-button>
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
</script>

<style lang="css" scoped>
.indirect-schedule-info-card {
  padding: 1rem;
  border: 1px solid #e0e0e0;
  border-radius: 0.5rem;
  background-color: #fff;
  box-shadow: 0 1px 8px rgba(0, 0, 0, 0.1);
}

.indirect-root {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 1rem;
}

.indirect-main-info {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.indirect-schedule-info {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
  align-items: center;
  text-align: center;
  margin-right: 0.5rem;
}

.schedule-time {
  font-size: 2rem;
  font-weight: bold;
  color: #333;
  margin-top: -0.3rem;
}

.schedule-station {
  font-size: 1rem;
  margin-top: -0.3rem;
  color: #333;
}

.schedule-station-mid {
  font-size: 1.2rem;
  font-weight: bold;
  color: #333;
  text-align: center;
  margin-top: -0.2rem;
  margin-bottom: -0.2rem;
}

.schedule-process {
  display: flex;
  flex-direction: column;
  width: 100px;
}

.train-number {
  font-size: 1.2rem;
  font-weight: bold;
  color: #555;
}
.train-number:hover {
  color: #1677ff;
  cursor: pointer;
}
.train-number.small {
  font-size: 0.9rem;
}

.travel-time {
  font-size: 0.8rem;
  color: #888;
}

.schedule-arrow {
  display: flex;
  align-items: center;
  margin-top: -0.05rem;
  margin-bottom: 0.05rem;
}

.arrival-info {
  display: flex;
  flex-direction: row;
}

.arrival-info-main {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.over-date-flag {
  padding-top: 0.1rem;
  color: #1677ff;
  font-size: 0.9rem;
  font-weight: bold;
  text-align: left;
  width: 25px;
}

.ticket-info {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.ticket-info-ride {
  display: flex;
  flex-direction: row;
  gap: 1rem;
  align-items: center;
}

.ride-title {
  font-size: 1rem;
  font-weight: bold;
  color: #1677ff;
}

.seat-info {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 0.5rem;
}

.seat-type-info {
  font-size: 0.9rem;
  font-weight: bold;
  color: #333;
}

.remain-count-info {
  font-size: 1rem;
}

.remain-count-info.rich {
  color: #45b787;
  font-weight: bold;
}
.remain-count-info.few {
  color: #333;
}
.remain-count-info.little {
  color: #ff4d4f;
  font-weight: bold;
}
.remain-count-info.none {
  color: #bbb;
}

.function-area {
  display: flex;
  align-items: center;
  flex-direction: column;
  position: right;
  margin-left: auto;
  gap: 1.3rem;
}

.price-info {
  font-size: 1.4rem;
  font-weight: bold;
  color: #f94d00;
}

.book-btn {
  display: flex;
  align-items: center;
  position: right;
  margin-left: auto;
}
</style>

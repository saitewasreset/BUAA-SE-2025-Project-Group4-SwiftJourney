<template>
  <div class="direct-schedule-info-card">
    <div class="direct-root">
      <!-- 车次信息 -->
      <div class="direct-schedule-info">
        <!-- 出发站信息 -->
        <div class="departure-info">
          <!-- 时间 -->
          <div class="schedule-time">{{ formatTime(content.departureTime) }}</div>
          <!-- 站名 -->
          <div class="schedule-station">{{ content.departureStation }}</div>
        </div>
        <!-- 过程信息 -->
        <div class="schedule-process">
          <!-- 车次信息 -->
          <a-popover :title="content.trainNumber + ' 车次信息'" trigger="click">
            <template #content>
              <a-table
                size="small"
                :columns="columns"
                :data-source="processedRoute"
                :pagination="false"
                :customRow="handleCustomRow"
                :scroll="{ y: 200 }"
                style="width: 400px"
              />
            </template>
            <div class="train-number" :class="{ small: content.trainNumber.length > 5 }">
              {{ content.trainNumber }}
            </div>
          </a-popover>
          <!-- 箭头 -->
          <div class="schedule-arrow">
            <img src="@/assets/TicketArrowGrey.svg" />
          </div>
          <!-- 运行时间 -->
          <div class="travel-time">{{ formatTravelTime(content.travelTime) }}</div>
        </div>
        <!-- 到达站信息 -->
        <div class="arrival-info">
          <div class="arrival-info-main">
            <!-- 时间 -->
            <div class="schedule-time">{{ formatTime(content.arrivalTime) }}</div>
            <!-- 站名 -->
            <div class="schedule-station">{{ content.arrivalStation }}</div>
          </div>
          <!-- 过夜标志 -->
          <div class="over-date-flag">
            <div v-if="overDateFlag">+{{ overDateNum }}</div>
          </div>
        </div>
      </div>
      <!-- 车票信息 -->
      <div class="ticket-info">
        <!-- 座位 - 车票元素 -->
        <div v-for="(seatInfo, index) in sortedSeatInfo" :key="index">
          <!-- 座位类型 -->
          <div class="seat-type-info">{{ seatInfo.seatType }}</div>
          <!-- 价格 -->
          <div class="price-info">SC {{ seatInfo.price }}</div>
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
      <!-- 功能区 -->
      <div class="function-area">
        <a-button :disable="!checkBookable" type="primary" class="book-btn">订票</a-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type {
  directScheduleInfo,
  stoppingStationInfo,
  seatTypeInfo,
} from '@/interface/ticketServiceInterface'

// -------------------- 接口数据类型 --------------------
const props = withDefaults(
  defineProps<{
    content?: directScheduleInfo
  }>(),
  {
    content: () => ({
      departureStation: '加载中..',
      departureTime: '2000-01-01T08:00:00+08:00',
      arrivalStation: '加载中..',
      arrivalTime: '2000-01-02T17:00:00+08:00',
      originStation: '加载中..',
      originDepartureTime: '2000-01-01T00:00:00+08:00',
      terminalStation: '加载中..',
      terminalArrivalTime: '2000-01-02T23:00:00+08:00',
      trainNumber: 'D9999',
      travelTime: 33 * 60 * 60,
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
          departureTime: '2000-01-02T18:00:00+08:00',
          arrivalTime: '2000-01-02T17:00:00+08:00',
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
  const departureDate = new Date(props.content.departureTime)
  const overNum = date.getDate() - departureDate.getDate()
  if (overNum > 0) {
    return `${time} +${overNum}`
  }
  return time
}
// -------------------- 过夜标志逻辑 --------------------
const overDateNum = computed(() => {
  const departureDate = new Date(props.content.departureTime)
  const arrivalDate = new Date(props.content.arrivalTime)
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
  // 检查是否有任何座位类型还有余票（left > 0）
  return Object.values(props.content.seatInfo).some((seatInfo) => seatInfo.left > 0)
})
// -------------------- 车次信息表格 --------------------
// 应用行样式
const handleCustomRow = (record: any) => ({
  style: record.rowStyle,
})
// 按时间顺序排序车次信息
const sortedRoute = computed(() => {
  return props.content.route.sort((a, b) => {
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
const startIndex = computed(() => {
  return sortedRoute.value.findIndex(
    (station) =>
      station.stationName === props.content.departureStation &&
      station.departureTime === props.content.departureTime,
  )
})
const endIndex = computed(() => {
  return sortedRoute.value.findIndex(
    (station) =>
      station.stationName === props.content.arrivalStation &&
      station.arrivalTime === props.content.arrivalTime,
  )
})
// 处理车次信息
const processedRoute = computed(() => {
  return sortedRoute.value.map((station, index) => {
    const rowStyle: any = {}
    if (index >= startIndex.value && index <= endIndex.value) {
      rowStyle.backgroundColor = '#f0f5ff'
    }
    if (index === startIndex.value || index === endIndex.value) {
      rowStyle.color = '#1677ff'
    }
    return {
      ...station,
      rowStyle,
    }
  })
})
// 处理座位信息
const sortedSeatInfo = computed(() => {
  return Object.values(props.content.seatInfo).sort((a: seatTypeInfo, b: seatTypeInfo) => {
    return a.seatType.localeCompare(b.seatType)
  })
})
</script>

<style lang="css" scoped>
.direct-schedule-info-card {
  padding: 1rem;
  border: 1px solid #e0e0e0;
  border-radius: 0.5rem;
  background-color: #fff;
  box-shadow: 0 1px 8px rgba(0, 0, 0, 0.1);
}

.direct-root {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 1rem;
}

.direct-schedule-info {
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
  flex-direction: row;
  gap: 1rem;
}

.seat-type-info {
  font-size: 0.9rem;
  font-weight: bold;
  color: #333;
}

.price-info {
  font-size: 0.7rem;
  color: #f94d00;
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
  position: right;
  margin-left: auto;
}
</style>

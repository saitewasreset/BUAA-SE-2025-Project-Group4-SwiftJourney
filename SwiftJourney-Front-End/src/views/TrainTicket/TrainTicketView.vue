<template>
  <div class="root">
    <div>
      <TicketSearch />
    </div>
    <div class="train-ticket">
      <div>
        <TrainFilter />
      </div>
      <div>
        <div class="right-top-area">
          <a-radio-group v-model:value="selectedDate" class="date-picker">
            <a-radio-button
              v-for="(day, index) in ticketServiceStore.dateRange"
              :key="index"
              :value="day.date"
              class="date-btn"
            >
              {{ day.display }}
            </a-radio-button>
          </a-radio-group>
          <div class="sort-part">
            <div
              class="sort-item"
              :class="{ 'sort-item-active': ticketServiceStore.isSortByDepartureTime }"
              @click="ticketServiceStore.toggleSortType(SortType.DepartureTime)"
            >
              出发时间
              <div v-if="ticketServiceStore.isSortByDepartureTime">
                {{ ticketServiceStore.sortOrderAsc ? '早 - 晚' : '晚 - 早' }}
              </div>
            </div>
            <div
              class="sort-item"
              :class="{ 'sort-item-active': ticketServiceStore.isSortByTravelTime }"
              @click="ticketServiceStore.toggleSortType(SortType.TravelTime)"
            >
              运行时长
              <div v-if="ticketServiceStore.isSortByTravelTime">
                {{ ticketServiceStore.sortOrderAsc ? '短 - 长' : '长 - 短' }}
              </div>
            </div>
            <div
              class="sort-item"
              :class="{ 'sort-item-active': ticketServiceStore.isSortByPrice }"
              @click="ticketServiceStore.toggleSortType(SortType.Price)"
            >
              价格排序
              <div v-if="ticketServiceStore.isSortByPrice">
                {{ ticketServiceStore.sortOrderAsc ? '低 - 高' : '高 - 低' }}
              </div>
            </div>
          </div>
        </div>
        <div v-if="ticketServiceStore.queryMode === 'direct'" class="schedule-card">
          <div v-for="(item, index) in directResults" :key="index">
            <directScheduleInfoCard :content="item" />
          </div>
        </div>
        <div v-if="ticketServiceStore.queryMode === 'indirect'" class="schedule-card">
          <div v-for="(item, index) in indirectResults" :key="index">
            <indirectScheduleInfoCard :content="item" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import TicketSearch from '@/components/TicketSearch/TicketSearch.vue'
import TrainFilter from '@/components/TrainTicketPage/TrainFilter.vue'
import directScheduleInfoCard from '@/components/TrainTicketPage/directScheduleInfoCard.vue'
import indirectScheduleInfoCard from '@/components/TrainTicketPage/indirectScheduleInfoCard.vue'
import { computed, onMounted } from 'vue'
import { useTicketServiceStore } from '@/stores/ticketService'
import {
  SortType,
  type directScheduleInfo,
  type indirectScheduleInfo,
} from '@/interface/ticketServiceInterface'
import { TicketServiceApi } from '@/api/TicketServiceApi/TicketServiceApi'

const ticketServiceStore = useTicketServiceStore()

// -------------------- 日期相关 --------------------
const selectedDate = computed({
  get: () => ticketServiceStore.queryDate,
  set: (value: string) => {
    ticketServiceStore.queryDate = value
  },
})

// -------------------- 查询相关 --------------------
// 类型安全的计算属性
const directResults = computed(() => {
  if (ticketServiceStore.queryMode === 'direct') {
    return ticketServiceStore.displaySchedules as directScheduleInfo[]
  }
  return []
})

const indirectResults = computed(() => {
  if (ticketServiceStore.queryMode === 'indirect') {
    return ticketServiceStore.displaySchedules as indirectScheduleInfo[]
  }
  return []
})
// only for test
onMounted(async () => {
  console.log('组件已挂载，开始查询车次信息...', ticketServiceStore.queryDate)
  // ticketServiceStore.queryMode = 'indirect'
  const response = await TicketServiceApi.queryDirectSchedule({
    departureDate: ticketServiceStore.queryDate,
    departureCity: '天津市',
    arrivalCity: '南京市',
  })
  console.log('查询结果:', response.data)
  ticketServiceStore.handleResponse(response.data)
})
</script>

<style lang="css" scoped>
.root {
  display: block;
}

.train-ticket {
  margin: 0 auto;
  max-width: 1200px;
  display: grid;
  grid-template-columns: 2fr 4fr;
  grid-template-rows: auto;
  gap: 20px;
  padding: 20px;
}

.right-top-area {
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.15);
}

.date-picker {
  display: flex;
  justify-content: space-between;
}

.date-btn {
  flex: 1 1 0;
  text-align: center;
  white-space: nowrap; /* 禁止文字换行 */
  padding-inline: 0;
  border-end-start-radius: 0;
  border-end-end-radius: 0;
}

.date-btn.ant-radio-button-wrapper-checked {
  background-color: #eaf2fc;
}

.sort-part {
  display: flex;
  justify-content: space-around;
  padding: 0.3rem 1rem;
  border: 1px solid #d9d9d9;
  font-size: 0.9rem;
  border-end-start-radius: 6px;
  border-end-end-radius: 6px;
  margin-bottom: 1rem;
  color: #333;
}

.sort-item {
  display: flex;
  flex-direction: row;
  justify-content: center;
  gap: 0.5rem;
  width: 30%;
  user-select: none;
}

.sort-item.sort-item-active {
  color: #1677ff;
  font-weight: bold;
}

.schedule-card {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
</style>

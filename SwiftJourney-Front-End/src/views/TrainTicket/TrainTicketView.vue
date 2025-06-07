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
import { computed, watch } from 'vue'
import { useTicketServiceStore } from '@/stores/ticketService'
import {
  SortType,
  type directScheduleInfo,
  type indirectScheduleInfo,
} from '@/interface/ticketServiceInterface'

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

// -------------------- 监听查询模式和日期变化 --------------------
watch(
  () => ticketServiceStore.queryMode,
  async (newMode) => {
    await ticketServiceStore.querySchedule()
  },
)

watch(
  () => ticketServiceStore.queryDate,
  async (newDate) => {
    await ticketServiceStore.querySchedule()
  },
)
</script>

<style lang="scss" scoped>
.root {
  min-height: 100vh;
  background: linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 25%, #fef3c7 75%, #fef7cd 100%);
  position: relative;
  
  &::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: 
      radial-gradient(circle at 20% 80%, rgba(120, 119, 198, 0.1) 0%, transparent 50%),
      radial-gradient(circle at 80% 20%, rgba(255, 119, 198, 0.1) 0%, transparent 50%),
      radial-gradient(circle at 40% 40%, rgba(120, 219, 226, 0.1) 0%, transparent 50%);
    pointer-events: none;
  }
}

.train-ticket {
  position: relative;
  z-index: 1;
  margin: 0 auto;
  max-width: 1200px;
  display: grid;
  grid-template-columns: 2fr 4fr;
  grid-template-rows: auto;
  gap: 20px;
  padding: 20px;
}

.right-top-area {
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(20px);
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.6);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
  overflow: hidden;
}

.date-picker {
  display: flex;
  justify-content: space-between;
}

.date-btn {
  flex: 1 1 0;
  text-align: center;
  white-space: nowrap;
  padding-inline: 0;
  border-end-start-radius: 0;
  border-end-end-radius: 0;
  border: none;
  background: transparent;
  
  &:hover {
    background: rgba(59, 130, 246, 0.05);
  }
}

.date-btn.ant-radio-button-wrapper-checked {
  background: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
  font-weight: 600;
}

.sort-part {
  display: flex;
  justify-content: space-around;
  padding: 16px 20px;
  font-size: 0.9rem;
  color: #64748b;
  background: rgba(248, 250, 252, 0.8);
}

.sort-item {
  display: flex;
  flex-direction: row;
  justify-content: center;
  gap: 0.5rem;
  width: 30%;
  user-select: none;
  cursor: pointer;
  padding: 8px 12px;
  border-radius: 8px;
  transition: all 0.3s ease;
  
  &:hover {
    background: rgba(59, 130, 246, 0.08);
    color: #3b82f6;
  }
  
  &.sort-item-active {
    color: #3b82f6;
    font-weight: 700;
    background: rgba(59, 130, 246, 0.1);
  }
}

.schedule-card {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 0 4px;
}

// 响应式设计
@media (max-width: 1024px) {
  .train-ticket {
    grid-template-columns: 1fr;
    max-width: 800px;
  }
}

@media (max-width: 768px) {
  .train-ticket {
    padding: 16px;
    gap: 16px;
  }
  
  .sort-part {
    flex-direction: column;
    gap: 8px;
  }
  
  .sort-item {
    width: 100%;
    justify-content: flex-start;
  }
}
</style>

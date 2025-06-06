<template>
  <div class="root">
    <div>search</div>
    <div class="train-ticket">
      <TrainFilter />
      <div>
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
        <div>
          <directScheduleInfo />
          <indirectScheduleInfoCard />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import TrainFilter from '@/components/TrainTicketPage/TrainFilter.vue'
import directScheduleInfo from '@/components/TrainTicketPage/directScheduleInfoCard.vue'
import indirectScheduleInfoCard from '@/components/TrainTicketPage/indirectScheduleInfoCard.vue'
import { computed } from 'vue'
import { useTicketServiceStore } from '@/stores/ticketService'
import { SortType } from '@/interface/ticketServiceInterface'

const ticketServiceStore = useTicketServiceStore()

// -------------------- 日期相关 --------------------
const selectedDate = computed({
  get: () => ticketServiceStore.queryDate,
  set: (value: string) => {
    ticketServiceStore.queryDate = value
  },
})
</script>

<style lang="css" scoped>
.root {
  display: block;
}

.train-ticket {
  display: grid;
  grid-template-columns: 2fr 4fr;
  grid-template-rows: auto;
  gap: 20px;
  padding: 20px 0;
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
  width: 20%;
}

.sort-item.sort-item-active {
  color: #1677ff;
  font-weight: bold;
}
</style>

<template>
  <a-card class="schedule-filter-card">
    <div class="line">
      <p>仅显示有票车次</p>
      <a-switch v-model:checked="onlyShowAvailable" />
    </div>
    <a-divider />
    <div class="type">
      <div class="line">
        <p>车次类型</p>
        <a-checkbox
          class="check-all"
          :checked="checkGroups[CheckType.TrainType].checkAll"
          @change="ticketServiceStore.onCheckAllBoxChange(CheckType.TrainType)"
          :indeterminate="checkGroups[CheckType.TrainType].indeterminate"
        >
          全选
        </a-checkbox>
      </div>
      <a-checkbox-group
        v-model:value="checkGroups[CheckType.TrainType].checkedList"
        :options="checkGroups[CheckType.TrainType].options"
      />
    </div>
    <a-divider />
    <div class="type">
      <div class="line">
        <p>车次席别</p>
        <a-checkbox
          class="check-all"
          :checked="checkGroups[CheckType.SeatType].checkAll"
          @change="ticketServiceStore.onCheckAllBoxChange(CheckType.SeatType)"
          :indeterminate="checkGroups[CheckType.SeatType].indeterminate"
        >
          全选
        </a-checkbox>
      </div>
      <a-checkbox-group
        v-model:value="checkGroups[CheckType.SeatType].checkedList"
        :options="checkGroups[CheckType.SeatType].options"
      />
    </div>
    <a-divider />
    <div class="type">
      <div class="line">
        <p>出发时间</p>
        <a-time-range-picker
          v-model:value="startTimeRange"
          format="HH:mm"
          :minute-step="10"
          @change="(value: [Dayjs, Dayjs]) => ticketServiceStore.onTimePickerChange('start', value)"
          class="time-picker"
        >
          <template #suffixIcon></template>
        </a-time-range-picker>
        <a-button class="reset-button" @click="ticketServiceStore.resetTimeRange('start')"
          >重置</a-button
        >
      </div>
      <a-slider
        range
        v-model:value="startTimeRangeNumber"
        :min="0"
        :max="1439"
        :tipFormatter="TimeFormatter"
        @change="(value: [number, number]) => ticketServiceStore.onSliderChange('start', value)"
        :marks="{
          0: '00:00',
          360: '06:00',
          720: '12:00',
          1080: '18:00',
          1439: '23:59',
        }"
      />
    </div>
    <a-divider />
    <div class="type">
      <div class="line">
        <p>到达时间</p>
        <a-time-range-picker
          v-model:value="endTimeRange"
          format="HH:mm"
          :minute-step="10"
          @change="(value: [Dayjs, Dayjs]) => ticketServiceStore.onTimePickerChange('end', value)"
          class="time-picker"
        >
          <template #suffixIcon></template>
        </a-time-range-picker>
        <a-button class="reset-button" @click="ticketServiceStore.resetTimeRange('end')">重置</a-button>
      </div>
      <a-slider
        range
        v-model:value="endTimeRangeNumber"
        :min="0"
        :max="1439"
        :tipFormatter="TimeFormatter"
        @change="(value: [number, number]) => ticketServiceStore.onSliderChange('end', value)"
        :marks="{
          0: '00:00',
          360: '06:00',
          720: '12:00',
          1080: '18:00',
          1439: '23:59',
        }"
      />
    </div>
    <div v-if="showMoreFilter">
      <a-divider />
      <div class="type">
        <div class="line">
          <p>出发车站</p>
          <a-checkbox
            class="check-all"
            :checked="checkGroups[CheckType.DepartureStation].checkAll"
            @change="ticketServiceStore.onCheckAllBoxChange(CheckType.DepartureStation)"
            :indeterminate="checkGroups[CheckType.DepartureStation].indeterminate"
          >
            全选
          </a-checkbox>
        </div>
        <a-checkbox-group
          v-model:value="checkGroups[CheckType.DepartureStation].checkedList"
          :options="checkGroups[CheckType.DepartureStation].options"
        />
      </div>
      <a-divider />
      <div class="type">
        <div class="line">
          <p>中转车站</p>
          <a-checkbox
            class="check-all"
            :checked="checkGroups[CheckType.TransferStation].checkAll"
            @change="ticketServiceStore.onCheckAllBoxChange(CheckType.TransferStation)"
            :indeterminate="checkGroups[CheckType.TransferStation].indeterminate"
          >
            全选
          </a-checkbox>
        </div>
        <a-checkbox-group
          v-model:value="checkGroups[CheckType.TransferStation].checkedList"
          :options="checkGroups[CheckType.TransferStation].options"
        />
      </div>
      <a-divider />
      <div class="type">
        <div class="line">
          <p>到达车站</p>
          <a-checkbox
            class="check-all"
            :checked="checkGroups[CheckType.ArrivalStation].checkAll"
            @change="ticketServiceStore.onCheckAllBoxChange(CheckType.ArrivalStation)"
            :indeterminate="checkGroups[CheckType.ArrivalStation].indeterminate"
          >
            全选
          </a-checkbox>
        </div>
        <a-checkbox-group
          v-model:value="checkGroups[CheckType.ArrivalStation].checkedList"
          :options="checkGroups[CheckType.ArrivalStation].options"
        />
      </div>
    </div>
    <div class="card-bottom">
      <a-button @click="showMoreFilter = !showMoreFilter">
        <div v-if="!showMoreFilter" class="more-filter-btn">
          <DownOutlined />
          <span>展开更多筛选</span>
        </div>
        <div v-else class="more-filter-btn">
          <UpOutlined />
          <span>收起部分筛选</span>
        </div>
      </a-button>
    </div>
  </a-card>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { DownOutlined, UpOutlined } from '@ant-design/icons-vue'
import dayjs, { Dayjs } from 'dayjs'
import { CheckType } from '@/interface/ticketServiceInterface'
import { useTicketServiceStore } from '@/stores/ticketService'

const ticketServiceStore = useTicketServiceStore()

// -------------------- 筛选 --------------------
// -------------------- 变量定义 --------------------

// 是否显示更多筛选
const showMoreFilter = ref<boolean>(false)

// 只显示有票的车次
const onlyShowAvailable = computed({
  get: () => ticketServiceStore.onlyShowAvailable,
  set: (value: boolean) => {
    ticketServiceStore.onlyShowAvailable = value
  },
})

// -------------------- 多选列表 --------------------
const checkGroups = computed(() => ticketServiceStore.checkGroups)

// 多选列表监听
checkGroups.value.forEach((checkGroup) => {
  watch(
    () => checkGroup.checkedList,
    (newVal) => {
      checkGroup.checkAll = newVal.length === checkGroup.options.length
      checkGroup.indeterminate = !!newVal.length && newVal.length < checkGroup.options.length
    },
  )
})

// -------------------- 时间选择 --------------------
const startTimeRange = computed({
  get: () => ticketServiceStore.startTimeRange,
  set: (value: [Dayjs, Dayjs]) => {
    ticketServiceStore.startTimeRange = value
  },
})
const endTimeRange = computed({
  get: () => ticketServiceStore.endTimeRange,
  set: (value: [Dayjs, Dayjs]) => {
    ticketServiceStore.endTimeRange = value
  },
})
const startTimeRangeNumber = computed({
  get: () => ticketServiceStore.startTimeRangeNumber,
  set: (value: [number, number]) => {
    ticketServiceStore.startTimeRangeNumber = value
  },
})
const endTimeRangeNumber = computed({
  get: () => ticketServiceStore.endTimeRangeNumber,
  set: (value: [number, number]) => {
    ticketServiceStore.endTimeRangeNumber = value
  },
})
const TimeFormatter = (value: number) => {
  const hour = Math.floor(value / 60)
  const minute = value % 60
  return `${hour.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')}`
}
</script>

<style lang="css" scoped>
.schedule-filter-card {
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.15);
}

.line {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  margin-bottom: 1rem;
  p {
    margin-bottom: 0px;
    font-weight: bold;
    font-size: 1rem;
    color: #333;
  }
  .check-all {
    position: right;
    margin-left: auto;
  }
}

.time-picker {
  width: 55%;
}

.reset-button {
  position: right;
  margin-left: auto;
}

.card-bottom {
  display: flex;
  justify-content: center;
  margin-top: 1rem;
}

.more-filter-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
</style>

import { defineStore } from 'pinia'
import type { CheckBoxGroup } from '@/interface/ticketServiceInterface'
import { SortType } from '@/interface/ticketServiceInterface'
import dayjs, { Dayjs } from 'dayjs'
import { useUserStore } from './user'

const userStore = useUserStore()

export const useTicketServiceStore = defineStore('ticketService', {
  state: () => ({
    // -------------------- 筛选相关 --------------------
    // 只显示有票的车次
    onlyShowAvailable: false,
    checkGroups: [
      // 车次类型
      {
        options: ['G/C', 'D', 'Z', 'T', 'K', '其他'],
        checkedList: ['G/C', 'D', 'Z', 'T', 'K', '其他'],
        indeterminate: false,
        checkAll: true,
      },
      // 座位类型
      {
        options: ['加载中...'],
        checkedList: ['加载中...'],
        indeterminate: false,
        checkAll: true,
      },
      // 出发站
      {
        options: ['加载中...'],
        checkedList: ['加载中...'],
        indeterminate: false,
        checkAll: true,
      },
      // 中转站
      {
        options: ['加载中...'],
        checkedList: ['加载中...'],
        indeterminate: false,
        checkAll: true,
      },
      // 到达站
      {
        options: ['加载中...'],
        checkedList: ['加载中...'],
        indeterminate: false,
        checkAll: true,
      },
    ] as CheckBoxGroup[],
    // 时间
    startTimeRange: [dayjs('00:00', 'HH:mm'), dayjs('23:59', 'HH:mm')] as [Dayjs, Dayjs],
    endTimeRange: [dayjs('00:00', 'HH:mm'), dayjs('23:59', 'HH:mm')] as [Dayjs, Dayjs],
    startTimeRangeNumber: [0, 1439] as [number, number],
    endTimeRangeNumber: [0, 1439] as [number, number],
    // -------------------- 排序相关 --------------------
    // 排序方式
    sortType: SortType.DepartureTime,
    // 排序顺序：是否升序
    // true: 升序，false: 降序
    sortOrderAsc: true,
    // -------------------- 查询相关 --------------------
    // 查询日期
    queryDate: new Date().toISOString().split('T')[0],
  }),
  getters: {
    // -------------------- 时间相关 --------------------
    // 生成14天日期范围
    dateRange() {
      const days = []
      const today = new Date()

      for (let i = 0; i < 14; i++) {
        const date = new Date()
        date.setDate(today.getDate() + i)

        days.push({
          date: date.toISOString().split('T')[0], // YYYY-MM-DD格式
          display: `${date.getMonth() + 1}-${date.getDate()}`, // M-D格式
        })
      }
      return days
    },
    // -------------------- 排序相关 --------------------
    // 判断模式
    isSortByDepartureTime(): boolean {
      return this.sortType === SortType.DepartureTime
    },
    isSortByTravelTime(): boolean {
      return this.sortType === SortType.TravelTime
    },
    isSortByPrice(): boolean {
      return this.sortType === SortType.Price
    },
  },
  actions: {
    // -------------------- 多选框更新逻辑 --------------------
    // -------------------- 全选框状态变化 --------------------
    onCheckAllBoxChange(checkType: number) {
      const group = this.checkGroups[checkType]
      if (!group) return
      group.checkAll = !group.checkAll
      group.checkedList = group.checkAll ? group.options : []
      group.indeterminate = false
    },
    // -------------------- 时间更新逻辑 --------------------
    // 时间转换逻辑
    // 分钟数值 => Dayjs 时间范围
    minutesToDayjsRange(minutes: [number, number]): [Dayjs, Dayjs] {
      const start = dayjs().startOf('day').add(minutes[0], 'minute')
      const end = dayjs().startOf('day').add(minutes[1], 'minute')
      return [start, end]
    },
    // Dayjs 时间范围 => 分钟数值
    dayjsRangeToMinutes(range: [Dayjs, Dayjs]): [number, number] {
      const start = range[0].hour() * 60 + range[0].minute()
      const end = range[1].hour() * 60 + range[1].minute()
      return [start, end]
    },
    // 滑动条更新逻辑
    onSliderChange(type: 'start' | 'end', value: [number, number]) {
      if (type === 'start') {
        this.startTimeRangeNumber = value
        this.startTimeRange = this.minutesToDayjsRange(value)
      } else if (type === 'end') {
        this.endTimeRangeNumber = value
        this.endTimeRange = this.minutesToDayjsRange(value)
      }
    },
    // 时间选择框更新逻辑
    onTimePickerChange(type: 'start' | 'end', value: [Dayjs, Dayjs]) {
      if (type === 'start') {
        this.startTimeRange = value
        this.startTimeRangeNumber = this.dayjsRangeToMinutes(value)
      } else if (type === 'end') {
        this.endTimeRange = value
        this.endTimeRangeNumber = this.dayjsRangeToMinutes(value)
      }
    },
    // 重置时间逻辑
    resetTimeRange(type: 'start' | 'end') {
      if (type === 'start') {
        this.startTimeRange = [dayjs('00:00', 'HH:mm'), dayjs('23:59', 'HH:mm')]
        this.startTimeRangeNumber = [0, 1439]
      } else if (type === 'end') {
        this.endTimeRange = [dayjs('00:00', 'HH:mm'), dayjs('23:59', 'HH:mm')]
        this.endTimeRangeNumber = [0, 1439]
      }
    },
    // ---------------------- 排序相关 --------------------
    // 切换排序方式
    toggleSortType(type: SortType) {
      if (this.sortType === type) {
        // 如果当前排序方式已选中，则切换排序顺序
        this.sortOrderAsc = !this.sortOrderAsc
      } else {
        // 否则，设置新的排序方式并默认升序
        this.sortType = type
        this.sortOrderAsc = true
      }
    },
  },
})

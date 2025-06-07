import { defineStore } from 'pinia'
import type {
  CheckBoxGroup,
  directScheduleInfo,
  indirectScheduleInfo,
  QueryMode,
  scheduleRequest,
  seatTypeInfo,
} from '@/interface/ticketServiceInterface'
import { CheckType, SortType } from '@/interface/ticketServiceInterface'
import dayjs, { Dayjs } from 'dayjs'
import { useUserStore } from './user'
import { useGeneralStore } from './general'
import { TicketServiceApi } from '@/api/TicketServiceApi/TicketServiceApi'
import { message } from 'ant-design-vue'

const generalStore = useGeneralStore()
const userStore = useUserStore()

export const useTicketServiceStore = defineStore('ticketService', {
  state: () => ({
    // -------------------- 筛选相关 --------------------
    // 只显示有票的车次
    onlyShowAvailable: false,
    checkGroups: [
      // 车次类型
      {
        options: ['G/C', 'D', 'T', 'K', 'Z', '其他'],
        checkedList: ['G/C', 'D', 'T', 'K', 'Z', '其他'],
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
    // 默认查询今天的日期
    queryDate: new Date().toISOString().split('T')[0],
    // 查询城市/站点
    queryDepartureText: '',
    queryArrivalText: '',
    // 查询模式
    queryMode: 'direct' as QueryMode,
    // 查询结果
    queryResult: [] as directScheduleInfo[] | indirectScheduleInfo[],
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
    // -------------------- 筛选结果 --------------------
    // 根据筛选条件过滤显示的车次（所有条件同时满足）
    displaySchedules(): directScheduleInfo[] | indirectScheduleInfo[] {
      let filteredSchedules = [...this.queryResult]

      // 1. 只显示有票的车次
      if (this.onlyShowAvailable) {
        if (this.queryMode === 'direct') {
          filteredSchedules = filteredSchedules.filter((schedule: any) => {
            const directSchedule = schedule as directScheduleInfo
            return Object.values(directSchedule.seatInfo).some((seatInfo) => seatInfo.left > 0)
          })
        } else if (this.queryMode === 'indirect') {
          filteredSchedules = filteredSchedules.filter((schedule: any) => {
            const indirectSchedule = schedule as indirectScheduleInfo
            // 检查第一程是否有余票（安全版本）
            const firstRideHasTicket = Object.values(indirectSchedule.first_ride.seatInfo).some(
              (seatInfo) => seatInfo.left > 0,
            )
            // 检查第二程是否有余票（安全版本）
            const secondRideHasTicket = Object.values(indirectSchedule.second_ride.seatInfo).some(
              (seatInfo) => seatInfo.left > 0,
            )
            // 只有两程都有余票才能订票
            return firstRideHasTicket && secondRideHasTicket
          })
        }
      }

      // 2. 车次类型筛选
      const trainTypeChecked = this.checkGroups[CheckType.TrainType].checkedList
      if (trainTypeChecked.length < this.checkGroups[CheckType.TrainType].options.length) {
        if (this.queryMode === 'direct') {
          filteredSchedules = filteredSchedules.filter((schedule: any) => {
            const directSchedule = schedule as directScheduleInfo
            const trainNumber = directSchedule.trainNumber
            const trainType = trainNumber.charAt(0)
            if (trainType === 'G' || trainType === 'C') {
              return trainTypeChecked.includes('G/C')
            } else if (trainType === 'D') {
              return trainTypeChecked.includes('D')
            } else if (trainType === 'T') {
              return trainTypeChecked.includes('T')
            } else if (trainType === 'K') {
              return trainTypeChecked.includes('K')
            } else if (trainType === 'Z') {
              return trainTypeChecked.includes('Z')
            } else {
              return trainTypeChecked.includes('其他')
            }
          })
        } else if (this.queryMode === 'indirect') {
          filteredSchedules = filteredSchedules.filter((schedule: any) => {
            const indirectSchedule = schedule as indirectScheduleInfo
            const firstTrainNumber = indirectSchedule.first_ride.trainNumber
            const secondTrainNumber = indirectSchedule.second_ride.trainNumber
            const firstTrainType = firstTrainNumber.charAt(0)
            const secondTrainType = secondTrainNumber.charAt(0)

            const subTrainType = ['G', 'C']
            if (subTrainType.includes(firstTrainType) && subTrainType.includes(secondTrainType)) {
              return trainTypeChecked.includes('G/C')
            }
            subTrainType.push('D')
            if (subTrainType.includes(firstTrainType) && subTrainType.includes(secondTrainType)) {
              return trainTypeChecked.includes('D')
            }
            subTrainType.push('T')
            if (subTrainType.includes(firstTrainType) && subTrainType.includes(secondTrainType)) {
              return trainTypeChecked.includes('T')
            }
            subTrainType.push('K')
            if (subTrainType.includes(firstTrainType) && subTrainType.includes(secondTrainType)) {
              return trainTypeChecked.includes('K')
            }
            subTrainType.push('Z')
            if (subTrainType.includes(firstTrainType) && subTrainType.includes(secondTrainType)) {
              return trainTypeChecked.includes('Z')
            }
            return trainTypeChecked.includes('其他')
          })
        }
      }

      // 3. 座位类型筛选
      const seatTypeChecked = this.checkGroups[CheckType.SeatType].checkedList
      if (
        seatTypeChecked.length < this.checkGroups[CheckType.SeatType].options.length &&
        !seatTypeChecked.includes('加载中...')
      ) {
        if (this.queryMode === 'direct') {
          filteredSchedules = filteredSchedules.filter((schedule: any) => {
            const directSchedule = schedule as directScheduleInfo
            return Object.keys(directSchedule.seatInfo).some((seatType) =>
              seatTypeChecked.includes(seatType),
            )
          })
        } else if (this.queryMode === 'indirect') {
          filteredSchedules = filteredSchedules.filter((schedule: any) => {
            const indirectSchedule = schedule as indirectScheduleInfo
            const firstRideSeatTypes = Object.keys(indirectSchedule.first_ride.seatInfo)
            const secondRideSeatTypes = Object.keys(indirectSchedule.second_ride.seatInfo)
            return (
              firstRideSeatTypes.some((seatType) => seatTypeChecked.includes(seatType)) ||
              secondRideSeatTypes.some((seatType) => seatTypeChecked.includes(seatType))
            )
          })
        }
      }

      // 4. 出发站筛选
      const departureStationChecked = this.checkGroups[CheckType.DepartureStation].checkedList
      if (
        departureStationChecked.length <
          this.checkGroups[CheckType.DepartureStation].options.length &&
        !departureStationChecked.includes('加载中...')
      ) {
        if (this.queryMode === 'direct') {
          filteredSchedules = filteredSchedules.filter((schedule: any) => {
            const directSchedule = schedule as directScheduleInfo
            const departureStation = directSchedule.departureStation
            return departureStationChecked.includes(departureStation)
          })
        } else if (this.queryMode === 'indirect') {
          filteredSchedules = filteredSchedules.filter((schedule: any) => {
            const indirectSchedule = schedule as indirectScheduleInfo
            const departureStation = indirectSchedule.first_ride.departureStation
            return departureStationChecked.includes(departureStation)
          })
        }
      }

      // 5. 中转站筛选（仅适用于中转车次）
      if (this.queryMode === 'indirect') {
        const transferStationChecked = this.checkGroups[CheckType.TransferStation].checkedList
        if (
          transferStationChecked.length <
            this.checkGroups[CheckType.TransferStation].options.length &&
          !transferStationChecked.includes('加载中...')
        ) {
          filteredSchedules = filteredSchedules.filter((schedule: any) => {
            const indirectSchedule = schedule as indirectScheduleInfo
            const transferStations = indirectSchedule.second_ride.departureStation
            return transferStationChecked.includes(transferStations)
          })
        }
      }

      // 6. 到达站筛选
      const arrivalStationChecked = this.checkGroups[CheckType.ArrivalStation].checkedList
      if (
        arrivalStationChecked.length < this.checkGroups[CheckType.ArrivalStation].options.length &&
        !arrivalStationChecked.includes('加载中...')
      ) {
        if (this.queryMode === 'direct') {
          filteredSchedules = filteredSchedules.filter((schedule: any) => {
            const directSchedule = schedule as directScheduleInfo
            const arrivalStation = directSchedule.arrivalStation
            return arrivalStationChecked.includes(arrivalStation)
          })
        } else if (this.queryMode === 'indirect') {
          filteredSchedules = filteredSchedules.filter((schedule: any) => {
            const indirectSchedule = schedule as indirectScheduleInfo
            const arrivalStation = indirectSchedule.second_ride.arrivalStation
            return arrivalStationChecked.includes(arrivalStation)
          })
        }
      }

      // 7. 出发时间筛选
      filteredSchedules = filteredSchedules.filter((schedule: any) => {
        if (this.queryMode === 'direct') {
          const directSchedule = schedule as directScheduleInfo
          const departureTime = directSchedule.departureTime
          const departureDate = new Date(departureTime)
          const departureTimeNumber = departureDate.getHours() * 60 + departureDate.getMinutes()
          return (
            departureTimeNumber >= this.startTimeRangeNumber[0] &&
            departureTimeNumber <= this.startTimeRangeNumber[1]
          )
        } else if (this.queryMode === 'indirect') {
          const indirectSchedule = schedule as indirectScheduleInfo
          const firstDepartureTime = indirectSchedule.first_ride.departureTime
          const firstDepartureDate = new Date(firstDepartureTime)
          const firstDepartureTimeNumber =
            firstDepartureDate.getHours() * 60 + firstDepartureDate.getMinutes()
          return (
            firstDepartureTimeNumber >= this.startTimeRangeNumber[0] &&
            firstDepartureTimeNumber <= this.startTimeRangeNumber[1]
          )
        }
      })

      // 8. 到达时间筛选
      filteredSchedules = filteredSchedules.filter((schedule: any) => {
        if (this.queryMode === 'direct') {
          const directSchedule = schedule as directScheduleInfo
          const arrivalTime = directSchedule.arrivalTime
          const arrivalDate = new Date(arrivalTime)
          const arrivalTimeNumber = arrivalDate.getHours() * 60 + arrivalDate.getMinutes()
          return (
            arrivalTimeNumber >= this.endTimeRangeNumber[0] &&
            arrivalTimeNumber <= this.endTimeRangeNumber[1]
          )
        } else if (this.queryMode === 'indirect') {
          const indirectSchedule = schedule as indirectScheduleInfo
          const secondArrivalTime = indirectSchedule.second_ride.arrivalTime
          const secondArrivalDate = new Date(secondArrivalTime)
          const secondArrivalTimeNumber =
            secondArrivalDate.getHours() * 60 + secondArrivalDate.getMinutes()
          return (
            secondArrivalTimeNumber >= this.endTimeRangeNumber[0] &&
            secondArrivalTimeNumber <= this.endTimeRangeNumber[1]
          )
        }
      })

      // 9. 排序
      filteredSchedules.sort((a: any, b: any) => {
        let compareValue = 0

        if (this.queryMode === 'direct') {
          const directA = a as directScheduleInfo
          const directB = b as directScheduleInfo

          switch (this.sortType) {
            case SortType.DepartureTime:
              compareValue =
                new Date(directA.departureTime).getTime() -
                new Date(directB.departureTime).getTime()
              break
            case SortType.TravelTime:
              compareValue = directA.travelTime - directB.travelTime
              break
            case SortType.Price:
              compareValue = directA.price - directB.price
              break
            default:
              break
          }
        } else if (this.queryMode === 'indirect') {
          const indirectA = a as indirectScheduleInfo
          const indirectB = b as indirectScheduleInfo
          switch (this.sortType) {
            case SortType.DepartureTime:
              const firstDepartureA = new Date(indirectA.first_ride.departureTime).getTime()
              const firstDepartureB = new Date(indirectB.first_ride.departureTime).getTime()
              compareValue = firstDepartureA - firstDepartureB
              break
            case SortType.TravelTime:
              const totalTravelTimeA =
                indirectA.first_ride.travelTime +
                indirectA.relaxing_time +
                indirectA.second_ride.travelTime
              const totalTravelTimeB =
                indirectB.first_ride.travelTime +
                indirectB.relaxing_time +
                indirectB.second_ride.travelTime
              compareValue = totalTravelTimeA - totalTravelTimeB
              break
            case SortType.Price:
              const firstPriceA = indirectA.first_ride.price
              const firstPriceB = indirectB.first_ride.price
              const secondPriceA = indirectA.second_ride.price
              const secondPriceB = indirectB.second_ride.price
              compareValue = firstPriceA + secondPriceA - (firstPriceB + secondPriceB)
              break
            default:
              break
          }
        }
        return this.sortOrderAsc ? compareValue : -compareValue
      })

      return filteredSchedules as directScheduleInfo[] | indirectScheduleInfo[]
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
    // ---------------------- 查询相关 --------------------
    // 查询方法封装
    async querySchedule() {
      if (this.queryDate === '') {
        message.error('请填写查询日期')
        return
      }
      if (this.queryDepartureText === '' || this.queryArrivalText === '') {
        message.error('请填写出发地点和到达地点')
        return
      }
      const params: scheduleRequest = {
        departureDate: this.queryDate,
      }
      // 根据文本判断城市/站点
      const checkDepartureText = generalStore.checkInputString(this.queryDepartureText)
      const checkArrivalText = generalStore.checkInputString(this.queryArrivalText)
      if (checkDepartureText === undefined || checkArrivalText === undefined) {
        message.error('出发地点或到达地点格式不正确，请检查输入')
        return
      }
      if (checkDepartureText.targetType === 'city') {
        params.departureCity = checkDepartureText.target
      } else if (checkDepartureText.targetType === 'station') {
        params.departureStation = checkDepartureText.target
      }
      if (checkArrivalText.targetType === 'city') {
        params.arrivalCity = checkArrivalText.target
      } else if (checkArrivalText.targetType === 'station') {
        params.arrivalStation = checkArrivalText.target
      }
      // 根据查询模式选择查询方法
      if (this.queryMode === 'direct') {
        try {
          const response = await TicketServiceApi.queryDirectSchedule(params)
          if (response.data) {
            this.handleResponse(response.data)
          }
        } catch (e: any) {
          message.error(`查询直达车次失败: ${e.message}`)
        }
      } else if (this.queryMode === 'indirect') {
        try {
          const response = await TicketServiceApi.queryIndirectSchedule(params)
          if (response.data) {
            this.handleResponse(response.data)
          }
        } catch (e: any) {
          message.error(`查询中转车次失败: ${e.message}`)
        }
      } else {
        message.error('查询模式不正确，请选择直达或中转')
      }
    },
    // 处理查询结果
    handleResponse(res: any) {
      if (res.code === 200) {
        this.queryResult = res.data.solutions
        // 更新筛选选项
        // 座位类型
        this.checkGroups[CheckType.SeatType].options = Array.from(
          new Set(
            this.queryResult.flatMap((schedule: any) => {
              if (this.queryMode === 'direct') {
                const directSchedule = schedule as directScheduleInfo
                return Object.keys(directSchedule.seatInfo)
              }
              if (this.queryMode === 'indirect') {
                const indirectSchedule = schedule as indirectScheduleInfo
                return [
                  ...Object.keys(indirectSchedule.first_ride.seatInfo),
                  ...Object.keys(indirectSchedule.second_ride.seatInfo),
                ]
              }
              return []
            }),
          ),
        ).sort()
        this.checkGroups[CheckType.SeatType].checkedList =
          this.checkGroups[CheckType.SeatType].options
        this.checkGroups[CheckType.SeatType].indeterminate = false
        this.checkGroups[CheckType.SeatType].checkAll = true
        // 出发站
        this.checkGroups[CheckType.DepartureStation].options = Array.from(
          new Set(
            this.queryResult.flatMap((schedule: any) => {
              if (this.queryMode === 'direct') {
                const directSchedule = schedule as directScheduleInfo
                return [directSchedule.departureStation]
              }
              if (this.queryMode === 'indirect') {
                const indirectSchedule = schedule as indirectScheduleInfo
                return [indirectSchedule.first_ride.departureStation]
              }
              return []
            }),
          ),
        ).sort()
        this.checkGroups[CheckType.DepartureStation].checkedList =
          this.checkGroups[CheckType.DepartureStation].options
        this.checkGroups[CheckType.DepartureStation].indeterminate = false
        this.checkGroups[CheckType.DepartureStation].checkAll = true
        // 中转站
        if (this.queryMode === 'indirect') {
          this.checkGroups[CheckType.TransferStation].options = Array.from(
            new Set(
              this.queryResult.flatMap((schedule: any) => {
                if (this.queryMode === 'indirect') {
                  const indirectSchedule = schedule as indirectScheduleInfo
                  return [indirectSchedule.second_ride.departureStation]
                }
                return []
              }),
            ),
          ).sort()
          this.checkGroups[CheckType.TransferStation].checkedList =
            this.checkGroups[CheckType.TransferStation].options
          this.checkGroups[CheckType.TransferStation].indeterminate = false
          this.checkGroups[CheckType.TransferStation].checkAll = true
        }
        // 到达站
        this.checkGroups[CheckType.ArrivalStation].options = Array.from(
          new Set(
            this.queryResult.flatMap((schedule: any) => {
              if (this.queryMode === 'direct') {
                const directSchedule = schedule as directScheduleInfo
                return [directSchedule.arrivalStation]
              }
              if (this.queryMode === 'indirect') {
                const indirectSchedule = schedule as indirectScheduleInfo
                return [indirectSchedule.second_ride.arrivalStation]
              }
              return []
            }),
          ),
        ).sort()
        this.checkGroups[CheckType.ArrivalStation].checkedList =
          this.checkGroups[CheckType.ArrivalStation].options
        this.checkGroups[CheckType.ArrivalStation].indeterminate = false
        this.checkGroups[CheckType.ArrivalStation].checkAll = true
        message.success('查询成功')
      } else if (res.code === 403) {
        message.error('查询失败: 无效的 session_id')
      } else if (res.code === 404) {
        message.error('查询失败: 无效的城市或车站名称')
      } else if (res.code === 12001) {
        message.error('查询失败: 查询字段不符合要求，请将问题报告给开发人员')
      } else {
        message.error(`查询失败: 未知错误`)
      }
    },
  },
})

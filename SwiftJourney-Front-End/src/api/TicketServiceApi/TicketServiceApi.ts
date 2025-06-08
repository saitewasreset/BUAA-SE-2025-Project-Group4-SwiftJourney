import { getRequest, postRequest } from '../axios'
import type {
  scheduleRequest, trainTransactionRequest
} from '@/interface/ticketServiceInterface'

export const TicketServiceApi = {
  // 查询直达车次
  queryDirectSchedule: (params: scheduleRequest) => {
    return postRequest('/api/train/schedule/query_direct', params)
  },

  // 查询中转车次
  queryIndirectSchedule: (params: scheduleRequest) => {
    return postRequest('/api/train/schedule/query_indirect', params)
  },

  // 提交订单
  submitOrder: (params: trainTransactionRequest) => {
    return postRequest('/api/train/order/new', params)
  },
}

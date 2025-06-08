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
        <div v-if="ticketServiceStore.isLoading" class="schedule-card">
          <div class="loading-container">
            <!-- 现代化骨架屏 -->
            <div class="skeleton-tickets">
              <div 
                v-for="n in 2" 
                :key="n" 
                class="skeleton-ticket-card"
                :style="{ animationDelay: `${(n - 1) * 0.1}s` }"
              >
                <!-- 骨架屏头部 -->
                <div class="skeleton-header">
                  <div class="skeleton-train-info">
                    <div class="skeleton-train-number"></div>
                    <div class="skeleton-train-type"></div>
                  </div>
                  <div class="skeleton-status">
                    <div class="skeleton-badge"></div>
                  </div>
                </div>

                <!-- 骨架屏路线信息 -->
                <div class="skeleton-route">
                  <div class="skeleton-station">
                    <div class="skeleton-time"></div>
                    <div class="skeleton-station-name"></div>
                  </div>
                  <div class="skeleton-route-line">
                    <div class="skeleton-duration"></div>
                    <div class="skeleton-line"></div>
                  </div>
                  <div class="skeleton-station">
                    <div class="skeleton-time"></div>
                    <div class="skeleton-station-name"></div>
                  </div>
                </div>

                <!-- 骨架屏座位和价格信息 -->
                <div class="skeleton-details">
                  <div class="skeleton-seats">
                    <div class="skeleton-seat-item" v-for="i in 4" :key="i">
                      <div class="skeleton-seat-type"></div>
                      <div class="skeleton-seat-price"></div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div v-else-if="ticketServiceStore.queryResult.length === 0" class="schedule-card">
          <div class="empty-container">
            <div class="empty-content">
              <div class="empty-text">
                <h3 class="empty-title">暂无车次信息</h3>
                <p class="empty-subtitle">请在上方搜索框中输入出发地和目的地进行查询</p>
                <img style="width: 300px; height: 300px" src="./EmptyTrain.svg" alt="暂无车次" class="empty-svg" />
              </div>
            </div>
          </div>
        </div>
        <div v-else class="schedule-card">
          <!-- 车次列表内容区域 -->
          <div class="schedule-list">
            <div v-if="ticketServiceStore.queryMode === 'direct'">
              <div v-if="paginatedDirectResults.length === 0" class="empty-state">
                <a-empty description="没有符合条件的直达车次" />
              </div>
              <div v-else class="ticket-list">
                <div v-for="(item, index) in paginatedDirectResults" :key="index" class="ticket-item">
                  <directScheduleInfoCard :content="item" />
                </div>
              </div>
            </div>
            <div v-if="ticketServiceStore.queryMode === 'indirect'">
              <div v-if="paginatedIndirectResults.length === 0" class="empty-state">
                <a-empty description="没有符合条件的中转车次" />
              </div>
              <div v-else class="ticket-list">
                <div v-for="(item, index) in paginatedIndirectResults" :key="index" class="ticket-item">
                  <indirectScheduleInfoCard :content="item" />
                </div>
              </div>
            </div>
          </div>
          <!-- 分页组件 -->
          <div class="pagination-container">
            <a-pagination
              v-model:current="currentPage"
              v-model:page-size="pageSize"
              :total="totalResults"
              :show-size-changer="false"
              :show-quick-jumper="false"
              :show-total="(total: number, range: number[]) => `第 ${range[0]}-${range[1]} 条，共 ${total} 条车次`"
              size="default"
              class="custom-pagination"
            />
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
import { computed, watch, ref } from 'vue'
import { useTicketServiceStore } from '@/stores/ticketService'
import {
  SortType,
  type directScheduleInfo,
  type indirectScheduleInfo,
} from '@/interface/ticketServiceInterface'

const ticketServiceStore = useTicketServiceStore()

// -------------------- 分页相关 --------------------
const currentPage = ref(1)
const pageSize = computed(() => {
  return ticketServiceStore.queryMode === 'direct' ? 5 : 3;
}) // 每页显示5条数据

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

// 总结果数量
const totalResults = computed(() => {
  return ticketServiceStore.queryMode === 'direct' 
    ? directResults.value.length 
    : indirectResults.value.length
})

// 分页后的直达车次结果
const paginatedDirectResults = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return directResults.value.slice(start, end)
})

// 分页后的中转车次结果
const paginatedIndirectResults = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return indirectResults.value.slice(start, end)
})

// -------------------- 监听查询模式和日期变化 --------------------
watch(
  () => ticketServiceStore.queryMode,
  async (newMode) => {
    currentPage.value = 1 // 重置页码
    await ticketServiceStore.querySchedule()
  },
)

watch(
  () => ticketServiceStore.queryDate,
  async (newDate) => {
    currentPage.value = 1 // 重置页码
    await ticketServiceStore.querySchedule()
  },
)

// 监听筛选结果变化，重置页码
watch(
  () => ticketServiceStore.displaySchedules,
  () => {
    currentPage.value = 1
  },
  { deep: true }
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
  background: linear-gradient(135deg, 
    rgba(248, 250, 252, 0.8) 0%, 
    rgba(255, 255, 255, 0.9) 100%
  );
  border-radius: 16px 16px 0 0;
  padding: 6px;
  gap: 2px;
  position: relative;
  overflow: hidden;
  
  // 添加顶部光晕效果
  &::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg, 
      transparent 0%, 
      rgba(59, 130, 246, 0.3) 20%, 
      rgba(139, 92, 246, 0.3) 50%,
      rgba(59, 130, 246, 0.3) 80%, 
      transparent 100%
    );
  }
}

.date-btn {
  flex: 1 1 0;
  text-align: center;
  white-space: nowrap;
  padding: 16px 12px !important; // 增加高度
  border-end-start-radius: 0;
  border-end-end-radius: 0;
  border: none;
  background: transparent;
  border-radius: 12px;
  font-size: 14px;
  font-weight: 500;
  color: #64748b;
  position: relative;
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
  min-height: 56px; // 确保最小高度
  display: flex;
  align-items: center;
  justify-content: center;
  
  // 玻璃质感背景
  &::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(135deg, 
      rgba(255, 255, 255, 0.4) 0%, 
      rgba(255, 255, 255, 0.1) 100%
    );
    border-radius: 12px;
    opacity: 0;
    transition: opacity 0.3s ease;
  }
  
  // 悬停光晕效果
  &::after {
    content: '';
    position: absolute;
    top: 50%;
    left: 50%;
    width: 0;
    height: 0;
    background: radial-gradient(circle, 
      rgba(59, 130, 246, 0.15) 0%, 
      rgba(59, 130, 246, 0.05) 40%,
      transparent 70%
    );
    border-radius: 50%;
    transform: translate(-50%, -50%);
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
    pointer-events: none;
  }
  
  &:hover {
    background: rgba(59, 130, 246, 0.06);
    color: #3b82f6;
    transform: translateY(-2px) scale(1.02);
    box-shadow: 
      0 6px 20px rgba(59, 130, 246, 0.12),
      0 2px 8px rgba(0, 0, 0, 0.08),
      inset 0 1px 0 rgba(255, 255, 255, 0.3);
    
    &::before {
      opacity: 1;
    }
    
    &::after {
      width: 120%;
      height: 120%;
    }
  }
  
  &:active {
    transform: translateY(-1px) scale(0.98);
  }
}

// 选中状态的现代化样式
.date-btn.ant-radio-button-wrapper-checked {
  background: linear-gradient(135deg, 
    #3b82f6 0%, 
    #1d4ed8 50%,
    #1e40af 100%
  );
  color: white;
  font-weight: 700;
  transform: translateY(-2px);
  box-shadow: 
    0 8px 32px rgba(59, 130, 246, 0.4),
    0 4px 16px rgba(59, 130, 246, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.3),
    inset 0 -1px 0 rgba(0, 0, 0, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  position: relative;
  z-index: 2;
  
  // 激活状态的玻璃质感
  &::before {
    background: linear-gradient(135deg, 
      rgba(255, 255, 255, 0.3) 0%, 
      rgba(255, 255, 255, 0.1) 50%,
      transparent 100%
    );
    opacity: 1;
  }
  
  // 激活状态的内发光
  &::after {
    width: 100%;
    height: 100%;
    background: radial-gradient(circle at center, 
      rgba(255, 255, 255, 0.2) 0%, 
      transparent 60%
    );
  }
  
  &:hover {
    background: linear-gradient(135deg, 
      #4f46e5 0%, 
      #3b82f6 50%,
      #2563eb 100%
    );
    transform: translateY(-3px) scale(1.02);
    box-shadow: 
      0 12px 40px rgba(59, 130, 246, 0.5),
      0 6px 20px rgba(59, 130, 246, 0.3),
      inset 0 1px 0 rgba(255, 255, 255, 0.4),
      inset 0 -1px 0 rgba(0, 0, 0, 0.1);
  }
}

// 第一个和最后一个按钮的特殊圆角
.date-btn:first-child {
  border-radius: 12px 4px 4px 12px;
}

.date-btn:last-child {
  border-radius: 4px 12px 12px 4px;
}

// 如果只有一个按钮，保持完整圆角
.date-btn:first-child:last-child {
  border-radius: 12px;
}

// 响应式设计优化
@media (max-width: 768px) {
  .date-picker {
    padding: 4px;
    gap: 1px;
  }
  
  .date-btn {
    padding: 12px 8px !important;
    font-size: 13px;
    min-height: 48px;
  }
}

@media (max-width: 480px) {
  .date-btn {
    padding: 10px 6px !important;
    font-size: 12px;
    min-height: 44px;
  }
}

.empty-text {
  // display: flex;
  // justify-content: center;
  align-items: center;
  text-align: center;
  font-size: 20px;
  margin-top: 40px;
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
  padding: 0 4px;
  min-height: 600px;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.6);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
  overflow: hidden;
}

.schedule-list {
  flex: 1;
  overflow-y: auto;
  padding: 16px 8px 0 8px;
  
  // 自定义滚动条样式
  &::-webkit-scrollbar {
    width: 6px;
  }
  
  &::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.05);
    border-radius: 3px;
  }
  
  &::-webkit-scrollbar-thumb {
    background: rgba(0, 0, 0, 0.2);
    border-radius: 3px;
    
    &:hover {
      background: rgba(0, 0, 0, 0.3);
    }
  }
}

.ticket-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-bottom: 12px; // 为最后一个元素添加底部间距
}

.ticket-item {
  // 可以在这里为单个车票项添加特殊样式
}

.pagination-container {
  padding: 24px 20px;
  display: flex;
  justify-content: center;
  align-items: center;
  background: linear-gradient(135deg, 
    rgba(255, 255, 255, 0.25) 0%, 
    rgba(255, 255, 255, 0.15) 50%,
    rgba(255, 255, 255, 0.1) 100%
  );
  backdrop-filter: blur(20px) saturate(180%);
  border-top: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 0 0 16px 16px;
  flex-shrink: 0;
  position: relative;
  
  // 玻璃质感边框
  &::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(135deg, 
      rgba(255, 255, 255, 0.1) 0%, 
      rgba(255, 255, 255, 0.05) 100%
    );
    border-radius: 0 0 16px 16px;
    border: 1px solid rgba(255, 255, 255, 0.18);
    border-top: none;
    pointer-events: none;
  }
  
  // 微妙的内阴影
  &::after {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: radial-gradient(
      ellipse at center top,
      rgba(59, 130, 246, 0.03) 0%,
      transparent 70%
    );
    pointer-events: none;
  }
}

:deep(.custom-pagination) {
  position: relative;
  z-index: 1;
  
  // 页码按钮
  .ant-pagination-item {
    border: 1px solid rgba(59, 130, 246, 0.15);
    border-radius: 12px;
    background: linear-gradient(135deg, 
      rgba(255, 255, 255, 0.7) 0%, 
      rgba(255, 255, 255, 0.4) 100%
    );
    backdrop-filter: blur(10px) saturate(120%);
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    overflow: hidden;
    box-shadow: 
      0 2px 8px rgba(0, 0, 0, 0.04),
      0 1px 2px rgba(0, 0, 0, 0.06),
      inset 0 1px 0 rgba(255, 255, 255, 0.3);
    
    // 玻璃质感高光
    &::before {
      content: '';
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      height: 50%;
      background: linear-gradient(180deg, 
        rgba(255, 255, 255, 0.2) 0%, 
        transparent 100%
      );
      pointer-events: none;
    }
    
    &:hover {
      border-color: rgba(59, 130, 246, 0.3);
      background: linear-gradient(135deg, 
        rgba(59, 130, 246, 0.08) 0%, 
        rgba(59, 130, 246, 0.04) 100%
      );
      transform: translateY(-2px) scale(1.05);
      box-shadow: 
        0 8px 25px rgba(59, 130, 246, 0.15),
        0 4px 10px rgba(0, 0, 0, 0.1),
        inset 0 1px 0 rgba(255, 255, 255, 0.4);
    }
    
    a {
      color: #475569;
      font-weight: 600;
      font-size: 14px;
      transition: all 0.3s ease;
      position: relative;
      z-index: 1;
      
      &:hover {
        color: #3b82f6;
      }
    }
  }
  
  // 当前激活页码
  .ant-pagination-item-active {
    border-color: rgba(59, 130, 246, 0.4);
    background: linear-gradient(135deg, 
      #3b82f6 0%, 
      #1d4ed8 50%,
      #1e40af 100%
    );
    box-shadow: 
      0 8px 32px rgba(59, 130, 246, 0.4),
      0 4px 16px rgba(59, 130, 246, 0.2),
      inset 0 1px 0 rgba(255, 255, 255, 0.3),
      inset 0 -1px 0 rgba(0, 0, 0, 0.1);
    transform: translateY(-1px);
    
    &::before {
      background: linear-gradient(180deg, 
        rgba(255, 255, 255, 0.3) 0%, 
        transparent 100%
      );
    }
    
    a {
      color: white;
      font-weight: 700;
      text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
    }
    
    &:hover {
      transform: translateY(-3px) scale(1.05);
      box-shadow: 
        0 12px 40px rgba(59, 130, 246, 0.5),
        0 6px 20px rgba(59, 130, 246, 0.3),
        inset 0 1px 0 rgba(255, 255, 255, 0.4),
        inset 0 -1px 0 rgba(0, 0, 0, 0.1);
      background: linear-gradient(135deg, 
        #4f46e5 0%, 
        #3b82f6 50%,
        #2563eb 100%
      );
    }
  }
  
  // 前后翻页按钮
  .ant-pagination-prev,
  .ant-pagination-next {
    border: 1px solid rgba(59, 130, 246, 0.12);
    border-radius: 12px;
    background: linear-gradient(135deg, 
      rgba(255, 255, 255, 0.6) 0%, 
      rgba(255, 255, 255, 0.3) 100%
    );
    backdrop-filter: blur(12px);
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    overflow: hidden;
    box-shadow: 
      0 2px 8px rgba(0, 0, 0, 0.04),
      0 1px 2px rgba(0, 0, 0, 0.06),
      inset 0 1px 0 rgba(255, 255, 255, 0.25);
    
    &::before {
      content: '';
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      height: 50%;
      background: linear-gradient(180deg, 
        rgba(255, 255, 255, 0.15) 0%, 
        transparent 100%
      );
      pointer-events: none;
    }
    
    &:hover {
      border-color: rgba(59, 130, 246, 0.25);
      background: linear-gradient(135deg, 
        rgba(59, 130, 246, 0.06) 0%, 
        rgba(59, 130, 246, 0.03) 100%
      );
      transform: translateY(-2px) scale(1.05);
      box-shadow: 
        0 8px 25px rgba(59, 130, 246, 0.12),
        0 4px 10px rgba(0, 0, 0, 0.08),
        inset 0 1px 0 rgba(255, 255, 255, 0.3);
    }
    
    .ant-pagination-item-link {
      color: #64748b;
      background: transparent;
      border: none;
      transition: all 0.3s ease;
      font-size: 16px;
      
      &:hover {
        color: #3b82f6;
      }
    }
  }
  
  // 禁用状态
  .ant-pagination-disabled {
    opacity: 0.4;
    cursor: not-allowed;
    
    &:hover {
      border-color: rgba(59, 130, 246, 0.12);
      background: linear-gradient(135deg, 
        rgba(255, 255, 255, 0.6) 0%, 
        rgba(255, 255, 255, 0.3) 100%
      );
      transform: none;
      box-shadow: 
        0 2px 8px rgba(0, 0, 0, 0.04),
        0 1px 2px rgba(0, 0, 0, 0.06),
        inset 0 1px 0 rgba(255, 255, 255, 0.25);
    }
    
    .ant-pagination-item-link {
      color: #cbd5e1;
      
      &:hover {
        color: #cbd5e1;
      }
    }
  }
  
  // 跳转省略号
  .ant-pagination-jump-prev,
  .ant-pagination-jump-next {
    border: 1px solid rgba(59, 130, 246, 0.12);
    border-radius: 12px;
    background: linear-gradient(135deg, 
      rgba(255, 255, 255, 0.6) 0%, 
      rgba(255, 255, 255, 0.3) 100%
    );
    backdrop-filter: blur(12px);
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    overflow: hidden;
    box-shadow: 
      0 2px 8px rgba(0, 0, 0, 0.04),
      0 1px 2px rgba(0, 0, 0, 0.06),
      inset 0 1px 0 rgba(255, 255, 255, 0.25);
    
    // 玻璃质感高光
    &::before {
      content: '⋯';
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      height: 50%;
      background: linear-gradient(180deg, 
        rgba(255, 255, 255, 0.15) 0%, 
        transparent 100%
      );
      pointer-events: none;
    }
    
    &:hover {
      border-color: rgba(59, 130, 246, 0.25);
      background: linear-gradient(135deg, 
        rgba(59, 130, 246, 0.06) 0%, 
        rgba(59, 130, 246, 0.03) 100%
      );
      transform: translateY(-2px) scale(1.05);
      box-shadow: 
        0 8px 25px rgba(59, 130, 246, 0.12),
        0 4px 10px rgba(0, 0, 0, 0.08),
        inset 0 1px 0 rgba(255, 255, 255, 0.3);
    }
    
    .ant-pagination-item-container {
      display: flex;
      align-items: center;
      justify-content: center;
      height: 100%;
      
      .ant-pagination-item-link-icon {
        color: #64748b;
        transition: all 0.3s ease;
        filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.1));
        font-size: 14px;
      }
      
      .ant-pagination-item-ellipsis {
        color: #94a3b8;
        font-weight: 600;
        font-size: 14px;
        letter-spacing: 2px;
      }
    }
    
    &:hover {
      .ant-pagination-item-container {
        .ant-pagination-item-link-icon {
          color: #3b82f6;
          transform: scale(1.2);
        }
        
        .ant-pagination-item-ellipsis {
          color: #3b82f6;
        }
      }
    }
  }
  
  // 专门处理省略号的显示逻辑
  .ant-pagination-item-ellipsis {
    color: #94a3b8;
    font-weight: 600;
    font-size: 14px;
    letter-spacing: 1px;
    user-select: none;
  }
  
  // 确保省略号图标正确显示
  .ant-pagination-item-link-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    
    svg {
      width: 14px;
      height: 14px;
    }
  }
  
  // 修复可能的覆盖问题
  .ant-pagination-jump-prev,
  .ant-pagination-jump-next {
    .ant-pagination-item-container {
      position: relative;
      z-index: 1;
      
      .ant-pagination-item-link-icon,
      .ant-pagination-item-ellipsis {
        position: relative;
        z-index: 2;
      }
    }
  }
}

// 如果省略号仍然不显示，添加强制显示样式
:deep(.ant-pagination-jump-prev),
:deep(.ant-pagination-jump-next) {
  .ant-pagination-item-container {
    .ant-pagination-item-ellipsis {
      display: inline-block !important;
      visibility: visible !important;
      opacity: 1 !important;
    }
    
    .ant-pagination-item-link-icon {
      display: none;
    }
  }
  
  &:hover {
    .ant-pagination-item-container {
      .ant-pagination-item-ellipsis {
        display: none !important;
      }
      
      .ant-pagination-item-link-icon {
        display: inline-flex !important;
      }
    }
  }
}

/* 现代化加载状态样式 */
.loading-container {
  position: relative;
  min-height: 600px;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  align-items: stretch;
  background: linear-gradient(135deg, 
    rgba(255, 255, 255, 0.95) 0%, 
    rgba(248, 250, 252, 0.95) 100%
  );
  padding: 16px 8px;
}

/* 骨架屏票卡样式 */
.skeleton-tickets {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
}

.skeleton-ticket-card {
  background: rgba(255, 255, 255, 0.9);
  border-radius: 12px;
  border: 1px solid rgba(226, 232, 240, 0.4);
  padding: 20px;
  position: relative;
  overflow: hidden;
  animation: fadeInUp 0.6s cubic-bezier(0.4, 0, 0.2, 1) forwards;
  opacity: 0;
  backdrop-filter: blur(10px);
  box-shadow: 
    0 4px 16px rgba(0, 0, 0, 0.04),
    0 0 0 1px rgba(255, 255, 255, 0.5);
  
  /* 玻璃质感边框 */
  &::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(135deg, 
      rgba(255, 255, 255, 0.2) 0%, 
      rgba(255, 255, 255, 0.05) 100%
    );
    border-radius: 12px;
    pointer-events: none;
  }
}

/* 骨架屏头部 */
.skeleton-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  position: relative;
  z-index: 2;
}

.skeleton-train-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.skeleton-train-number {
  width: 80px;
  height: 24px;
  background: linear-gradient(135deg, #e2e8f0 0%, #f1f5f9 100%);
  border-radius: 8px;
  position: relative;
  overflow: hidden;
}

.skeleton-train-type {
  width: 60px;
  height: 16px;
  background: linear-gradient(135deg, #e2e8f0 0%, #f1f5f9 100%);
  border-radius: 6px;
  position: relative;
  overflow: hidden;
}

.skeleton-status {
  display: flex;
  align-items: center;
}

.skeleton-badge {
  width: 80px;
  height: 28px;
  background: linear-gradient(135deg, #e2e8f0 0%, #f1f5f9 100%);
  border-radius: 8px;
  position: relative;
  overflow: hidden;
}

/* 骨架屏路线信息 */
.skeleton-route {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  gap: 20px;
  align-items: center;
  margin-bottom: 20px;
  position: relative;
  z-index: 2;
}

.skeleton-station {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.skeleton-time {
  width: 60px;
  height: 20px;
  background: linear-gradient(135deg, #e2e8f0 0%, #f1f5f9 100%);
  border-radius: 6px;
  position: relative;
  overflow: hidden;
}

.skeleton-station-name {
  width: 80px;
  height: 18px;
  background: linear-gradient(135deg, #e2e8f0 0%, #f1f5f9 100%);
  border-radius: 6px;
  position: relative;
  overflow: hidden;
}

.skeleton-route-line {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.skeleton-duration {
  width: 70px;
  height: 16px;
  background: linear-gradient(135deg, #e2e8f0 0%, #f1f5f9 100%);
  border-radius: 6px;
  position: relative;
  overflow: hidden;
}

.skeleton-line {
  width: 120px;
  height: 3px;
  background: linear-gradient(90deg, 
    rgba(59, 130, 246, 0.2) 0%, 
    rgba(139, 92, 246, 0.2) 50%,
    rgba(59, 130, 246, 0.2) 100%
  );
  border-radius: 2px;
  position: relative;
  overflow: hidden;
}

/* 骨架屏座位和价格信息 */
.skeleton-details {
  position: relative;
  z-index: 2;
}

.skeleton-seats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 12px;
}

.skeleton-seat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 12px;
  background: rgba(248, 250, 252, 0.6);
  border-radius: 8px;
  border: 1px solid rgba(226, 232, 240, 0.3);
}

.skeleton-seat-type {
  width: 60px;
  height: 16px;
  background: linear-gradient(135deg, #e2e8f0 0%, #f1f5f9 100%);
  border-radius: 6px;
  position: relative;
  overflow: hidden;
}

.skeleton-seat-price {
  width: 80px;
  height: 20px;
  background: linear-gradient(135deg, 
    rgba(59, 130, 246, 0.15) 0%, 
    rgba(139, 92, 246, 0.15) 100%
  );
  border-radius: 6px;
  position: relative;
  overflow: hidden;
}

/* 通用骨架屏闪烁动画 */
.skeleton-train-number,
.skeleton-train-type,
.skeleton-badge,
.skeleton-time,
.skeleton-station-name,
.skeleton-duration,
.skeleton-seat-type,
.skeleton-seat-price {
  &::after {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, 
      transparent, 
      rgba(255, 255, 255, 0.8), 
      transparent
    );
    animation: shimmer 2s infinite;
  }
}

.skeleton-line {
  &::after {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, 
      transparent, 
      rgba(59, 130, 246, 0.4), 
      transparent
    );
    animation: shimmer 2s infinite;
  }
}

/* 动画定义 */
@keyframes shimmer {
  0% {
    left: -100%;
  }
  100% {
    left: 100%;
  }
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 骨架屏渐进式延迟动画 */
.skeleton-ticket-card:nth-child(1) { animation-delay: 0.05s; }
.skeleton-ticket-card:nth-child(2) { animation-delay: 0.1s; }
.skeleton-ticket-card:nth-child(3) { animation-delay: 0.15s; }
.skeleton-ticket-card:nth-child(4) { animation-delay: 0.2s; }
.skeleton-ticket-card:nth-child(5) { animation-delay: 0.25s; }

/* 响应式设计 */
@media (max-width: 768px) {
  .skeleton-route {
    grid-template-columns: 1fr;
    gap: 12px;
    text-align: center;
  }
  
  .skeleton-route-line {
    order: -1;
  }
  
  .skeleton-line {
    width: 80px;
  }
  
  .skeleton-seats {
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: 8px;
  }
  
  .skeleton-seat-item {
    padding: 8px;
  }
}
</style>

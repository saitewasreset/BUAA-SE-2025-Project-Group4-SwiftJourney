<template>
  <div class="hotel-search-card">
    <el-card shadow="hover" class="search-card">
      <!-- 标题区域 -->
      <div class="card-header">
        <div class="header-content">
          <div class="icon-wrapper">
            <el-icon size="24" color="#409eff">
              <House />
            </el-icon>
          </div>
          <div class="title-section">
            <h3 class="card-title">预订酒店</h3>
            <p class="card-subtitle">舒适住宿，安心之旅</p>
          </div>
        </div>
      </div>

      <!-- 搜索表单区域 -->
      <div class="search-form">
        <!-- 城市选择区域 -->
        <div class="form-section">
          <div class="city-selection-wrapper">
            <Teleport to="body">
              <SelectCard
                v-if="isCurChooseRefActive"
                :el="inputRef"
                :input="cityInput"
                @handleCityClick="handleCityClick"
                :style="selectCardStyle"
                type="city"
              />
            </Teleport>
            
            <div class="input-group">
              <label class="input-label">目的地城市</label>
              <a-input
                ref="cityInputRef"
                id="HotelCityInput"
                class="city-input"
                v-model:value="hotelQuery.target"
                :bordered="false"
                size="large"
                placeholder="请选择目的地城市"
                @input="handleCityInput"
                @focus="handleInputFocus"
                @compositionupdate="handleCompositionUpdate"
              />
            </div>
          </div>
        </div>

        <!-- 酒店名称区域 -->
        <div class="form-section">
          <div class="input-group">
            <label class="input-label">酒店名称（选填）</label>
            <a-input
              class="hotel-input"
              v-model:value="hotelQuery.search"
              :bordered="false"
              size="large"
              placeholder="酒店名称"
            />
          </div>
        </div>

        <!-- 日期选择区域 -->
        <div class="form-section date-section">
          <div class="input-group">
            <label class="input-label">入住时间</label>
            <div class="date-display">
              <span class="date-text">入住</span>
              <span class="nights-text">{{ dateRangeNum }}晚</span>
              <span class="date-text">退房</span>
            </div>
            <a-range-picker
              v-model:value="selectedDateRange"
              class="date-picker"
              :locale="locale"
              :format="dateFormat"
              :bordered="false"
              :allow-clear="false"
              :disabled-date="disabledDate"
              @change="onDateRangeChange"
              size="large"
            />
          </div>
        </div>

        <!-- 搜索按钮 -->
        <div class="form-section search-section">
          <a-button
            type="primary"
            size="large"
            class="search-button"
            @click="handleSearch"
          >
            <template #icon>
              <SearchOutlined />
            </template>
            <span>搜索酒店</span>
          </a-button>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, computed, watch, Teleport, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { House } from '@element-plus/icons-vue';
import { SearchOutlined } from '@ant-design/icons-vue';
import SelectCard from '../SelectCard/SelectCard.vue';
import type { HotelQuery } from '@/interface/hotelInterface';
import { useGeneralStore } from '@/stores/general';
import { ElMessage } from 'element-plus';

import locale from 'ant-design-vue/es/date-picker/locale/zh_CN';
import dayjs from 'dayjs';
import 'dayjs/locale/zh-cn';

dayjs.locale('zh-cn');

const router = useRouter();
const generalStore = useGeneralStore();

const today = dayjs();
const nextday = today.add(1, 'day');

const hotelQuery = ref<HotelQuery>({
  target: '',
  targetType: "city",
  beginDate: formateDate(today),
  endDate: formateDate(nextday),
  search: ''
});

// 日期相关
const dateFormat = 'MM-DD(ddd)';
const selectedDateRange = ref([today, nextday]);
const dateRangeNum = ref<number>(1);

function disabledDate(current: any) {
  return current && current < dayjs().startOf('day');
}

function onDateRangeChange(dateRange: any) {
  if (dateRange && dateRange.length === 2) {
    hotelQuery.value.beginDate = formateDate(dateRange[0]);
    hotelQuery.value.endDate = formateDate(dateRange[1]);
    const startDate = dayjs(dateRange[0]);
    const endDate = dayjs(dateRange[1]);
    dateRangeNum.value = endDate.diff(startDate, 'day');
  }
}

function formateDate(date: any) {
  if (!date) return '';
  return dayjs(date).format('YYYY-MM-DD');
}

// 城市选择相关
const isCurChooseRefActive = ref(false);
const inputRef = ref<HTMLElement | undefined>(undefined);
const selectCardStyle = ref({});
const cityInput = ref('');

async function handleInputFocus() {
  const inputElement = document.getElementById('HotelCityInput') as HTMLElement;
  inputRef.value = inputElement;
  
  if (inputElement) {
    const rect = inputElement.getBoundingClientRect();
    selectCardStyle.value = {
      position: 'fixed',
      top: `${rect.bottom + 5}px`,
      left: `${rect.left}px`,
      width: `${rect.width}px`,
      zIndex: 9999,
      background: 'white',
      border: '1px solid #d9d9d9',
      borderRadius: '6px',
      boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
      maxHeight: '300px',
      overflow: 'auto'
    };
  }
  
  isCurChooseRefActive.value = false;
  await nextTick();
  isCurChooseRefActive.value = true;
}

function handleCityClick(item: string) {
  hotelQuery.value.target = item;
  isCurChooseRefActive.value = false;
}

function handleGlobalClick(event: MouseEvent) {
  const citySelectElement = document.querySelector('.city_choose_wrap');
  const inputElement = inputRef.value;

  if (
    citySelectElement &&
    !citySelectElement.contains(event.target as Node) &&
    inputElement &&
    !inputElement.contains(event.target as Node)
  ) {
    isCurChooseRefActive.value = false;
  }
}

const handleCityInput = () => {
  cityInput.value = hotelQuery.value.target;
};

const handleCompositionUpdate = (event: CompositionEvent) => {
  cityInput.value = hotelQuery.value.target + event.data.toLowerCase();
};

watch(() => hotelQuery.value.target, (newValue) => {
  cityInput.value = newValue || '';
});

// 搜索功能
function handleSearch() {
  if (!checkHotelQuery()) {
    return;
  }

  const result = generalStore.checkInputString(hotelQuery.value.target);
  if (result === undefined) {
    ElMessage.error('请输入正确的城市名');
    return;
  }

  // 跳转到酒店搜索页面，并传递查询参数
  router.push({
    name: 'hotel',
    query: {
      target: result.target,
      targetType: result.targetType,
      beginDate: hotelQuery.value.beginDate,
      endDate: hotelQuery.value.endDate,
      search: hotelQuery.value.search || ''
    }
  });
}

function checkHotelQuery() {
  hotelQuery.value.target = hotelQuery.value.target.trim();
  if (hotelQuery.value.target === '') {
    ElMessage.error('目的地不能为空');
    return false;
  }
  if (hotelQuery.value.beginDate === '' || hotelQuery.value.endDate === '') {
    ElMessage.error('入住和离店时间不能为空');
    return false;
  }
  if (dateRangeNum.value > 7) {
    ElMessage.error('入住时间不能超过7晚');
    return false;
  }
  return true;
}

onMounted(() => {
  generalStore.init();
  document.addEventListener('click', handleGlobalClick);
});

onUnmounted(() => {
  document.removeEventListener('click', handleGlobalClick);
});
</script>

<style lang="scss" scoped>
.hotel-search-card {
  width: 100%;
  max-width: 500px;
}

.search-card {
  border-radius: 16px;
  border: none;
  background: linear-gradient(135deg, #ffffff 0%, #f0f8ff 100%);
  box-shadow: 0 8px 32px rgba(64, 158, 255, 0.12);
  transition: all 0.3s ease;
  overflow: visible;

  &:hover {
    transform: translateY(-4px);
    box-shadow: 0 16px 48px rgba(64, 158, 255, 0.18);
  }
}

.search-card :deep(.el-card__body) {
  padding: 24px;
  overflow: visible;
}

.card-header {
  margin-bottom: 20px;
  padding-bottom: 16px;
  border-bottom: 1px solid #e8f4ff;
}

.header-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.icon-wrapper {
  padding: 8px;
  background: linear-gradient(135deg, #409eff, #67c23a);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 12px rgba(64, 158, 255, 0.3);
}

.title-section {
  flex: 1;
}

.card-title {
  margin: 0 0 4px 0;
  font-size: 20px;
  font-weight: 700;
  color: #1e3a8a;
  line-height: 1.2;
}

.card-subtitle {
  margin: 0;
  font-size: 14px;
  color: #6b7280;
  font-weight: 500;
}

.search-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-section {
  position: relative;

  &.date-section {
    .input-group {
      .date-display {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 8px;
        padding: 0 4px;
      }

      .date-text {
        font-size: 12px;
        color: #6b7280;
        font-weight: 500;
      }

      .nights-text {
        font-size: 12px;
        color: #409eff;
        font-weight: 600;
        background: rgba(64, 158, 255, 0.1);
        padding: 2px 8px;
        border-radius: 12px;
      }
    }
  }

  &.search-section {
    margin-top: 8px;
  }
}

.city-selection-wrapper {
  position: relative;
}

.input-group {
  background: #f8fafc;
  border-radius: 12px;
  padding: 12px 16px;
  transition: all 0.3s ease;

  &:hover {
    background: #f1f5f9;
    transform: translateY(-1px);
  }

  &:focus-within {
    background: #f1f5f9;
    transform: translateY(-1px);
    box-shadow: 0 0 0 2px rgba(64, 158, 255, 0.2);
  }
}

.input-label {
  display: block;
  font-size: 12px;
  color: #6b7280;
  margin-bottom: 6px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.city-input,
.hotel-input {
  font-size: 16px;
  font-weight: 600;
  color: #1e40af;
  background: transparent;
  width: 100%;

  &::placeholder {
    color: #9ca3af;
    font-weight: 400;
  }
}

.date-picker {
  width: 100%;
  background: transparent;

  :deep(.ant-picker-input) {
    font-size: 14px;
    font-weight: 600;
    color: #1e40af;

    input::placeholder {
      color: #9ca3af;
      font-weight: 400;
    }
  }

  :deep(.ant-picker-range-separator) {
    color: #6b7280;
  }
}

.search-button {
  width: 100%;
  height: 48px;
  border-radius: 12px;
  font-size: 16px;
  font-weight: 600;
  background: linear-gradient(135deg, #409eff 0%, #67c23a 100%);
  border: none;
  box-shadow: 0 4px 16px rgba(64, 158, 255, 0.3);
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;

  &::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.2), transparent);
    transition: left 0.5s;
  }

  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba(64, 158, 255, 0.4);
    background: linear-gradient(135deg, #337ecc 0%, #5daf34 100%);

    &::before {
      left: 100%;
    }
  }

  &:active {
    transform: translateY(0);
  }

  span {
    margin-left: 8px;
    font-weight: 700;
  }
}

// 响应式设计
@media (max-width: 640px) {
  .hotel-search-card {
    max-width: 100%;
  }

  .search-card :deep(.el-card__body) {
    padding: 20px;
  }

  .card-title {
    font-size: 18px;
  }

  .search-form {
    gap: 14px;
  }

  .search-button {
    height: 44px;
    font-size: 15px;
  }
}
</style>
<template>
    <div class="ticket-search">
        <el-card shadow="hover" class="search-card">
            <!-- 票务模式选择 -->
            <div class="ticket-mode-section">
                <a-radio-group :value="selectedTicketMode" @change="handleSelectTicketMode" class="mode-radio">
                    <a-radio value="OneWay">
                        <span class="mode-text">直达</span>
                    </a-radio>
                    <a-radio value="Transfer">
                        <span class="mode-text">中转</span>
                    </a-radio>
                </a-radio-group>
            </div>

            <!-- 主要搜索区域 -->
            <div class="search-container">
                <!-- 城市选择区域 -->
                <div class="city-selection-wrapper">
                    <CitySelect 
                        v-if="isCurChooseRefActive"
                        :el="inputRef"
                        @handleCityClick="handleCityClick"
                    />
                    
                    <div class="city-selection">
                        <!-- 出发城市 -->
                        <div class="city-input-group departure">
                            <label class="city-label">出发城市</label>
                            <a-input
                                id="DepartureCityInput"
                                @Focus="handleInputFocus('DepartureCityInput')"
                                class="city-input"
                                :bordered="false"
                                size="large"
                                v-model:value="departureCity"
                                placeholder="请选择出发城市"
                            />
                        </div>

                        <!-- 交换按钮 -->
                        <div class="swap-button-wrapper">
                            <a-button
                                class="swap-button"
                                shape="circle"
                                :icon="h(SwapOutlined)"
                                @click="swapCitys"
                            />
                        </div>

                        <!-- 到达城市 -->
                        <div class="city-input-group arrival">
                            <label class="city-label">到达城市</label>
                            <a-input
                                id="ArrivalCityInput"
                                @Focus="handleInputFocus('ArrivalCityInput')"
                                class="city-input arrival-input"
                                :bordered="false"
                                size="large"
                                v-model:value="arrivalCity"
                                placeholder="请选择到达城市"
                            />
                        </div>
                    </div>
                </div>

                <!-- 日期选择区域 -->
                <div class="date-selection">
                    <label class="date-label">出发日期</label>
                    <a-date-picker
                        suffix-icon=""
                        id="DatePicker"
                        size="large"
                        :locale="locale"
                        :format="dateFormat"
                        :bordered="false"
                        class="date-picker"
                        placeholder="选择出发日期"
                        :disabledDate="disabledDate"
                        v-model:value="selectedDate"
                    />
                </div>

                <!-- 搜索按钮 -->
                <div class="search-button-wrapper">
                    <a-button type="primary" size="large" class="search-button">
                        <template #icon>
                            <SearchOutlined />
                        </template>
                        <span class="search-text">查询车票</span>
                    </a-button>
                </div>
            </div>
        </el-card>
    </div>
</template>

<script setup lang="ts">
    import { ref, h, nextTick } from 'vue';
    import { onMounted, onUnmounted } from 'vue';
    
    import { SwapOutlined, SearchOutlined } from '@ant-design/icons-vue';
    
    //-------------DatePicker----------------

    import locale from 'ant-design-vue/es/date-picker/locale/zh_CN';
    import dayjs, { Dayjs } from 'dayjs';
    import 'dayjs/locale/zh-cn';

    dayjs.locale('zh-cn');

    const dateFormat = 'YYYY-MM-DD     dddd';

    // 添加日期相关的响应式数据
    const selectedDate = ref<Dayjs | null>(null);

    // 禁用过去日期的函数
    const disabledDate = (current: Dayjs) => {
        // 禁用今天之前的所有日期
        return current && current < dayjs().startOf('day');
    };

    // --------------------------------------

    // ----------selectTicketMode------------

    const selectedTicketMode = ref('OneWay');

    // 优化事件处理函数
    function handleSelectTicketMode(e: any) {
        selectedTicketMode.value = e.target.value;
    }

    // --------------------------------------

    // -------------SelectCity---------------

    const departureCity = ref<string>('');
    const arrivalCity = ref<string>('');

    function swapCitys() {
        const temp = departureCity.value;
        departureCity.value = arrivalCity.value;
        arrivalCity.value = temp;
    }

    const inputRef = ref<HTMLElement | undefined>(undefined)

    const isCurChooseRefActive = ref<boolean>(false);

    import CitySelect from './CitySelect/CitySelect.vue';

    const selectedInputId = ref<string>('');

    async function handleInputFocus(id: string) {
        selectedInputId.value = id;
        const inputElement = document.getElementById(id) as HTMLElement;
        inputRef.value = inputElement;
        isCurChooseRefActive.value = false;
        await nextTick();
        isCurChooseRefActive.value = true;
    }

    function handleCityClick(item: Object) {
        const cityName: string = item.cityName;
        if(selectedInputId.value === 'DepartureCityInput') {
            departureCity.value = cityName;
        } else if(selectedInputId.value === 'ArrivalCityInput') {
            arrivalCity.value = cityName;
        }
        isCurChooseRefActive.value = false;
    };


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

    onMounted(() => {
        document.addEventListener('click', handleGlobalClick);
    });

    onUnmounted(() => {
        document.removeEventListener('click', handleGlobalClick);
    });

</script>

<style lang="scss" scoped>
.ticket-search {
    width: 100%;
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
}

/* 搜索卡片 */
.search-card {
    border-radius: 16px;
    border: none;
    background: linear-gradient(135deg, #ffffff 0%, #f8f9ff 100%);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
    transition: all 0.3s ease;
    position: relative;
    overflow: visible; /* 确保内容可以溢出显示 */
    
    &:hover {
        transform: translateY(-2px);
        box-shadow: 0 12px 40px rgba(0, 0, 0, 0.12);
    }
}

.search-card :deep(.el-card__body) {
    padding: 32px 40px;
    overflow: visible; /* 确保卡片内容可以溢出 */
}

/* 票务模式选择区域 */
.ticket-mode-section {
    margin-bottom: 32px;
    padding-bottom: 20px;
    border-bottom: 1px solid #e4e7ed;
}

.mode-radio {
    display: flex;
    gap: 24px;
    
    .mode-text {
        font-size: 16px;
        font-weight: 500;
        color: #606266;
        transition: color 0.2s ease; /* 减少过渡时间 */
    }
}

.mode-radio :deep(.ant-radio-wrapper) {
    padding: 8px 16px;
    border-radius: 8px;
    transition: all 0.2s ease; /* 减少过渡时间 */
    
    &:hover {
        background-color: rgba(64, 158, 255, 0.05);
    }
    
    &.ant-radio-wrapper-checked {
        background-color: rgba(64, 158, 255, 0.1);
        
        .mode-text {
            color: #409eff;
            font-weight: 600;
        }
    }
}

/* 主搜索容器 */
.search-container {
    display: flex;
    align-items: flex-end;
    gap: 24px;
    flex-wrap: wrap;
    position: relative;
    overflow: visible; /* 确保容器内容可以溢出 */
}

/* 城市选择区域 */
.city-selection-wrapper {
    flex: 1;
    min-width: 400px;
    position: relative;
}

.city-selection {
    display: flex;
    align-items: center;
    background: #f8f9fa;
    border-radius: 12px;
    padding: 24px 20px 20px;
    position: relative;
    transition: all 0.3s ease;
    /* 移除可能的 overflow 限制 */
    
    &:hover {
        background: #f0f2f5;
        transform: translateY(-1px);
    }
}

.city-input-group {
    flex: 1;
    
    &.departure {
        text-align: left;
    }
    
    &.arrival {
        text-align: right;
    }
}

.city-label {
    display: block;
    font-size: 14px;
    color: #909399;
    margin-bottom: 8px;
    font-weight: 500;
}

.city-input {
    font-size: 18px;
    font-weight: 600;
    color: #303133;
    background: transparent;
    
    &.arrival-input :deep(.ant-input) {
        text-align: right;
    }
    
    &:focus {
        color: #409eff;
    }
}

.city-input :deep(.ant-input) {
    font-size: 18px;
    font-weight: 600;
    background: transparent;
    
    &::placeholder {
        color: #c0c4cc;
        font-weight: 400;
    }
}

/* 交换按钮 */
.swap-button-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 16px;
}

.swap-button {
    width: 40px;
    height: 40px;
    border: 2px solid #e4e7ed;
    background: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.3s ease;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    
    &:hover {
        border-color: #409eff;
        color: #409eff;
        transform: rotate(180deg);
        box-shadow: 0 4px 12px rgba(64, 158, 255, 0.3);
    }
}

/* 日期选择区域 */
.date-selection {
    min-width: 200px;
    background: #f8f9fa;
    border-radius: 12px;
    padding: 24px 20px 20px;
    transition: all 0.3s ease;
    
    &:hover {
        background: #f0f2f5;
        transform: translateY(-1px);
    }
}

.date-label {
    display: block;
    font-size: 14px;
    color: #909399;
    margin-bottom: 8px;
    font-weight: 500;
}

.date-picker {
    width: 100%;
    background: transparent;
    
    :deep(.ant-picker-input) {
        font-size: 16px;
        font-weight: 600;
        color: #303133;
        
        input::placeholder {
            color: #c0c4cc;
            font-weight: 400;
        }
    }
}

/* 搜索按钮区域 */
.search-button-wrapper {
    display: flex;
    align-items: center;
}

.search-button {
    height: 88px;
    width: 120px;
    border-radius: 12px;
    font-size: 16px;
    font-weight: 600;
    background: linear-gradient(135deg, #409eff 0%, #67c23a 100%);
    border: none;
    box-shadow: 0 4px 16px rgba(64, 158, 255, 0.3);
    transition: all 0.3s ease;
    
    &:hover {
        transform: translateY(-2px);
        box-shadow: 0 8px 24px rgba(64, 158, 255, 0.4);
        background: linear-gradient(135deg, #337ecc 0%, #5daf34 100%);
    }
    
    &:active {
        transform: translateY(0);
    }
}

.search-text {
    margin-left: 8px;
}

/* 响应式设计 */
@media (max-width: 1024px) {
    .search-container {
        flex-direction: column;
        align-items: stretch;
    }
    
    .city-selection-wrapper {
        min-width: unset;
    }
    
    .city-selection {
        flex-direction: column;
        gap: 16px;
        text-align: center;
    }
    
    .city-input-group {
        &.arrival {
            text-align: center;
        }
    }
    
    .city-input.arrival-input :deep(.ant-input) {
        text-align: center;
    }
    
    .swap-button-wrapper {
        order: 2;
        margin: 0;
    }
    
    .swap-button {
        transform: rotate(90deg);
        
        &:hover {
            transform: rotate(270deg);
        }
    }
    
    .search-button-wrapper {
        justify-content: center;
    }
    
    .search-button {
        width: 200px;
    }
}

@media (max-width: 768px) {
    .ticket-search {
        padding: 16px;
    }
    
    .search-card :deep(.el-card__body) {
        padding: 24px 20px;
    }
    
    .mode-radio {
        justify-content: center;
    }
}
</style>

/* 为城市选择弹出框添加全局样式 */
<style lang="scss">
/* 全局样式覆盖 */
.search-button .ant-btn-icon {
    font-size: 18px;
}

.search-button:not(:disabled):hover {
    background: linear-gradient(135deg, #337ecc 0%, #5daf34 100%) !important;
}

/* 确保城市选择弹出框有足够的层级 */
.city_choose_wrap {
    background: white !important;
    border-radius: 8px !important;
    box-shadow: 0 12px 48px rgba(0, 0, 0, 0.25) !important; /* 更强的阴影 */
    border: 1px solid #e4e7ed !important;
}

</style>

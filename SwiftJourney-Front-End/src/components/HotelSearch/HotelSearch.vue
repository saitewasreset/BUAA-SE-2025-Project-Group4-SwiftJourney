<template>
    <div class="hotel-search">
        <div class="search-card">
            <img class="background-hotel-image" src="../../assets/hotel-image.png" alt="background hotel image">
            <img class="background-hotel-text" src="../../assets/hotel-text.png" alt="background hotel text">
            <p class="background-hotel-order-text">预订酒店</p>
            <div class="SelectCity">
                <CitySelect v-if="isChooseCity" :el="inputRef" @handleCityClick="handleCityClick"/>
                <div class="TargetCity">
                    <p>目的地城市</p>
                    <a-input class="CityInput" v-model:value="hotelQuery.target" id="CityInput"
                    :bordered="false" size="large" placeholder="目的地"
                    @Focus="handleInputFocus()"></a-input>
                </div>
            </div>
            <div class="SelectHotel">
                <div class="TargetHotel">
                    <p>酒店名称(选填)</p>
                    <a-input class="HotelInput" v-model:value="hotelQuery.search"
                    :bordered="false" size="large" placeholder="酒店名称"></a-input>
                </div>
            </div>
            <div class="SelectDate">
                <div class="TargetDate">
                    <div class="SelectDateText">
                        <p>入住</p>
                        <p>--{{ dateRangeNum }}晚--</p>
                        <p>退房</p>
                    </div>
                    <div>
                        <a-range-picker suffix-icon="" id="DatePicker" class="DatePicker" v-model:value="selectedDateRange"
                        size="large" :locale="locale" :format="dateFormat" :bordered="false" :allow-clear="false"
                        :disabled-date="disabledDate" @change="onDateRangeChange"/>
                    </div>
                </div>
            </div>
            <div class="HotelSearchButton">
                <a-button type="primary" size="large" @click="searchHotel">
                    <template #icon>
                        <SearchOutlined />
                    </template>
                    搜索
                </a-button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue';
import { onMounted, onUnmounted } from 'vue';
import type { HotelQuery } from '@/interface/hotelInterface';
import CitySelect from '../TicketSearch/CitySelect/CitySelect.vue';
import { SearchOutlined } from '@ant-design/icons-vue';
import locale from 'ant-design-vue/es/date-picker/locale/zh_CN';
import dayjs from 'dayjs';
import 'dayjs/locale/zh-cn';
import { ElMessage } from 'element-plus';

dayjs.locale('zh-cn');

const today = dayjs();
const nextday = today.add(1, 'day');
const hotelQuery = ref<HotelQuery> ({
    target: '',
    target_type: "city",
    beginDate: formateDate(today),
    endDate: formateDate(nextday),
});

//---------------------------日期---------------------------

const dateFormat = 'YYYY-MM-DD(dddd)';
const selectedDateRange = ref([today, nextday]);

const dateRangeNum = ref<Number>(1);

function disabledDate(current: any) {
    return current && current < dayjs().startOf('day');
}

function onDateRangeChange(dateRange: any, dateStr: []) {
    hotelQuery.value.beginDate = formateDate(dateRange[0]);
    hotelQuery.value.endDate = formateDate(dateRange[1]);
    let startDate = dayjs(dateRange[0]);
    let endDate = dayjs(dateRange[1])
    dateRangeNum.value = endDate.diff(startDate, 'day');
}

function formateDate(date: any) {
    if(!date) return '';
    return dayjs(date).format('YYYY-MM-DD');
}

//---------------------------城市---------------------------

const isChooseCity = ref(false);
const inputRef = ref<HTMLElement | undefined>(undefined);

async function handleInputFocus() {
    const inputElement = document.getElementById('CityInput') as HTMLElement;
    inputRef.value = inputElement;
    isChooseCity.value = false;
    await nextTick();
    isChooseCity.value = true;
}

function handleCityClick(item: Object) {
    const cityName: string = item.cityName;
    hotelQuery.value.target = cityName;
    isChooseCity.value = false;
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
        isChooseCity.value = false;
    }
}

onMounted(() => {
    document.addEventListener('click', handleGlobalClick);
});

onUnmounted(() => {
    document.removeEventListener('click', handleGlobalClick);
});

//---------------------------------------------------------
function searchHotel() {
    if(!checkHotelQuery()) {
        return;
    }
    ElMessage.success('搜索成功');
    console.log(hotelQuery.value);
}

function checkHotelQuery() {
    if(hotelQuery.value.target == '') {
        ElMessage.error('目的地城市不能为空');
        return false;
    }
    if(hotelQuery.value.beginDate == '' || hotelQuery.value.endDate == '') {
        ElMessage.error('入住和离店时间不能为空')
        return false;
    }
    return true;
}

</script>


<style scoped>
.hotel-search {
    width: 1000px;
    min-height: 700px;
    display: flex;
    justify-content: center;
}

.search-card {
    min-width: 1035px;
    height: 200px;
    background: linear-gradient(to bottom right, #40A5F8, #ffffff);
    position: relative; /* 用于支持绝对定位的子元素 */
    border-radius: 8px; /* 圆角大小 */
}

.background-hotel-image {
    position: absolute;
    top: 0;
    left: 0;
    width: 300px;
    height: auto;
}

.background-hotel-text {
    position: absolute;
    top: 10px;
    right: 20px;
    width: 160px;
    height: auto;

}

.background-hotel-order-text {
    text-align: center;
    position: absolute;
    top: 4px;
    left: 20px;
    font-size: 24px;
    color: #ffffff;
}

.SelectCity {
    position: absolute;
    top: 60px;
    left: 20px;
    background-color: #ffffff;
    display: flex;
    border-radius: 8px;
    padding: 15px;
    height: 100px;
}

.SelectCity p {
    margin-left: 11px;
    margin-bottom: 0;
    font-size: 16px;
    color: rgb(189,190,194);
}

.CityInput {
    display: block;
    font-size: 1.25rem;
    font-weight: bolder;
    width: 150px;
}

.SelectHotel {
    position: absolute;
    top: 60px;
    left: 220px;
    background-color: #ffffff;
    display: flex;
    border-radius: 8px;
    padding: 15px;
    height: 100px;
}

.SelectHotel p {
    margin-left: 11px;
    margin-bottom: 0;
    font-size: 16px;
    color: rgb(189,190,194);
}

.HotelInput {
    display: block;
    font-size: 1.25rem;
    font-weight: bolder;
    width: 150px;
}

.SelectDate {
    position: absolute;
    top: 60px;
    left: 420px;
    background-color: #ffffff;
    display: block;
    border-radius: 8px;
    padding: 15px;
    height: 100px;
}

.SelectDate p {
    margin-left: 11px;
    margin-right: 11px;
    margin-bottom: 0;
    font-size: 16px;
    color: rgb(189,190,194);
}

.SelectDateText {
    display: flex;
    justify-content: space-between;
}

.DatePicker {
    width: 450px;
}

::v-deep(#DatePicker) {
    font-size: 1.25rem;
    font-weight: bolder;
    text-align: left;
}

::v-deep(.ant-picker-range-separator) {
  display: none;
}

::v-deep(.DatePicker .ant-picker-input > input) {
  font-size: 1.25rem;
  font-weight: bolder;
  text-align: right;
}

.HotelSearchButton {
    position: absolute;
    top: 60px;
    left: 890px;
    margin-right: 3%;
    margin-left: 3%;
    height: 100px;
    width: 90px;
}

.HotelSearchButton .ant-btn {
    height: 100%;
    width: 100%;
}
</style>
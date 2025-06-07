<template>
    <div class="hotel-search">
        <div class="search-card" :style="isHeadPage ? 'margin-top: 30vh;' : 'margin-top: 15px;'">
            <div class="SelectHotel">
                <div class="TargetHotel">
                    <p>查询预订餐食车次</p>
                    <a-input class="HotelInput" v-model:value="trainNumber"
                    :bordered="false" size="large" placeholder="请输入车次"></a-input>
                </div>
            </div>
            <div class="SelectDate">
                <div class="TargetDate">
                    <div class="SelectDateText">
                        <p>列车始发日期</p>
                    </div>
                    <div>
                        <a-date-picker suffix-icon="" id="DatePicker" class="DatePicker" v-model:value="originDepartureTime"
                        size="large" :locale="locale" :format="dateFormat" :bordered="false" :allow-clear="false"
                        :disabled-date="disabledDate"/>
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
import { ref } from 'vue';
import { SearchOutlined } from '@ant-design/icons-vue';
import { mealApi } from '@/api/MealApi/mealApi';
import locale from 'ant-design-vue/es/date-picker/locale/zh_CN';
import dayjs from 'dayjs';
import 'dayjs/locale/zh-cn';
import { ElMessage } from 'element-plus';
import type { TrainDishInfo } from '@/interface/mealInterface';

const today = dayjs();
dayjs.locale('zh-cn');

const isHeadPage = ref(true);
const trainNumber = ref('');
const originDepartureTime = ref(today);

const dateFormat = 'YYYY-MM-DD(dddd)';

function disabledDate(current: any) {
    return current && ( current < dayjs().startOf('day').subtract(2, 'day') || current > dayjs().startOf('day').add(15, 'day') );
}

async function searchHotel() {
    if(!checkInput()) return;

    await mealApi.dishQuery({
        trainNumber: trainNumber.value.trim(),
        originDepartureTime: originDepartureTime.value.format('YYYY-MM-DD'),
    }).then((res) => {
        if(res.status == 200) {
            if(res.data.code == 200) {
                successGetMeal(res.data.data);
            } else if (res.data.code == 404) {
                ElMessage.error('查询的车次不存在');
            } else if (res.data.code == 403) {
                ElMessage.error('会话无效');
            }
        }
    }).catch((err) => {
        ElMessage.error(err);
    })

}
function checkInput(): boolean {
    if(trainNumber.value.trim() === '') {
        ElMessage.error('请输入您要查询车次');
        return false;
    }
    return true;
}
function successGetMeal(trainDishInfo: TrainDishInfo) {
    console.log(trainDishInfo);
}

</script>

<style scoped>
.hotel-search {
    height: 100%;
    width: 1035px;
}

.search-card {
    min-width: 1035px;
    background: linear-gradient(to bottom right, #40A5F8, #ffffff);
    border-radius: 8px; /* 圆角大小 */
    display: flex;
    justify-content: center;
    padding: 25px;
    gap: 50px;
}


.SelectHotel {
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
    width: 250px;
}

.SelectDate {
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
    width: 280px;
}

::v-deep(#DatePicker) {
    font-size: 1.25rem;
    font-weight: bolder;
    text-align: left;
}

::v-deep(.DatePicker .ant-picker-input > input) {
  font-size: 1.25rem;
  font-weight: bolder;
  text-align: right;
}

.HotelSearchButton {
    height: 100px;
    width: 90px;
}

.HotelSearchButton .ant-btn {
    height: 100%;
    width: 100%;
}

</style>
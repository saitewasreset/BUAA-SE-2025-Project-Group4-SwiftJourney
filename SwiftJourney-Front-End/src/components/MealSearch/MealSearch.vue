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
                <a-button type="primary" size="large" @click="getTrainInfo">
                    <template #icon>
                        <SearchOutlined />
                    </template>
                    搜索
                </a-button>
            </div>
        </div>
        <div v-if="!isHeadPage" class="StationSelected">
            <el-checkbox v-for="(station, index) in stations" :key="index" v-model="stationsShow[station]">
                {{ station }}{{ station == '餐车' ? '' : '站' }}
            </el-checkbox>
        </div>
        <div v-if="!isHeadPage" class="Grid">
            <div style="padding-top: 50px"><SelectCard /></div>
            <el-scrollbar height="550px" class="DishInfo">
                <div v-if="false" class="HotelUnFind">
                    <img class="UnfindImage" src="../../assets/unfind.jpg" alt="unfind">
                    <p style="text-align: center;">没有搜索到符合条件的酒店，请重新输入</p>
                </div>
                <div v-for="(info, index) in dishInfo?.dishes" :key="index">
                    <el-card v-if="info.station ? stationsShow[info.station] : stationsShow['餐车']" class="DishInfoCard" shadow="always">
                        <p class="ShopName">{{ info.shopName }}</p>
                        <el-table class="DishInfoTable" :data="info.dishes" border >
                            <el-table-column label="图片">
                                <template #default="scope">
                                    <div class="FoodImageContainer">
                                        <img class="FoodImage" :src="scope.row.picture" alt="food-image">
                                    </div>
                                </template>
                            </el-table-column>
                            <el-table-column prop="name" label="餐品名称"></el-table-column>
                            <el-table-column v-if="info.shopName == '餐车'" label="提供时段">
                                <template #default="scope">
                                    <div>{{ lunchChange(scope.row.availableTime) }}</div>
                                </template>
                            </el-table-column>
                            <el-table-column v-if="info.shopName == '餐车'" prop="type" label="类别"></el-table-column>
                            <el-table-column prop="price" label="价格" sortable></el-table-column>
                            <el-table-column label="操作">
                                <template #default="scope">
                                    <el-tooltip :content="dishInfo?.reason" :disabled="dishInfo?.canBooking">
                                        <el-button class="OrderButton" type="primary" 
                                        :disabled="!dishInfo?.canBooking"
                                        @click="info.shopName == '餐车' ? 
                                        orderDish(info.shopName, scope.row) : orderMeal(info.shopName, scope.row, info.station)">
                                        订
                                        </el-button>
                                    </el-tooltip>
                                </template>
                            </el-table-column>
                        </el-table>
                    </el-card>
                </div>
            </el-scrollbar>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, h, ref } from 'vue';
import { SearchOutlined } from '@ant-design/icons-vue';
import { mealApi } from '@/api/MealApi/mealApi';
import { TicketServiceApi } from '@/api/TicketServiceApi/TicketServiceApi';
import locale from 'ant-design-vue/es/date-picker/locale/zh_CN';
import dayjs from 'dayjs';
import 'dayjs/locale/zh-cn';
import { ElMessage, ElMessageBox, ElOption, ElSelect } from 'element-plus';
import type { TrainDishInfo, MealInfo, Takeaway, TakeawayDishInfo } from '@/interface/mealInterface';
import type { TrainScheduleInfo } from '@/interface/ticketServiceInterface';
import SelectCard from './MealOrderCard.vue'
import { useMealOrderStore } from '@/stores/mealOrder';

const mealOrderStore = useMealOrderStore();

const today = dayjs();
dayjs.locale('zh-cn');

const isHeadPage = ref(true);
const trainNumber = ref('');
const originDepartureTime = ref(today);

const dateFormat = 'YYYY-MM-DD(dddd)';

function disabledDate(current: any) {
    return current && ( current < dayjs().startOf('day').subtract(2, 'day') || current > dayjs().startOf('day').add(15, 'day') );
}


async function getTrainInfo() {
    if(!checkInput()) return;
    
    await TicketServiceApi.trainSchedule({
        trainNumber: trainNumber.value.trim(),
        departureDate: originDepartureTime.value.format('YYYY-MM-DD'),
    }).then((res) => {
        if(res.status == 200) {
            if(res.data.code == 200) {
                getMail(res.data.data);
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

const departureTime = ref('');
const stations = ref<string[]>([]);
const stationsShow = ref<{[stations: string]: boolean}>({});


async function getMail(trainInfo: TrainScheduleInfo) {
    departureTime.value = trainInfo.originDepartureTime;
    const tepStations: string[] = [];
    const tepStationsMap: {[stations: string]: boolean} = {};
    trainInfo.route.forEach((value) => {
        tepStations.push(value.stationName);
        tepStationsMap[value.stationName] = true;
    })
    tepStations.push('餐车');
    tepStationsMap['餐车'] = true;
    stations.value = tepStations;
    stationsShow.value = tepStationsMap;

    await mealApi.dishQuery({
        trainNumber: trainNumber.value.trim(),
        originDepartureTime: departureTime.value,
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
const dishInfo = ref<MealInfo>();
function successGetMeal(trainDishInfo: TrainDishInfo) {
    const tepInfo: MealInfo = {
        trainNumber: trainDishInfo.trainNumber,
        originDepartureTime: trainDishInfo.originDepartureTime,
        terminalArrivalTime: trainDishInfo.terminalArrivalTime,
        canBooking: trainDishInfo.canBooking,
        reason: trainDishInfo.reason,
        dishes: []
    }

    const canche: Takeaway = {
        shopName: '餐车',
        dishes: trainDishInfo.dishes,
    }
    tepInfo.dishes.push(canche);

    for(const station in trainDishInfo.takeaway) {
        trainDishInfo.takeaway[station].forEach((value) => {
            const tepShopInfo: Takeaway = {
                ...value,
                station: station, 
            }
            tepInfo.dishes.push(tepShopInfo);
        })
    }

    mealOrderStore.deleteAll();
    dishInfo.value = tepInfo;
    isHeadPage.value = false;
}

const orderDish = (shopName: string, food: TakeawayDishInfo) => {
    if(food.availableTime?.length == 1) {
        orderMeal(shopName, food, undefined, food.availableTime[0]);
    } else {
        const select = ref<'lunch' | 'dinner'>('lunch');
        ElMessageBox({
            title: '请选择用餐时段',
            message: () => 
            h(ElSelect, {
                modelValue: select.value,
                'onUpdate:modelValue': (val: 'lunch' | 'dinner') => {
                    select.value = val
                },
                style: { width: '100px' },
            }, [
                h(ElOption, {
                    key: 'lunch',
                    label: '午餐',
                    value: 'lunch',
                }),
                h(ElOption, {
                    key: 'dinner',
                    label: '晚餐',
                    value: 'dinner',
                })
            ]),
            type: 'info',
            confirmButtonText: '确定',
        }).then(() => {
            orderMeal(shopName, food, undefined, select.value);
        }).catch((err) => {
            console.log(err);
        })
    }
}

const orderMeal = (shopName: string, food: TakeawayDishInfo, station?: string, dishTime?: 'lunch' | 'dinner') => {
    if(!mealOrderStore.add(trainNumber.value, departureTime.value, shopName, food, station, dishTime)){
        ElMessage.error('不可在同一订单为不同车次预订餐品');
    } else {
        ElMessage.success('加入预订列表成功，可在预定列表中修改数量');
    }
}

const lunchChangeTab = {
    lunch: '午餐',
    dinner: '晚餐',
}

const lunchChange = (time: ('lunch' | 'dinner')[]) => {
    let str = ''
    time.forEach((value) => {
        str = str + lunchChangeTab[value] + ' ';
    })
    return str;
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
    padding: 15px;
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



.StationSelected {
    display: flex;
    justify-content: center;
    padding: 5px;
    flex-wrap: wrap;
}


.Grid {
    display: grid;
    display: flex;
    justify-content: space-between;
    gap: 5px;
}

.DishInfo {
    width: 700px;
    padding-right: 20px;
}

.DishInfoCard {
    margin-bottom: 20px;
}
.ShopName {
    text-align: center;
    font-size: 1.25rem;
    font-weight: bold;
}
::v-deep(.el-table .el-table__cell) {
    text-align: center;
}
.DishInfoTable {
    resize: none;
}
::v-deep(.OrderButton span) {
    font-weight: bold;
}

.FoodImageContainer {
    display: flex;
    justify-content: center;
    align-items: center;
    width: 100px; 
    height: 100px;
    border-radius: 8px;
    overflow: hidden;
    cursor: pointer;
}
.FoodImage { 
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.3s ease;
}
.FoodImage:hover {
    transform: scale(1.1);
}

</style>
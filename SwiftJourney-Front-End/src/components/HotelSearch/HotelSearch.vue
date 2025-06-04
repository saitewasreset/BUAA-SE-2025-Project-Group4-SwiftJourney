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
        <div class="Grid">
            <div class="Selected">
                <p class="title">筛选</p>
                <p class="sub-title">最低价格 {{ moneyFormat(moneyValue) }}</p>
                <el-slider class="rating-slider" range v-model="moneyValue" :marks="moneyMarks" :show-tooltip="false" />
                <p class="sub-title" style="margin-top: 25px;">房型</p>
                <el-checkbox class="CheckBox" v-model="roomShowAll" label="全部房型" 
                @change="toggleRoomShowAll"/>
                <el-checkbox class="CheckBox" v-model="roomTypeFree" label="只看剩余房型" />
                <el-checkbox class="CheckBox" v-for="(key, index) in roomList" :key="index" v-model="key.isShow" :label="key.type" />
                <p class="sub-title">评分 {{ ratingFormat(ratingValue) }}</p>
                <el-slider class="rating-slider" v-model="ratingValue" :show-tooltip="false" />
                <p class="sub-title">评论数 {{ ratingCountFormat(ratingCountValue) }}</p>
                <el-slider class="rating-slider" v-model="ratingCountValue" :marks="ratingCountMarks" :show-tooltip="false" />
            </div>
            <el-scrollbar height="540px" class="HotelInfo">
                <div v-for="(info, index) in hotelGInfoWRoom" :key="index">
                    <el-card v-if="isCardShow(info.rating, moneyDisplays[index], info.ratingCount) && roomTypeDisplays[index] != ''" class="HotelInfoCard" shadow="always">
                        <div class="HotelImageContainer">
                            <img class="HotelImage" :src="info.picture" alt="hotel-image" @click="goToDetail(info)">
                        </div>
                        <div class="HotelInfoShow">
                            <p class="HotelName">{{ info.name }}</p>
                            <p class="HotelGeneralInfo">{{ info.info }}</p>
                            <p class="HotelRoomType"> {{ roomTypeDisplays[index] }} </p>
                            <div class="HotelRateContainer">
                                <el-rate class="HotelRate" v-model="info.rating" disabled show-score 
                                text-color="#ff9900" size="large" score-template="{value}"/>
                                <p class="RatingNum">{{ info.ratingCount }}人评论</p>
                            </div>
                        </div>
                        <div class="RightInfoShow">
                            <div class="HotelMoney">
                                <p class="p1">SC</p>
                                <p class="p2">{{ moneyDisplays[index] }}</p>
                                <p class="p1">起</p>
                            </div>
                            <p class="LiveNum">{{ info.totalBookings }}人住过</p>
                            <el-button class="DetailButton" type="primary" size="large" @click="goToDetail(info)">查看详情</el-button>
                        </div>
                    </el-card>
                </div>
            </el-scrollbar>
        </div>
    </div>
    <div class="fixed-order-card"><HotelOrderCard v-if="isOrderShow" /></div>
    <div class="fixed-icon">
        <el-button class="FixedButton" type="primary" circle @click="showRoomOrder">
            <el-icon><Goods /></el-icon>
        </el-button>
    </div>
</template>

<script setup lang="ts">
import { ref, nextTick, reactive, computed, watch } from 'vue';
import { onMounted, onUnmounted } from 'vue';
import type { HotelQuery, HotelGeneralInfo, HotelGInfoWRoom, HotelOrderQuery, HotelRoomDetailInfo } from '@/interface/hotelInterface';
import CitySelect from '../TicketSearch/CitySelect/CitySelect.vue';
import { SearchOutlined } from '@ant-design/icons-vue';
import locale from 'ant-design-vue/es/date-picker/locale/zh_CN';
import dayjs from 'dayjs';
import 'dayjs/locale/zh-cn';
import { ElMessage } from 'element-plus';
import { hotelApi } from '@/api/HotelApi/hotelApi';
import { useRouter } from 'vue-router';

dayjs.locale('zh-cn');

const today = dayjs();
const nextday = today.add(1, 'day');
const hotelQuery = ref<HotelQuery> ({
    target: '',
    targetType: "city",
    beginDate: formateDate(today),
    endDate: formateDate(nextday),
});

//---------------------------日期---------------------------
const beginDate = ref(hotelQuery.value.beginDate);
const endDate = ref(hotelQuery.value.endDate);

const dateFormat = 'YYYY-MM-DD(dddd)';
const selectedDateRange = ref([today, nextday]);

const dateRangeNum = ref<number>(1);

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
async function searchHotel() {
    if(!checkHotelQuery()) {
        return;
    }
    await hotelApi.hotelQuery(hotelQuery.value)
    .then((res) => {
        if(res.status == 200){
            if(res.data.code == 200) {
                successSearchHotel(res.data.data);
            }  else if (res.data.code == 403) {
                ElMessage.error('会话无效');
            } else if (res.data.code == 404) {
                ElMessage.error('查询的目标城市/火车站不存在');
            } else if (res.data.code == 21001) {
                ElMessage.error('入住/离开日期不合法：离开比入住早；只设置其中一个；入住时间超过 7 天');
            } else {
                throw new Error(res.data.message);
            }
        }
    }) .catch ((error) => {
        ElMessage.error(error);
        console.error(error);
    })
}

function checkHotelQuery() {
    if(hotelQuery.value.target == '') {
        ElMessage.error('目的地城市不能为空');
        return false;
    }
    hotelQuery.value.target = hotelQuery.value.target + '市';
    if(hotelQuery.value.beginDate == '' || hotelQuery.value.endDate == '') {
        ElMessage.error('入住和离店时间不能为空');
        return false;
    }
    if(dateRangeNum.value > 7) {
        ElMessage.error('入住时间不能超过7晚');
        return false;
    }
    return true;
}

//---------------------------------显示结果-----------------------------------
const hotelGInfoWRoom = ref<HotelGInfoWRoom[]>([]);
const roomSet = new Set<string>();
const roomList = ref<{type: string, isShow: boolean}[]>([]);
const roomMapIndex = new Map<string, number>();
const roomTypeFree = ref(false);
const roomShowAll = ref(true);
// 计算属性，判断是否所有房间都显示且 roomTypeFree 为 false
const roomShowAllComputed = computed(() => {
    return !roomList.value.some(room => !room.isShow) && !roomTypeFree.value;
});
// 监听 roomList 或 roomTypeFree 的变化，更新 roomShowAll
watch([roomList, roomTypeFree], () => {
    roomShowAll.value = roomShowAllComputed.value;
}, { deep: true });
// 方法，用于手动切换 roomShowAll
const toggleRoomShowAll = (value: boolean) => {
    roomTypeFree.value = false;
    roomList.value.forEach(room => {
        room.isShow = true;
    });
    roomShowAll.value = true;
};


async function successSearchHotel(hotelGeneralInfo: HotelGeneralInfo[]) {
    beginDate.value = hotelQuery.value.beginDate;
    endDate.value = hotelQuery.value.endDate;
    hotelGInfoWRoom.value = [];
    roomSet.clear();
    roomList.value = [];
    for(let tepInfo of hotelGeneralInfo) {
        let map = await hotelDetailRoom(tepInfo.hotelId)
        let tepInfoWRoom: HotelGInfoWRoom = {
            ...tepInfo,
            roomTypeMap: map,
        }
        hotelGInfoWRoom.value.push(tepInfoWRoom);
        roomSet.forEach((key) => {
            if (!roomMapIndex.has(key)) {
                roomList.value.push({
                    type: key,
                    isShow: true,
                });
                roomMapIndex.set(key, roomList.value.length - 1);
            }
        })
    }
}

async function hotelDetailRoom(id: string) {
    return hotelApi.hotelOrderInfo({
        hotelId: id, beginDate: hotelQuery.value.beginDate, endDate: hotelQuery.value.endDate} as HotelOrderQuery
    ).then((res) => {
        if(res.status == 200) {
            if(res.data.code == 200) {
                let myMap = new Map(Object.entries(res.data.data as { [key: string]: HotelRoomDetailInfo }));
                myMap.forEach((value, key) => {
                    roomSet.add(key);
                })
                return myMap;
            }  else {
                throw new Error(res.data.message);
            }
        }
    }).catch((error) => {
        ElMessage.error(error);
        return new Map<string, HotelRoomDetailInfo>();
    })
}

function MapToRoomType(map: Map<string, HotelRoomDetailInfo>, flag: boolean, roomStatusList: {type: string, isShow: boolean}[]) {
    let roomType = '';
    for(const [key, value] of map) {
        let index = roomMapIndex.get(key);
        if(index != null && roomStatusList[index].type == key && roomStatusList[index].isShow) {
            if(flag && value.remainCount > 0) {
                roomType = roomType + key + ' ';
            } else if(!flag) {
                roomType = roomType + key + ' ';
            }
        }
    }
    return roomType.trim();
}

function minMoney(map: Map<string, HotelRoomDetailInfo>, flag: boolean, roomStatusList: {type: string, isShow: boolean}[]) {
    let minMoney = -1;
    for(const [key, value] of map) {
        let index = roomMapIndex.get(key);
        if(index != null && roomStatusList[index].type == key && roomStatusList[index].isShow && (minMoney == -1 || minMoney > value.price)) {
            if(flag && value.remainCount > 0) {
                minMoney = value.price;
            } else if(!flag) {
                minMoney = value.price;
            }
        }
    }
    return minMoney;
}

//-------------------------------详情-----------------------------------
const router = useRouter();

function goToDetail(info: HotelGInfoWRoom) {
    const routeUrl = router.resolve({
        name: 'hotelDetail',
        params: { id: info.hotelId },
        query: {
          beginDate: beginDate.value,
          endDate: endDate.value,
        }
      });
    window.open(routeUrl.href, '_blank');
}


//-------------------------------筛选------------------------------------
const ratingValue = ref<number>(0);

function ratingFormat(rate: number) {
    if(rate == 0) {
        return ''
    }
    return (rate / 20).toFixed(1) + '+';
}

function isRatingShow(rate: number) {
    return ratingValue.value <= rate * 20;
}

const moneyValue = ref<number[]>([0, 100]);
interface Mark {
  label: string
}
type Marks = Record<number, Mark | string>
const moneyMarks = reactive<Marks>({
  0: '0',
  100: '1500以上',
})

function moneyFormat(money: number[]) {
    if(money[0] == 0 && money[1] == 100) {
        return '';
    } else if (money[1] == 100) {
        return 'SC ' + Math.round(money[0] * 0.3) * 50 + '+';
    } else {
        return 'SC ' + Math.round(money[0] * 0.3) * 50 + ' ~ ' + 'SC ' + Math.round(money[1] * 0.3) * 50;
    }
}

function isMoneyShow(money: number) {
    if(moneyValue.value[1] == 100) {
        return money >= Math.round(moneyValue.value[0] * 0.3) * 50;
    } else {
        return money >= Math.round(moneyValue.value[0] * 0.3) * 50 && money <= Math.round(moneyValue.value[1] * 0.3) * 50;
    }
}

const ratingCountValue = ref<number>(0);
const ratingCountMarks = reactive<Marks>({
    0: '0',
    500: '500以上'
})

function ratingCountFormat(ratingCount: number) {
    if(ratingCount == 0) {
        return ''
    }
    return Math.round(ratingCount * 0.2) * 25 + '+';
}

function isRatingCountShow(ratingCount: number) {
    return Math.round(ratingCountValue.value * 0.2) * 25 <= ratingCount;
}


function isCardShow(rate: number, money: number, ratingCount: number) {
    return isRatingShow(rate) && isMoneyShow(money) && isRatingCountShow(ratingCount);
}

const roomTypeDisplays = computed(() => {
    return hotelGInfoWRoom.value.map(info => 
        MapToRoomType(info.roomTypeMap, roomTypeFree.value, roomList.value)
    );
});

const moneyDisplays = computed(() =>{
    return hotelGInfoWRoom.value.map(info =>
        minMoney(info.roomTypeMap, roomTypeFree.value, roomList.value)
    )
})
//-----------------------------------roomOrder-------------------------------
import HotelOrderCard from '@/components/HotelSearch/HotelOrderCard.vue';
const isOrderShow = ref<boolean>(false);
function showRoomOrder() {
    isOrderShow.value = !isOrderShow.value;
}

//-----------------------------------debug-----------------------------------
/*import hotelImage from '../../assets/hotel.jpg'
const debugdataMap = new Map<string, HotelRoomDetailInfo>();
const debugHotelRoomDetailInfo1: HotelRoomDetailInfo = {
    capacity: 1,
    remainCount: 5,
    price: 200,
}
debugdataMap.set('标间', debugHotelRoomDetailInfo1);
const debugHotelRoomDetailInfo2: HotelRoomDetailInfo = {
    capacity: 2,
    remainCount: 3,
    price: 796,
}
debugdataMap.set('大床房', debugHotelRoomDetailInfo2);
const debugHotelRoomDetailInfo3: HotelRoomDetailInfo = {
    capacity: 2,
    remainCount: 0,
    price: 599,
}
debugdataMap.set('双床房', debugHotelRoomDetailInfo3);
const debugHotelRoomDetailInfo4: HotelRoomDetailInfo = {
    capacity: 3,
    remainCount: 1,
    price: 999,
}
debugdataMap.set('三人间', debugHotelRoomDetailInfo4);
const debugHotelRoomDetailInfo5: HotelRoomDetailInfo = {
    capacity: 3,
    remainCount: 0,
    price: 4000,
}
debugdataMap.set('总统套房', debugHotelRoomDetailInfo5);

const debugHoteldata1: HotelGInfoWRoom = {
    hotelId: '11111',
    name: '桔子水晶酒店',
    picture: hotelImage,
    rating: 4.8,
    ratingCount: 365,
    totalBookings: 1245,
    price: 200,
    roomTypeMap: debugdataMap,
    info: "本酒店距离火车站步行约5分钟，配备免费Wi-Fi与早餐。"
}

const debugdataMap2 = new Map<string, HotelRoomDetailInfo>();
const debugHotelRoomDetailInfo6: HotelRoomDetailInfo = {
    capacity: 3,
    remainCount: 0,
    price: 999,
}
debugdataMap2.set('三人间', debugHotelRoomDetailInfo6);
const debugHotelRoomDetailInfo7: HotelRoomDetailInfo = {
    capacity: 2,
    remainCount: 2,
    price: 496,
}
debugdataMap2.set('大床房', debugHotelRoomDetailInfo7);
const debugHotelRoomDetailInfo8: HotelRoomDetailInfo = {
    capacity: 1,
    remainCount: 1,
    price: 159,
}
debugdataMap2.set('标间', debugHotelRoomDetailInfo8);
const debugHoteldata2: HotelGInfoWRoom = {
    hotelId: '11112',
    name: '日升大酒店',
    picture: hotelImage,
    rating: 4.5,
    ratingCount: 86,
    totalBookings: 264,
    price: 159,
    roomTypeMap: debugdataMap2,
    info: "本酒店距离火车站步行约5分钟，配备免费Wi-Fi与早餐。"
}

hotelGInfoWRoom.value.push(debugHoteldata1);
hotelGInfoWRoom.value.push(debugHoteldata2);
hotelGInfoWRoom.value.push(debugHoteldata1);
hotelGInfoWRoom.value.push(debugHoteldata1);

const debugRoomSet = new Set<string>();
debugRoomSet.add('标间');
debugRoomSet.add('大床房');
debugRoomSet.add('双床房');
debugRoomSet.add('三人间');
debugRoomSet.add('总统套房');

roomList.value = [];
debugRoomSet.forEach((key) => {
    roomList.value.push({
        type: key,
        isShow: true,
    });
})
roomList.value.forEach((key, index) => {
    roomMapIndex.set(key.type, index);
})*/
</script>


<style scoped>
.hotel-search {
    height: 100%;
    width: 1035px;
}

.search-card {
    min-width: 1035px;
    height: 180px;
    background: linear-gradient(to bottom right, #40A5F8, #ffffff);
    position: relative; /* 用于支持绝对定位的子元素 */
    border-radius: 8px; /* 圆角大小 */
    margin-top: 70px;
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

.Grid {
    margin-top: 30px;
    display: flex;
    justify-content: space-between;
    gap: 5px;
}

.Selected {
    margin-left: 10px;
    width: 280px;
}
.title {
    font-size: 1.25rem;
    font-weight: bold;
    margin-bottom: 0;
}
.sub-title {
    font-size: 1rem;
    margin-bottom: 0;
    margin-top: 5px;
}

.rating-slider {
    width: 220px;
}
::v-deep(.CheckBox .el-checkbox__label) {
    font-size: 16px; 
    width: 85px;   
}

.HotelInfo {
    width: 730px;
}

.HotelInfoCard {
    margin-bottom: 25px;
    width: 700px;
    height: 160px;
    position: relative;
    border-radius: 8px;
}

.HotelImageContainer {
    position: absolute;
    top: 20px;
    left: 20px;
    display: flex;
    justify-content: center;
    align-items: center;
    width: 120px; 
    height: 120px;
    border-radius: 8px;
    overflow: hidden;
    cursor: pointer;
}
.HotelImage { 
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.3s ease;
}
.HotelImage:hover {
    transform: scale(1.1);
}

.HotelInfoShow {
    position: absolute;
    top: 20px;
    left: 160px;
}
.HotelName {
    font-size: 1.75rem;
    font-weight: bold;
    margin-bottom: 0;
}
.HotelRateContainer {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    gap: 15px;
}
::v-deep(.el-rate--large) {
    height: 0;
}
::v-deep(.el-rate) {
    --el-rate-font-size: 16px;
    --el-rate-icon-size: 20px;
}

.RatingNum {
    font-size: 16px;
    color: rgb(189,190,194);
    margin-bottom: 0;
}

.HotelGeneralInfo {
    font-size: 14px;
    margin-top: 0;
    margin-bottom: 0;
    width: 400px;
    white-space: nowrap; /* 防止文本换行 */
    overflow: hidden; /* 隐藏溢出的部分 */
    text-overflow: ellipsis; /* 使用省略号表示被隐藏的文本 */
}

.HotelRoomType {
    font-size: 18px;
    margin-top: 0;
    margin-bottom: 0;
    width: 400px;
    white-space: nowrap; /* 防止文本换行 */
    overflow: hidden; /* 隐藏溢出的部分 */
    text-overflow: ellipsis; /* 使用省略号表示被隐藏的文本 */
}

.RightInfoShow {
    position: absolute;
    top: 20px;
    right: 20px;
}
.HotelMoney {
    display: flex;
    align-items: baseline;
    justify-content: end;
    gap: 3px;
}
.HotelMoney .p1 {
    font-size: 16px;
    color: red;
    margin-bottom: 0;
}
.HotelMoney .p2 {
    font-size: 1.75rem;
    font-weight: bold;
    color: red;
    margin-bottom: 0;
}
.LiveNum {
    display: flex;
    justify-content: end;
    font-size: 16px;
    color: rgb(189,190,194);
    margin-bottom: 5px;
    margin-top: 5px;
}
.DetailButton {
    font-size: 20px;
    font-weight: bold;
}

.fixed-order-card {
    position: fixed;
    bottom: 200px;
    right: 30px;
    z-index: 1000;
}
.fixed-icon {
    position: fixed;
    bottom: 140px; /* 距离窗口底部的距离 */
    right: 150px; /* 距离窗口右侧的距离 */
    z-index: 1000; /* 确保图标在其他元素之上 */
    cursor: pointer;
}
::v-deep(.FixedButton span){
    font-size: 2rem;
}
.FixedButton {
    width: 50px;
    height: 50px;
}
</style>
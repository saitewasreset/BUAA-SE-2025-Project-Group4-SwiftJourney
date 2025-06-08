<template>
    <div class="hotel-search">
        <div class="search-card" :style="isHeadPage ? 'margin-top: 30vh;' : 'margin-top: 15px;'">
            <img class="background-hotel-image" src="../../assets/hotel-image.png" alt="background hotel image">
            <img class="background-hotel-text" src="../../assets/hotel-text.png" alt="background hotel text">
            <p class="background-hotel-order-text">预订酒店</p>
            
            <!-- 使用容器包装所有表单元素 -->
            <div class="form-container">
                <div class="SelectCity">
                    <!-- 使用 Teleport 将 SelectCard 渲染到 body 顶层 -->
                    <Teleport to="body">
                        <SelectCard 
                            v-if="isChooseCity" 
                            :el="inputRef" 
                            :input="cityInput" 
                            @handleCityClick="handleCityClick"
                            :style="selectCardStyle"
                        />
                    </Teleport>
                    <div class="TargetCity">
                        <p>目的地城市/车站</p>
                        <a-input 
                            ref="cityInputRef"
                            class="CityInput" 
                            v-model:value="hotelQuery.target" 
                            id="CityInput"
                            :bordered="false" 
                            size="large" 
                            placeholder="目的地" 
                            @input="handleCityInput"
                            @Focus="handleInputFocus"
                            @compositionupdate="handleCompositionUpdate"
                        />
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
        <div v-if="!isHeadPage" class="Grid">
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
            <el-scrollbar height="500px" class="HotelInfo">
                <div v-if="hotelGInfoWRoom.length == 0" class="HotelUnFind">
                    <img class="UnfindImage" src="../../assets/unfind.jpg" alt="unfind">
                    <p style="text-align: center;">没有搜索到符合条件的酒店，请重新输入</p>
                </div>
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
                            <p class="LiveNum" style="margin-right: 10px;">{{ info.totalBookings }}人住过</p>
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
import { ref, nextTick, reactive, computed, watch, Teleport } from 'vue';
import { onMounted, onUnmounted } from 'vue';
import { useRoute } from 'vue-router'; // 添加这行
import type { HotelQuery, HotelGeneralInfo, HotelGInfoWRoom, HotelOrderQuery, HotelRoomDetailInfo } from '@/interface/hotelInterface';
import SelectCard from '../SelectCard/SelectCard.vue'
import { SearchOutlined } from '@ant-design/icons-vue';
import locale from 'ant-design-vue/es/date-picker/locale/zh_CN';
import dayjs from 'dayjs';
import 'dayjs/locale/zh-cn';
import { ElMessage } from 'element-plus';
import { hotelApi } from '@/api/HotelApi/hotelApi';
import { useRouter } from 'vue-router';

dayjs.locale('zh-cn');

const route = useRoute(); // 添加这行
const today = dayjs();
const nextday = today.add(1, 'day');

// 修改初始化逻辑，从路由参数获取数据
const initHotelQuery = () => {
  const query = route.query;
  const targetType = query.targetType as string;
  return {
    target: (query.target as string) || '',
    targetType: (targetType === "station" ? "station" : "city") as "city" | "station",
    beginDate: (query.beginDate as string) || formateDate(today),
    endDate: (query.endDate as string) || formateDate(nextday),
    search: (query.search as string) || '',
  };
};

const hotelQuery = ref<HotelQuery>(initHotelQuery());

//---------------------------首页查询页切换-----------------
const isHeadPage = ref(true);

//---------------------------日期---------------------------
const beginDate = ref(hotelQuery.value.beginDate);
const endDate = ref(hotelQuery.value.endDate);

const dateFormat = 'YYYY-MM-DD(dddd)';

// 修改日期选择器初始值
const initDateRange = () => {
  if (hotelQuery.value.beginDate && hotelQuery.value.endDate) {
    return [dayjs(hotelQuery.value.beginDate), dayjs(hotelQuery.value.endDate)];
  }
  return [today, nextday];
};

const selectedDateRange = ref(initDateRange());

// 计算初始住宿天数
const initDateRangeNum = () => {
  if (hotelQuery.value.beginDate && hotelQuery.value.endDate) {
    const start = dayjs(hotelQuery.value.beginDate);
    const end = dayjs(hotelQuery.value.endDate);
    return end.diff(start, 'day');
  }
  return 1;
};

const dateRangeNum = ref<number>(initDateRangeNum());

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
const cityInputRef = ref();
const selectCardStyle = ref({});

async function handleInputFocus() {
    const inputElement = document.getElementById('CityInput') as HTMLElement;
    inputRef.value = inputElement;
    
    // 计算输入框的位置并设置 SelectCard 的绝对定位
    if (inputElement) {
        const rect = inputElement.getBoundingClientRect();
        selectCardStyle.value = {
            position: 'fixed',
            top: `${rect.bottom + 5}px`,
            left: `${rect.left}px`,
            zIndex: 9999,
            background: 'white',
            border: '1px solid #d9d9d9',
            borderRadius: '6px',
            boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
            maxHeight: '300px',
            overflow: 'auto'
        };
    }
    
    isChooseCity.value = false;
    await nextTick();
    isChooseCity.value = true;
}

function handleCityClick(item: string) {
    const cityName: string = item;
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

onMounted(async () => {
    document.addEventListener('click', handleGlobalClick);
    await generalStore.init();
    await checkAndAutoSearch();
});

onUnmounted(() => {
    document.removeEventListener('click', handleGlobalClick);
});

//---------------------------------------------------------
import { useGeneralStore } from '@/stores/general';
const generalStore = useGeneralStore();

async function searchHotel() {
    if(!checkHotelQuery()) {
        return;
    }

    let result = generalStore.checkInputString(hotelQuery.value.target);
    if(result == undefined) {
        ElMessage.error('请输入正确的城市名/站名');
        return;
    }

    let postQuery: HotelQuery = {
        beginDate: hotelQuery.value.beginDate,
        endDate: hotelQuery.value.endDate,
        target: result.target,
        targetType: result.targetType,
        search: hotelQuery.value.search,
    };
    await hotelApi.hotelQuery(postQuery)
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
    hotelQuery.value.target = hotelQuery.value.target.trim();
    if(hotelQuery.value.target == '') {
        ElMessage.error('目的地不能为空');
        return false;
    }
    hotelQuery.value.target = hotelQuery.value.target;
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

//---------------------------添加自动搜索逻辑---------------------------
const checkAndAutoSearch = async () => {
  // 检查是否有查询参数，如果有则自动执行搜索
  const query = route.query;
  if (query.target) {
    
    // 等待组件完全挂载后再执行搜索
    await nextTick();
    
    // 直接调用搜索函数
    try {
      await searchHotel();
      console.log('自动搜索执行完成');
    } catch (error) {
      console.error('自动搜索失败:', error);
    }
  } else {
    console.log('没有检测到查询参数，跳过自动搜索');
  }
};

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
    roomList.value = [];
    roomSet.clear();
    roomMapIndex.clear();
    isHeadPage.value = false;

    for(let tepInfo of hotelGeneralInfo) {
        let map = await hotelDetailRoom(tepInfo.hotelId)
        let tepInfoWRoom: HotelGInfoWRoom = {
            ...tepInfo,
            roomTypeMap: map || new Map<string, HotelRoomDetailInfo>(),
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

//----------------------------------城市拼音推荐-----------------------------
const cityInput = ref('');

watch(() => hotelQuery.value.target, (newValue) => {
    if (newValue) {
        cityInput.value = newValue;
    } else {
        cityInput.value = '';
    }
});

const handleCityInput = () => {
    cityInput.value = hotelQuery.value.target;
};

const handleCompositionUpdate = (event: CompositionEvent) => {
    cityInput.value = hotelQuery.value.target + event.data.toLowerCase();
};



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
    width: 100%;
    max-width: 1400px;
    margin: 0 auto;
    padding: 0 20px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    height: 100%;
}

.search-card {
    min-width: 100%;
    max-width: 1200px;
    margin: 0 auto;
    height: 180px;
    background: linear-gradient(to bottom right, #40A5F8, #ffffff);
    position: relative;
    border-radius: 8px;
    flex-shrink: 0;
}

/* 保持搜索卡片原样 */
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

/* 新增的容器样式 */
.form-container {
    position: absolute;
    top: 60px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 15px;
    width: fit-content;
}

.SelectCity {
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
    height: 100px;
    width: 90px;
}

.HotelSearchButton .ant-btn {
    height: 100%;
    width: 100%;
}

/* 美化网格布局 */
.Grid {
    margin-top: 20px;
    display: flex;
    justify-content: flex-start;
    gap: 15px; /* 减少间距 */
    width: 100%;
    margin-left: auto;
    margin-right: auto;
    padding: 0 10px;
    flex: 1;
}

/* 美化筛选面板 - 减小宽度并去除滚动 */
.Selected {
    width: 300px;
    flex-shrink: 0;
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.95), rgba(240, 248, 255, 0.95));
    border-radius: 12px;
    padding: 20px;
    box-shadow: 0 8px 20px rgba(59, 130, 246, 0.15);
    border: 2px solid rgba(147, 197, 253, 0.3);
    height: 500px;
    overflow: hidden;
    backdrop-filter: blur(10px);
}

/* 美化酒店信息区域 */
.HotelInfo {
    flex: 1;
    min-width: 0;
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.95), rgba(240, 248, 255, 0.95));
    border-radius: 16px;
    box-shadow: 0 10px 25px rgba(59, 130, 246, 0.15);
    border: 2px solid rgba(147, 197, 253, 0.3);
    overflow: hidden;
    backdrop-filter: blur(10px);
    height: 500px;
}

.title {
    font-size: 1.1rem;
    font-weight: 700;
    margin-bottom: 10px;
    color: #1e40af;
    text-align: center;
    position: relative;
}

.title::after {
    content: '';
    position: absolute;
    bottom: -4px;
    left: 50%;
    transform: translateX(-50%);
    width: 30px;
    height: 2px;
    background: linear-gradient(90deg, #3b82f6, #2563eb);
    border-radius: 2px;
}

.sub-title {
    font-size: 0.85rem;
    margin-bottom: 6px;
    margin-top: 12px;
    color: #2563eb;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 4px;
}

.sub-title::before {
    content: '🔍';
    font-size: 0.75rem;
}

.rating-slider {
    width: 90%;
    margin: 6px 0 10px 0;
}

::v-deep(.rating-slider .el-slider__runway) {
    background: linear-gradient(90deg, #dbeafe, #bfdbfe);
    border-radius: 4px;
    height: 4px;
}

::v-deep(.rating-slider .el-slider__bar) {
    background: linear-gradient(90deg, #3b82f6, #2563eb);
    border-radius: 4px;
}

::v-deep(.rating-slider .el-slider__button) {
    background: white;
    border: 2px solid #3b82f6;
    box-shadow: 0 2px 8px rgba(59, 130, 246, 0.3);
    width: 14px;
    height: 14px;
    transition: all 0.3s ease;
}

::v-deep(.rating-slider .el-slider__button:hover) {
    transform: scale(1.1);
    box-shadow: 0 3px 12px rgba(59, 130, 246, 0.4);
}

/* 美化复选框 */
::v-deep(.CheckBox) {
    margin-bottom: 5px;
    transition: all 0.3s ease;
}

::v-deep(.CheckBox:hover) {
    transform: translateX(1px);
}

::v-deep(.CheckBox .el-checkbox__label) {
    font-size: 12px;
    width: auto;
    color: #1e40af;
    font-weight: 500;
    transition: color 0.3s ease;
}

::v-deep(.CheckBox .el-checkbox__input.is-checked + .el-checkbox__label) {
    color: #3b82f6;
    font-weight: 600;
}

::v-deep(.CheckBox .el-checkbox__input.is-checked .el-checkbox__inner) {
    background: linear-gradient(135deg, #3b82f6, #2563eb);
    border-color: #3b82f6;
}

::v-deep(.CheckBox .el-checkbox__inner) {
    border: 1px solid #93c5fd;
    border-radius: 3px;
    transition: all 0.3s ease;
    width: 12px;
    height: 12px;
}

::v-deep(.CheckBox .el-checkbox__inner:hover) {
    border-color: #3b82f6;
    transform: scale(1.05);
}

::v-deep(.CheckBox .el-checkbox__inner::after) {
    width: 3px;
    height: 6px;
    left: 3px;
    top: 1px;
}

/* 滑块标记字体缩小 */
::v-deep(.rating-slider .el-slider__marks-text) {
    font-size: 10px !important;
    color: #2563eb;
    font-weight: 500;
}

/* 美化酒店卡片 */
.HotelInfoCard {
    margin-bottom: 12px;
    width: 100%;
    height: 100px;
    position: relative;
    border-radius: 16px;
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.98), rgba(240, 248, 255, 0.98));
    border: 2px solid rgba(147, 197, 253, 0.3);
    box-shadow: 0 6px 20px rgba(59, 130, 246, 0.12);
    transition: all 0.3s cubic-bezier(0.23, 1, 0.32, 1);
    overflow: hidden;
    cursor: pointer;
}

.HotelInfoCard::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: linear-gradient(135deg, rgba(59, 130, 246, 0.05), rgba(37, 99, 235, 0.05));
    opacity: 0;
    transition: opacity 0.3s ease;
}

.HotelInfoCard:hover {
    transform: translateY(-3px) scale(1.01);
    box-shadow: 0 12px 35px rgba(59, 130, 246, 0.25);
    border-color: #2563eb;
}

.HotelInfoCard:hover::before {
    opacity: 1;
}

.HotelImageContainer {
    position: absolute;
    top: 10px;
    left: 10px;
    display: flex;
    justify-content: center;
    align-items: center;
    width: 80px;
    height: 80px;
    border-radius: 12px;
    overflow: hidden;
    cursor: pointer;
    border: 2px solid rgba(59, 130, 246, 0.2);
    transition: all 0.3s ease;
}

.HotelInfoCard:hover .HotelImageContainer {
    border-color: #3b82f6;
    box-shadow: 0 4px 15px rgba(59, 130, 246, 0.3);
}

.HotelImage { 
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.3s cubic-bezier(0.23, 1, 0.32, 1);
    filter: contrast(1.1) saturate(1.1);
}

.HotelImage:hover {
    transform: scale(1.1) rotate(1deg);
}

.HotelInfoShow {
    position: absolute;
    top: 10px;
    left: 100px;
    z-index: 1;
    max-width: calc(100% - 200px);
}

.HotelName {
    font-size: 1rem;
    font-weight: 700;
    margin-bottom: 3px;
    color: #1e40af;
    transition: color 0.3s ease;
    line-height: 1.2;
}

.HotelInfoCard:hover .HotelName {
    color: #2563eb;
}

.HotelGeneralInfo {
    font-size: 10px;
    margin-top: 0;
    margin-bottom: 3px;
    width: 100%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: #1e40af;
    font-weight: 500;
    line-height: 1.2;
}

.HotelRoomType {
    font-size: 11px;
    margin-top: 0;
    margin-bottom: 4px;
    width: 100%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: #3b82f6;
    font-weight: 600;
    padding: 2px 4px;
    background: rgba(59, 130, 246, 0.1);
    border-radius: 4px;
    display: inline-block;
}

.HotelRateContainer {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: -18px;
}

::v-deep(.HotelRate) {
    --el-rate-font-size: 10px;
    --el-rate-icon-size: 12px;
    --el-rate-void-color: #dbeafe;
    --el-rate-fill-color: #3b82f6;
}

.RatingNum {
    font-size: 10px;
    color: #2563eb;
    margin-bottom: 0;
    font-weight: 500;
    background: rgba(37, 99, 235, 0.1);
    padding: 1px 4px;
    border-radius: 6px;
}

.RightInfoShow {
    position: absolute;
    top: 10px;
    right: 10px;
    text-align: right;
    z-index: 1;
}

.HotelMoney {
    display: flex;
    align-items: baseline;
    justify-content: end;
    gap: 2px;
    margin-bottom: 4px;
}

.HotelMoney .p1 {
    font-size: 12px;
    color: #2563eb;
    margin-bottom: 0;
    font-weight: 600;
}

.HotelMoney .p2 {
    font-size: 1.3rem;
    font-weight: 800;
    color: #2563eb;
    margin-bottom: 0;
    text-shadow: 0 1px 3px rgba(37, 99, 235, 0.2);
}

.LiveNum {
    font-size: 10px;
    color: #3b82f6;
    margin-bottom: 6px;
    margin-top: 2px;
    font-weight: 500;
    background: rgba(59, 130, 246, 0.1);
    padding: 1px 4px;
    border-radius: 4px;
    display: inline-block;
}

.DetailButton {
    font-size: 12px;
    font-weight: 600;
    background: linear-gradient(135deg, #3b82f6, #2563eb);
    border: none;
    border-radius: 8px;
    color: white;
    padding: 4px 12px;
    box-shadow: 0 4px 15px rgba(59, 130, 246, 0.3);
    transition: all 0.3s cubic-bezier(0.23, 1, 0.32, 1);
}

.DetailButton:hover {
    background: linear-gradient(135deg, #2563eb, #3b82f6);
    transform: translateY(-1px);
    box-shadow: 0 6px 20px rgba(59, 130, 246, 0.4);
}

.HotelUnFind {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    height: 100%;
    color: #2563eb;
}

.UnfindImage {
    width: 200px;
    height: auto;
    margin-bottom: 20px;
    opacity: 0.8;
}

/* 美化浮动元素 */
.fixed-order-card {
    position: fixed;
    bottom: 80px;
    right: 30px;
    z-index: 1000;
    filter: drop-shadow(0 8px 25px rgba(59, 130, 246, 0.3));
}

.fixed-icon {
    position: fixed;
    bottom: 30px;
    right: 30px;
    z-index: 1000;
    cursor: pointer;
}

.FixedButton {
    width: 50px;
    height: 50px;
    background: linear-gradient(135deg, #3b82f6, #2563eb);
    border: none;
    box-shadow: 0 6px 20px rgba(59, 130, 246, 0.4);
    transition: all 0.3s cubic-bezier(0.23, 1, 0.32, 1);
    position: relative;
    overflow: hidden;
}

.FixedButton:hover {
    background: linear-gradient(135deg, #2563eb, #3b82f6);
    transform: translateY(-2px) scale(1.05);
    box-shadow: 0 10px 30px rgba(59, 130, 246, 0.5);
}
</style>
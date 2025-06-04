<template>
    <div class="Container">
        <el-card class="HotelInfoCard" v-if="hotelDetailInfo">
            <el-carousel class="HotelImageCarousel">
                <el-carousel-item class="HotelImageContainer" v-for="(img, index) in hotelDetailInfo.picture" :key="index">
                    <img class="HotelImage" :src="img" alt="hotel image">
                </el-carousel-item>
            </el-carousel>
            <div class="HotelInfo1">
                <p class="HotelName">{{ hotelDetailInfo.name }}</p>
                <p class="HotelInfoinfo">{{ hotelDetailInfo.info }}</p>
                <div class="HotelAddressContainer">
                    <el-icon><Location /></el-icon>
                    <p class="HotelAddress">{{ hotelDetailInfo.address }}</p>
                </div>
                <div class="HotelPhoneContainer">
                    <el-icon style="margin-top: 7px;"><Phone /></el-icon>
                    <div>
                        <p class="HotelPhone" v-for="(phone, index) in hotelDetailInfo.phone.slice(0, 2)" :key="index">{{ phone }}</p>
                    </div>
                </div>
            </div>
            <div class="HotelInfo2">
                <el-rate class="HotelRate" v-model="hotelDetailInfo.rating" disabled show-score 
                text-color="#ff9900" size="large" score-template="{value}"/>
                <p class="TotalBookings">{{ hotelDetailInfo.totalBookings }}人住过</p>
                <p class="RatingNum">{{ hotelDetailInfo.ratingCount }}人评价</p>
                <div class="HotelMoney">
                    <p class="p1">SC</p>
                    <p class="p2">{{ minPrice }}</p>
                    <p class="p1">起</p>
                </div>
                <el-button class="SelectRoomButton" type="primary" size="large" @click="goToRoom">选择房间</el-button>
            </div>
        </el-card>
        <div class="Grid">
            <el-tabs class="demo-tabs" v-model="tabActiveName">
                <el-tab-pane label="房间" name="room">
                    <div class="SelectDateContainer">
                        <div class="SelectDate">
                            <div class="SelectDateText">
                                <p>入住</p>
                                <p>--{{ dateRangeNum }}晚--</p>
                                <p>退房</p>
                            </div>
                            <div>
                                <a-range-picker suffix-icon="" id="DatePicker" class="DatePicker" v-model:value="selectedDateRange"
                                size="large" :locale="locale" :format="dateFormat" :bordered="false" :allow-clear="false"
                                :disabled-date="disabledDate"/>
                            </div>
                        </div>
                        <div class="HotelSearchButton">
                            <a-button type="primary" size="large" @click="getHotelOrderInfo">
                                <template #icon>
                                    <SearchOutlined />
                                </template>
                                切换
                            </a-button>
                        </div>
                    </div>
                    <el-table class="RoomTable" :data="hotelRoomInfoList" border>
                        <el-table-column prop="roomType" label="房型" width="150px" />
                        <el-table-column label="可住人数" width="150px">
                            <template #default="scope">
                                <div style="display: flex; justify-content: center;">
                                    <div v-for="(index) in scope.row.capacity" :key="index">
                                        <el-icon><Avatar /></el-icon>
                                    </div>
                                </div>
                            </template>
                        </el-table-column>
                        <el-table-column prop="remainCount" label="剩余房间数" width="100px" />
                        <el-table-column prop="price" label="今日价格" sortable width="150px" />
                        <el-table-column label="操作" width="150px">
                            <template #default="scope">
                                <el-button class="OrderButton" type="primary" 
                                :disabled="scope.row.remainCount == 0" @click="orderRoom(scope.row)">
                                    订
                                </el-button>
                            </template>
                        </el-table-column>
                    </el-table>
                </el-tab-pane>
                <el-tab-pane label="评论" name="comment">
                    <div class="CommentContainer">
                        <el-scrollbar height="280px">
                            <el-card class="CommentCard" v-for="(comment, index) in hotelDetailInfo?.comments" :key="index">
                                <el-avatar shape="square" :size="50" style="position: absolute; top: 20px; left: 20px;" 
                                src="https://cube.elemecdn.com/9/c2/f0ee8a3c7c9638a54940382568c9dpng.png" />
                                <div class="Info1">
                                    <div style="display: flex; justify-content: flex-start; align-items: baseline; gap: 10px">
                                        <p style="font-size: 18px; margin-bottom: 0;">{{ comment.username }}</p>
                                        <p style="font-size: 14px; margin-bottom: 0;">{{ comment.commentTime }}</p>
                                    </div>
                                    <p style="max-width: 500px; word-wrap: break-word; margin-bottom: 0;">{{ comment.comment }}</p>
                                </div>
                                <div class="Info2">
                                    <el-rate v-model="comment.rating" disabled show-score 
                                    text-color="#ff9900" size="large" score-template="{value}"/>
                                </div>
                            </el-card>
                            <p class="bottom-text">----已经到底了----</p>
                        </el-scrollbar>
                    </div>
                    <el-card class="CommentSendCard">
                        <textarea v-model="commentTextarea" class="CommentInput" placeholder="快评论一下吧！"></textarea>
                        <div class="CommentRateContainer">
                            <el-rate v-model="commentRate" allow-half style="margin-left: 3px;"/>
                            <el-input-number v-model="commentRate" :precision="1" :step="0.1" :max="5" :min="0" style="width: 120px; margin-top: 5px;" />
                        </div>
                        <el-button class="CommentSendButton" type="success"
                        @click="sendComment">发送</el-button>
                    </el-card>
                </el-tab-pane>
            </el-tabs>
            <HotelOrderCard />
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onBeforeMount } from 'vue'
import { useRoute } from 'vue-router';
import { hotelApi } from "@/api/HotelApi/hotelApi";
import type { HotelOrderQuery, HotelRoomDetailInfo, HotelDetailInfo, HotelComment, HotelRoomInfo } from '@/interface/hotelInterface';
import { ElMessage } from 'element-plus';
import { SearchOutlined } from '@ant-design/icons-vue';

const route = useRoute();
const hotelId = route.params.id as string;
const beginDate = ref();
const endDate = ref();
function initBeginDateFromRoute() {
    const value = route.query.beginDate;
    if (typeof value === 'string') {
        beginDate.value = value;
    }
}
function initEndDateFromRoute() {
    const value = route.query.endDate;
    if (typeof value === 'string') {
        endDate.value = value;
    }
}

const hotelDetailInfo = ref<HotelDetailInfo>();

const hotelRoomInfoList = ref<HotelRoomInfo[]>([]);
const minPrice = computed(() => {
    let min = -1;
    hotelRoomInfoList.value.forEach((key) => {
        if(min == -1 || min > key.price) {
            min = key.price;
        }
    })
    return min;
});

onBeforeMount(async () => {
    initBeginDateFromRoute();
    initEndDateFromRoute();
    initSelectedDateRange();
    getHotelDetailInfo();
    getHotelOrderInfo();
    //getdebugInfo();
})

async function getHotelDetailInfo() {
    hotelApi.hotelInfo(hotelId)
    .then((res) => {
        if(res.status == 200) {
            if(res.data.code == 200) {
                hotelDetailInfo.value = res.data.data;
                sortCommentByDate();
            } else {
                throw new Error(res.data.message);
            }
        }
    }).catch((error) => {
        ElMessage.error(error);
        console.error(error);
    })
} 

async function getHotelOrderInfo() {
    let tepBeginDate = dayjs(selectedDateRange.value[0]).format('YYYY-MM-DD');
    let tepEndDate = dayjs(selectedDateRange.value[1]).format('YYYY-MM-DD');
    if (dateRangeNum.value > 7) {
        ElMessage.error('入住时间不能超过7晚');
        return;
    }
    const hotelOrderQuery: HotelOrderQuery = {
        hotelId: hotelId,
        beginDate: tepBeginDate,
        endDate: tepEndDate,
    }
    hotelApi.hotelOrderInfo(hotelOrderQuery)
    .then((res) => {
        if(res.status == 200) {
            if(res.data.code == 200) {
                hotelRoomInfoList.value = [];
                beginDate.value = tepBeginDate;
                endDate.value = tepEndDate;
                let myMap = new Map(Object.entries(res.data.data as { [key: string]: HotelRoomDetailInfo }));
                myMap.forEach((value, key) => {
                    let tepHotelRoomInfo: HotelRoomInfo = {
                        ...value,
                        roomType: key,
                    }
                    hotelRoomInfoList.value.push(tepHotelRoomInfo);
                })
                hotelRoomInfoList.value.sort((a, b) => {
                    if(a.remainCount == 0) {
                        return 1;
                    } else if(b.remainCount ==0) {
                        return -1;
                    } else {
                        return 0;
                    }
                });
            }  else {
                throw new Error(res.data.message);
            }
        }
    }) .catch((error) => {
        ElMessage.error(error);
        console.error(error);
    })
}

//对评论按照时间降序排列
function sortCommentByDate() {
    return hotelDetailInfo.value?.comments.sort((a, b) => {
        return b.commentTime.localeCompare(a.commentTime);
    })
}

//-----------------------------room---------------------------------------
const tabActiveName = ref<string>('room');
function goToRoom() {
    tabActiveName.value = 'room';
}

//------------------------------datepicker--------------------------------
import locale from 'ant-design-vue/es/date-picker/locale/zh_CN';
import dayjs from 'dayjs';
import 'dayjs/locale/zh-cn';

dayjs.locale('zh-cn');

const dateFormat = 'YYYY-MM-DD(dddd)';
const selectedDateRange = ref();
function initSelectedDateRange() {
    selectedDateRange.value = [dayjs(beginDate.value), dayjs(endDate.value)];
}
const dateRangeNum = computed(() => {
    let startDate = dayjs(selectedDateRange.value[0]);
    let endDate = dayjs(selectedDateRange.value[1])
    return endDate.diff(startDate, 'day');
})

function disabledDate(current: any) {
    return current && current < dayjs().startOf('day');
}

//-----------------------------订房间--------------------------------------
import HotelOrderCard from '@/components/HotelSearch/HotelOrderCard.vue';

import { useHotelOrderStore } from '@/stores/hotelOrder';
const hotelOrderStore = useHotelOrderStore();

const orderRoom = (room: HotelRoomInfo) => {
    if(!hotelOrderStore.add(room, hotelDetailInfo.value)){
        ElMessage.error('该酒店房型已在预订列表中');
    } else {
        ElMessage.success('加入预订列表成功，可在预定列表中修改数量');
    }
}

//-----------------------------返回上一个页面------------------------------

//-----------------------------评价---------------------------------------
const commentRate = ref<number>(0);
const commentTextarea = ref<string>('');
const hotelCommentQuota = ref<HotelCommentQuota | undefined>();

import type { HotelCommentQuota, NewHotelComment } from '@/interface/hotelInterface';

async function sendComment() {
    if(!checkComment()) {
        return;
    }
    if(hotelCommentQuota.value == undefined) {
        await hotelApi.hotelQuota(hotelId)
        .then((res) => {
            if(res.status == 200) {
                if(res.data.code == 200) {
                    hotelCommentQuota.value =  res.data.data as HotelCommentQuota;
                } else {
                    throw new Error(res.data.message);
                }
            }
        }) .catch((err) => {
            ElMessage.error(err);
        })
    }
    if(hotelCommentQuota.value == undefined) {
        return;
    }
    if(hotelCommentQuota.value.quota - hotelCommentQuota.value.used > 0) {
        realySendComment();
    } else {
        ElMessage.error('您暂无评价次数，请先预订酒店吧！')
    }
}

async function realySendComment() {
    let newComment: NewHotelComment = {
        hotelId: hotelId,
        rating: commentRate.value,
        comment: commentTextarea.value,
    }
    await hotelApi.hotelComment(newComment)
    .then((res)=> {
        if(res.status == 200){
            if(res.data.code == 200) {
                getHotelDetailInfo(); //重新获取，以更新评论
            } else {
                throw new Error(res.data.message);
            }
        } 
    }).catch((err) => {
        ElMessage.error(err);
    })
}

function checkComment() {
    if(commentTextarea.value.trim() == '') {
        ElMessage.error('请输入您的评论');
        return false;
    } else if (commentRate.value == 0) {
        ElMessage.error('请输入您的评分');
        return false;
    }
    return true;
}

//-----------------------------debugInfo-----------------------------------
import debugHotelImage1 from '../../assets/hotel.jpg';
import debugHotelImage2 from '../../assets/hotel2.jpg';
import { fa } from 'element-plus/es/locales.mjs';

const debugComments = [
    {
        username: '张三',
        commentTime: "2025-04-08 12:22:28",
        rating: 4.5,
        comment: "房间干净整洁，下次还会来。"
    }as HotelComment,
    {
        username: '李四',
        commentTime: "2024-09-24 10:22:28",
        rating: 4.8,
        comment: "前台小姐姐很热情。"
    }as HotelComment,
    {
        username: '王五',
        commentTime: "2025-02-05 10:00:28",
        rating: 4.5,
        comment: "房间干净整洁，下次还会来。房间干净整洁，下次还会来。房间干净整洁，下次还会来。房间干净整洁，下次还会来。房间干净整洁，下次还会来。房间干净整洁，下次还会来。房间干净整洁，下次还会来。房间干净整洁，下次还会来。"
    } as HotelComment,
] as HotelComment [];
const debugHotelDetailInfo: HotelDetailInfo = {
    hotelId: hotelId,
    name: '桔子水晶酒店',
    address: '北京朝阳街155号',
    phone: [
        "047-31633709",
        "024-62762345"
    ],
    info: "本酒店距离火车站步行约5分钟，配备免费Wi-Fi与早餐。",
    picture: [
        debugHotelImage1,
        debugHotelImage2
    ],
    rating: 4.6,
    ratingCount: 326,
    totalBookings: 1247,
    comments: debugComments,
}
const debugHotelRoomDetailInfo1: HotelRoomDetailInfo = {
    capacity: 1,
    remainCount: 5,
    price: 200,
}
const debugHotelRoomDetailInfo2: HotelRoomDetailInfo = {
    capacity: 2,
    remainCount: 3,
    price: 796,
}
const debugHotelRoomDetailInfo3: HotelRoomDetailInfo = {
    capacity: 2,
    remainCount: 0,
    price: 599,
}
const debugHotelRoomDetailInfo4: HotelRoomDetailInfo = {
    capacity: 3,
    remainCount: 1,
    price: 999,
}
const debugHotelRoomDetailInfo5: HotelRoomDetailInfo = {
    capacity: 3,
    remainCount: 0,
    price: 4000,
}
function getdebugInfo() {
    hotelDetailInfo.value = debugHotelDetailInfo,
    hotelRoomInfoList.value.push({
        ...debugHotelRoomDetailInfo1,
        roomType: '标间',
    })
    hotelRoomInfoList.value.push({
        ...debugHotelRoomDetailInfo2,
        roomType: '大床房',
    })
    hotelRoomInfoList.value.push({
        ...debugHotelRoomDetailInfo3,
        roomType: '双床房',
    })
    hotelRoomInfoList.value.push({
        ...debugHotelRoomDetailInfo4,
        roomType: '三人间',
    })
    hotelRoomInfoList.value.push({
        ...debugHotelRoomDetailInfo5,
        roomType: '行政套房',
    })
    hotelRoomInfoList.value.sort((a, b) => {
        if(a.remainCount == 0) {
            return 1;
        } else if(b.remainCount ==0) {
            return -1;
        } else {
            return 0;
        }
    });
    sortCommentByDate();
}
</script>

<style scoped>
.Container {
    display: flex;
    justify-content: center;
    flex-wrap: wrap;
}
.HotelInfoCard {
    width: 1035px;
    height: 250px;
    border-radius: 8px; /* 圆角大小 */
    margin-top: 70px;
    position: relative; /* 用于支持绝对定位的子元素 */
    overflow: hidden;
}
.HotelImageCarousel{
    position: absolute;
    left: 0;
    top: 0;
    width: 250px;
    height: 250px;
}
.HotelImageContainer{
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    overflow: hidden;
}
.HotelImage { 
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.HotelInfo1 {
    position: absolute;
    top: 15px;
    left: 280px;
}
.HotelName {
    font-size: 2rem;
    font-weight: bold;
    margin-bottom: 0;
}
.HotelInfoinfo {
    font-size: 18px;
    margin-bottom: 5px;
    margin-top: 10px;
}
.HotelAddressContainer {
    display: flex;
    justify-content: flex-start;
    align-items: center;
    gap: 10px;
    padding-top: 10px;
}
::v-deep(.HotelAddressContainer .el-icon) {
    height: 1.25rem;
    width: 1.25rem;
}
::v-deep(.HotelAddressContainer .el-icon svg) {
    height: 1.25rem;
    width: 1.25rem;
}
.HotelAddress {
    font-size: 1.25rem;
    margin-bottom: 5px;
    margin-top: 5px;
}
.HotelPhoneContainer {
    display: flex;
    justify-content: flex-start;
    gap: 10px;
}
::v-deep(.HotelPhoneContainer .el-icon) {
    height: 1.25rem;
    width: 1.25rem;
}
::v-deep(.HotelPhoneContainer .el-icon svg) {
    height: 1.25rem;
    width: 1.25rem;
}
.HotelPhone {
    font-size: 1.25rem;
    margin-bottom: 0;
}

.HotelInfo2 {
    position: absolute;
    top: 20px;
    right: 30px;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
}
::v-deep(.HotelRate) {
    --el-rate-font-size: 1.75rem;
    --el-rate-icon-size: 1.75rem;
}
.RatingNum {
    font-size: 16px;
    color: rgb(189,190,194);
    margin-bottom: 0;
}
.TotalBookings {
    font-size: 16px;
    color: rgb(189,190,194);
    margin-bottom: 0;
}
.HotelMoney {
    display: flex;
    align-items: baseline;
    justify-content: end;
    gap: 3px;
    padding-top: 25px;
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
.SelectRoomButton {
    font-size: 20px;
    font-weight: bold;
}


.Grid {
    display: flex;
    justify-content: space-between;
    gap: 35px;
    margin-top: 20px;
}
.demo-tabs {
    width: 700px;
}
::v-deep(.demo-tabs .el-tabs__item) {
    font-size: 18px;
}

.SelectDateContainer {
    display: flex;
    justify-content: center;
}
.SelectDate {
    display: block;
    border-radius: 8px;
    padding-left: 15px;
    padding-right: 15px;
    padding-top: 10px;
    padding-bottom: 10px;
    height: 80px;
    width: 430px;
    border: 1px solid rgb(189,190,194);
}
.SelectDate p {
    margin-left: 11px;
    margin-right: 11px;
    margin-bottom: 0;
    font-size: 14px;
    color: rgb(189,190,194);
}
.SelectDateText {
    display: flex;
    justify-content: space-between;
}
.DatePicker {
    width: 400px;
}
::v-deep(#DatePicker) {
    font-size: 16px;
    font-weight: bolder;
    text-align: left;
}
::v-deep(.ant-picker-range-separator) {
  display: none;
}
::v-deep(.DatePicker .ant-picker-input > input) {
  font-size: 16px;
  font-weight: bolder;
  text-align: right;
}

.HotelSearchButton {
    margin-right: 3%;
    margin-left: 3%;
    height: 80px;
    width: 90px;
}
.HotelSearchButton .ant-btn {
    height: 100%;
    width: 100%;
}

::v-deep(.el-table .el-table__cell) {
    text-align: center;
}
.RoomTable {
    margin-top: 20px;
}

.OrderInfo {
    position: relative;
    width: 300px;
    height: 450px;
}
.OrderInfo .title{
    font-size: 18px;
    font-weight: bold;
    margin-bottom: 10px;
}
.OrderOkButton {
    position: absolute;
    right: 20px;
    bottom: 20px;
}
::v-deep(.OrderOkButton span) {
    font-weight: bold;
}
::v-deep(.OrderButton span) {
    font-weight: bold;
}

.OrderInfoCardContainer {
    position: relative;
}
.OrderInfoCard {
    width: 240px;
    margin-bottom: 10px;
}
.DeleteIcon {
  position: absolute;
  top: 0;
  right: 0;
}
.DeleteIcon:hover {
  cursor: pointer; 
}
::v-deep(.DeleteIcon) {
    --color: rgb(160, 160, 160);
}

.fixed-icon {
    position: fixed;
    top: 100px; /* 距离窗口底部的距离 */
    left: 220px; /* 距离窗口右侧的距离 */
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



.CommentContainer {
    width: 700px;
    height: 290px;
    padding-right: 20px;
    padding-left: 20px;
    padding-bottom: 10px;
}
.CommentCard {
    width: 640px;
    margin-bottom: 10px;
    position: relative;
}
.CommentCard .Info1 {
    margin-left: 70px;
}
.CommentCard .Info2 {
    position: absolute;
    top: 15px;
    right: 25px;
}
.bottom-text {
    margin-right: 20px;
    color: rgb(160, 160, 160);
    text-align: center;
    margin-bottom: 0;
}

.CommentSendCard {
    width: 700px;
    height: 95px;
    position: relative;
    margin-top: 10px;
}
.CommentInput {
    position: absolute; 
    left: 10px; 
    top: 10px; 
    width: 450px; 
    height: 75px; 
    border: none; 
    resize: none; 
    outline: none;
}
.CommentRateContainer {
    position: absolute; 
    top: 15px; 
    left: 485px; 
    display: flex; 
    flex-direction: column;
}
.CommentSendButton {
    position: absolute; 
    right: 10px; 
    bottom: 10px;
}
</style>
<template>
    <div class="container">
        <!-- 酒店信息主卡片 -->
        <div class="hotel-info-section" v-if="hotelDetailInfo">
            <div class="hotel-info-card">
                <!-- 酒店图片轮播 -->
                <div class="hotel-gallery">
                    <el-carousel class="hotel-carousel" height="320px" :interval="4000" arrow="hover">
                        <el-carousel-item 
                            v-for="(img, index) in hotelDetailInfo.picture" 
                            :key="index"
                            class="carousel-item"
                        >
                            <div class="image-container">
                                <img class="hotel-image" :src="img" alt="酒店图片">
                                <div class="image-overlay">
                                    <div class="image-indicator">
                                        <span class="current">{{ index + 1 }}</span>
                                        <span class="divider">/</span>
                                        <span class="total">{{ hotelDetailInfo.picture?.length || 0 }}</span>
                                    </div>
                                </div>
                            </div>
                        </el-carousel-item>
                    </el-carousel>
                </div>

                <!-- 酒店详情信息 -->
                <div class="hotel-details">
                    <div class="hotel-main-info">
                        <div class="title-section">
                            <h1 class="hotel-name">{{ hotelDetailInfo.name }}</h1>
                            <div class="rating-section">
                                <el-rate 
                                    class="hotel-rating" 
                                    v-model="hotelDetailInfo.rating" 
                                    disabled 
                                    show-score 
                                    text-color="#ff6b35"
                                    :max="5"
                                    size="large"
                                />
                                <div class="rating-stats">
                                    <span class="rating-count">{{ hotelDetailInfo.ratingCount }}条评价</span>
                                    <span class="booking-count">{{ hotelDetailInfo.totalBookings }}人住过</span>
                                </div>
                            </div>
                        </div>

                        <div class="hotel-description">
                            <p class="description-text">{{ hotelDetailInfo.info }}</p>
                        </div>

                        <div class="hotel-contact">
                            <div class="contact-item">
                                <div class="contact-icon">
                                    <el-icon><Location /></el-icon>
                                </div>
                                <div class="contact-content">
                                    <span class="contact-label">地址</span>
                                    <span class="contact-value">{{ hotelDetailInfo.address }}</span>
                                </div>
                            </div>
                            <div class="contact-item">
                                <div class="contact-icon">
                                    <el-icon><Phone /></el-icon>
                                </div>
                                <div class="contact-content">
                                    <span class="contact-label">电话</span>
                                    <div class="phone-list">
                                        <span 
                                            v-for="(phone, index) in hotelDetailInfo.phone.slice(0, 2)" 
                                            :key="index"
                                            class="phone-number"
                                        >
                                            {{ phone }}
                                        </span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="hotel-price-action">
                        <div class="price-section">
                            <div class="price-label">最低价格</div>
                            <div class="price-display">
                                <span class="currency">SC</span>
                                <span class="price-amount">{{ minPrice }}</span>
                                <span class="price-suffix">起</span>
                            </div>
                        </div>
                        <el-button 
                            class="select-room-btn" 
                            type="primary" 
                            size="large" 
                            @click="goToRoom"
                        >
                            <template #icon>
                                <el-icon><House /></el-icon>
                            </template>
                            选择房间
                        </el-button>
                    </div>
                </div>
            </div>
        </div>

        <!-- 详细信息区域 -->
        <div class="details-section">
            <div class="details-container">
                <!-- 标签页内容 -->
                <div class="tabs-container">
                    <el-tabs v-model="tabActiveName" class="modern-tabs">
                        <!-- 房间标签页 -->
                        <el-tab-pane label="房间预订" name="room">
                            <div class="room-booking-section">
                                <!-- 日期选择器 -->
                                <div class="date-selector-card">
                                    <div class="date-selector-header">
                                        <h3 class="selector-title">选择入住日期</h3>
                                        <p class="selector-subtitle">选择您的入住和退房时间</p>
                                    </div>
                                    <div class="date-selector-content">
                                        <div class="date-picker-container">
                                            <div class="date-display">
                                                <div class="date-labels">
                                                    <span class="date-label">入住</span>
                                                    <span class="nights-count">{{ dateRangeNum }}晚</span>
                                                    <span class="date-label">退房</span>
                                                </div>
                                                <a-range-picker 
                                                    v-model:value="selectedDateRange"
                                                    class="modern-date-picker"
                                                    size="large" 
                                                    :locale="locale" 
                                                    :format="dateFormat" 
                                                    :bordered="false" 
                                                    :allow-clear="false"
                                                    :disabled-date="disabledDate"
                                                    @change="handleDateChange"
                                                />
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                <!-- 房间列表 -->
                                <div class="rooms-list-card" v-if="hotelRoomInfoList &&hotelRoomInfoList.length">
                                    <div class="rooms-header">
                                        <h3 class="rooms-title">可预订房型</h3>
                                        <p class="rooms-subtitle">选择适合您的房型</p>
                                    </div>
                                    <div class="rooms-table-container">
                                        <el-table 
                                            :data="hotelRoomInfoList" 
                                            class="modern-table"
                                            :header-cell-style="{ 
                                                background: '#f8fafc', 
                                                color: '#374151',
                                                fontWeight: '600',
                                                fontSize: '14px',
                                                textAlign: 'center'
                                            }"
                                            :cell-style="{ textAlign: 'center' }"
                                            style="width: 100%"
                                        >
                                            <el-table-column prop="roomType" label="房型" min-width="160">
                                                <template #default="scope">
                                                    <div class="room-type-cell">
                                                        <div class="room-icon">🏠</div>
                                                        <span class="room-name">{{ scope.row.roomType }}</span>
                                                    </div>
                                                </template>
                                            </el-table-column>
                                            
                                            <el-table-column label="可住人数" min-width="120">
                                                <template #default="scope">
                                                    <div class="capacity-cell">
                                                        <div class="capacity-icons">
                                                            <el-icon 
                                                                v-for="index in scope.row.capacity" 
                                                                :key="index"
                                                                class="person-icon"
                                                            >
                                                                <Avatar />
                                                            </el-icon>
                                                        </div>
                                                        <span class="capacity-text">{{ scope.row.capacity }}人</span>
                                                    </div>
                                                </template>
                                            </el-table-column>
                                            
                                            <el-table-column prop="remainCount" label="剩余房间" min-width="100">
                                                <template #default="scope">
                                                    <div class="remain-cell">
                                                        <div class="remain-badge" :class="getRemainClass(scope.row.remainCount)">
                                                            <span class="remain-number">{{ scope.row.remainCount }}</span>
                                                            <span class="remain-unit">间</span>
                                                        </div>
                                                        <div class="remain-status">
                                                            {{ getRemainStatus(scope.row.remainCount) }}
                                                        </div>
                                                    </div>
                                                </template>
                                            </el-table-column>
                                            
                                            <el-table-column prop="price" label="价格/晚" sortable min-width="120">
                                                <template #default="scope">
                                                    <div class="price-cell">
                                                        <div class="price-main">
                                                            <span class="price-currency">SC</span>
                                                            <span class="price-number">{{ scope.row.price }}</span>
                                                        </div>
                                                        <div class="price-total" v-if="dateRangeNum > 1">
                                                            <span class="total-text">{{ dateRangeNum }}晚共</span>
                                                            <span class="total-amount">SC{{ scope.row.price * dateRangeNum }}</span>
                                                        </div>
                                                    </div>
                                                </template>
                                            </el-table-column>
                                            
                                            <el-table-column label="操作" min-width="120">
                                                <template #default="scope">
                                                    <div class="action-cell">
                                                        <el-button 
                                                            class="book-btn" 
                                                            :class="{ 'sold-out': scope.row.remainCount === 0 }"
                                                            type="primary"
                                                            size="small"
                                                            :disabled="scope.row.remainCount === 0" 
                                                            @click="orderRoom(scope.row)"
                                                        >
                                                            <template #icon>
                                                                <el-icon>
                                                                    <ShoppingCart v-if="scope.row.remainCount > 0" />
                                                                    <Close v-else />
                                                                </el-icon>
                                                            </template>
                                                            <span class="btn-text">
                                                                {{ scope.row.remainCount === 0 ? '已售完' : '预订' }}
                                                            </span>
                                                        </el-button>
                                                    </div>
                                                </template>
                                            </el-table-column>
                                        </el-table>
                                    </div>
                                </div>
                            </div>
                        </el-tab-pane>

                        <!-- 评论标签页 -->
                        <el-tab-pane label="用户评价" name="comment">
                            <div class="comments-section">
                                <!-- 评论列表 -->
                                <div class="comments-list-card">
                                    <div class="comments-header">
                                        <h3 class="comments-title">用户评价</h3>
                                        <p class="comments-subtitle">来看看其他住客的真实体验</p>
                                    </div>
                                    <div class="comments-content">
                                        <el-scrollbar height="350px" class="comments-scrollbar">
                                            <div v-if="hotelDetailInfo?.comments.length === 0" class="no-comments">
                                                <div class="no-comments-icon">💬</div>
                                                <p class="no-comments-text">暂无评价，来写第一条评价吧！</p>
                                            </div>
                                            <div v-else class="comments-list">
                                                <div 
                                                    v-for="(comment, index) in hotelDetailInfo?.comments" 
                                                    :key="index"
                                                    class="comment-card"
                                                >
                                                    <div class="comment-header">
                                                        <div class="user-info">
                                                            <el-avatar 
                                                                class="user-avatar" 
                                                                :size="48"
                                                                src="https://cube.elemecdn.com/9/c2/f0ee8a3c7c9638a54940382568c9dpng.png" 
                                                            />
                                                            <div class="user-details">
                                                                <span class="username">{{ comment.username }}</span>
                                                                <span class="comment-time">{{ formatCommentTime(comment.commentTime) }}</span>
                                                            </div>
                                                        </div>
                                                        <div class="comment-rating">
                                                            <el-rate 
                                                                v-model="comment.rating" 
                                                                disabled 
                                                                show-score 
                                                                text-color="#ff6b35"
                                                                size="small"
                                                            />
                                                        </div>
                                                    </div>
                                                    <div class="comment-content">
                                                        <p class="comment-text">{{ comment.comment }}</p>
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="list-end">
                                                <span class="end-text">已显示全部评价</span>
                                            </div>
                                        </el-scrollbar>
                                    </div>
                                </div>

                                <!-- 发表评论 -->
                                <div class="comment-form-card">
                                    <div class="form-header">
                                        <h3 class="form-title">发表评价</h3>
                                        <p class="form-subtitle">分享您的住宿体验</p>
                                    </div>
                                    <div class="form-content">
                                        <div class="rating-input">
                                            <span class="rating-label">您的评分</span>
                                            <div class="rating-controls">
                                                <el-rate 
                                                    v-model="commentRate" 
                                                    allow-half 
                                                    class="comment-rate"
                                                    text-color="#ff6b35"
                                                />
                                                <el-input-number 
                                                    v-model="commentRate" 
                                                    :precision="1" 
                                                    :step="0.1" 
                                                    :max="5" 
                                                    :min="0" 
                                                    class="rating-number"
                                                />
                                            </div>
                                        </div>
                                        <div class="comment-input">
                                            <el-input
                                                v-model="commentTextarea"
                                                type="textarea"
                                                :rows="4"
                                                placeholder="请分享您的住宿体验，帮助其他旅客做出选择..."
                                                class="comment-textarea"
                                                maxlength="500"
                                                show-word-limit
                                                resize="none"
                                            />
                                        </div>
                                        <div class="form-actions">
                                            <el-button 
                                                class="submit-comment-btn" 
                                                type="primary"
                                                size="large"
                                                @click="sendComment"
                                            >
                                                <template #icon>
                                                    <el-icon><EditPen /></el-icon>
                                                </template>
                                                发表评价
                                            </el-button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </el-tab-pane>
                    </el-tabs>
                </div>

                <!-- 订单卡片 -->
                <div class="order-card-section">
                    <HotelOrderCard />
                </div>
            </div>
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
import { Location, Phone, House, Avatar, ShoppingCart, EditPen, Close } from '@element-plus/icons-vue';

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
})

async function getHotelDetailInfo() {
    hotelApi.hotelInfo(hotelId)
    .then((res: any) => {
        if(res.status == 200) {
            if(res.data.code == 200) {
                hotelDetailInfo.value = res.data.data;
                sortCommentByDate();
            } else {
                throw new Error(res.data.message);
            }
        }
    }).catch((error: any) => {
        ElMessage.error(error);
        console.error(error);
    })
} 

const capacityChangeTab = {
    标准间: 1,
    大床房: 2,
    行政套房: 3,
}

async function getHotelOrderInfo() {
    let tepBeginDate = dayjs(selectedDateRange.value[0]).format('YYYY-MM-DD');
    let tepEndDate = dayjs(selectedDateRange.value[1]).format('YYYY-MM-DD');
    if (dateRangeNum.value > 7) {
        ElMessage.error('入住时间不能超过7晚');
        hotelRoomInfoList.value = [];
        return;
    }
    const hotelOrderQuery: HotelOrderQuery = {
        hotelId: hotelId,
        beginDate: tepBeginDate,
        endDate: tepEndDate,
    }
    hotelApi.hotelOrderInfo(hotelOrderQuery)
    .then((res: any) => {
        if(res.status == 200) {
            if(res.data.code == 200) {
                hotelRoomInfoList.value = [];
                beginDate.value = tepBeginDate;
                endDate.value = tepEndDate;
                let myMap = new Map(Object.entries(res.data.data as { [key: string]: HotelRoomDetailInfo }));
                myMap.forEach((value, key) => {
                    let tepHotelRoomInfo: HotelRoomInfo = {
                        ...value,
                        roomType: key as "标准间" | "大床房" | "行政套房",
                    }
                    tepHotelRoomInfo.capacity = capacityChangeTab[tepHotelRoomInfo.roomType];
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
    }) .catch((error: any) => {
        ElMessage.error(error);
        console.error(error);
    })
}

async function handleDateChange() {
    await getHotelOrderInfo();
}

function sortCommentByDate() {
    return hotelDetailInfo.value?.comments.sort((a, b) => {
        return b.commentTime.localeCompare(a.commentTime);
    })
}

// 格式化评论时间
function formatCommentTime(timeStr: string): string {
    const date = new Date(timeStr);
    return date.toLocaleDateString('zh-CN', {
        year: 'numeric',
        month: 'long',
        day: 'numeric'
    });
}

const tabActiveName = ref<string>('room');
function goToRoom() {
    tabActiveName.value = 'room';
}

// 日期选择器相关
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

// 订房间相关
import HotelOrderCard from '@/components/HotelSearch/HotelOrderCard.vue';
import { useHotelOrderStore } from '@/stores/hotelOrder';

const hotelOrderStore = useHotelOrderStore();

const orderRoom = (room: HotelRoomInfo) => {
    if(!hotelOrderStore.add(room, hotelDetailInfo.value, beginDate.value, endDate.value)){
        ElMessage.error('该酒店房型已在预订列表中');
    } else {
        ElMessage.success('加入预订列表成功，可在预定列表中修改数量');
    }
}

// 评价相关
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
        .then((res: any) => {
            if(res.status == 200) {
                if(res.data.code == 200) {
                    hotelCommentQuota.value =  res.data.data as HotelCommentQuota;
                } else {
                    throw new Error(res.data.message);
                }
            }
        }) .catch((err: any) => {
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
    .then((res: any)=> {
        if(res.status == 200){
            if(res.data.code == 200) {
                getHotelDetailInfo();
                commentTextarea.value = '';
                commentRate.value = 0;
                ElMessage.success('评价发表成功！');
            } else {
                throw new Error(res.data.message);
            }
        } 
    }).catch((err: any) => {
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

// 添加这些辅助方法
function getRemainClass(count: number): string {
    if (count === 0) return 'sold-out';
    if (count <= 3) return 'low-stock';
    return 'in-stock';
}

function getRemainStatus(count: number): string {
    if (count === 0) return '已售完';
    if (count <= 3) return '仅剩几间';
    return '充足';
}
</script>

<style scoped>
/* 主容器 */
.container {
    min-height: 100vh;
    background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
    padding: 20px;
}

/* 酒店信息区域 */
.hotel-info-section {
    max-width: 1200px;
    margin: 0 auto 30px;
}

.hotel-info-card {
    background: rgba(255, 255, 255, 0.98);
    backdrop-filter: blur(20px);
    border-radius: 24px;
    box-shadow: 
        0 20px 40px rgba(0, 0, 0, 0.1),
        0 0 0 1px rgba(255, 255, 255, 0.2);
    overflow: hidden;
    display: grid;
    grid-template-columns: 400px 1fr;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.hotel-info-card:hover {
    transform: translateY(-4px);
    box-shadow: 
        0 32px 64px rgba(0, 0, 0, 0.15),
        0 0 0 1px rgba(255, 255, 255, 0.3);
}

/* 酒店图片轮播 */
.hotel-gallery {
    position: relative;
    height: 100%;
    min-height: 350px;
}

.hotel-carousel {
    height: 100%;
    border-radius: 24px 0 0 24px;
    overflow: hidden;
}

.carousel-item {
    height: 100%;
}

.image-container {
    position: relative;
    height: 100%;
    overflow: hidden;
}

.hotel-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.6s ease;
}

.carousel-item:hover .hotel-image {
    transform: scale(1.05);
}

.image-overlay {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    background: linear-gradient(transparent, rgba(0, 0, 0, 0.6));
    padding: 20px;
}

.image-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    font-size: 14px;
    font-weight: 500;
    gap: 4px;
}

.current {
    font-weight: 700;
}

.divider {
    opacity: 0.7;
}

/* 酒店详情 */
.hotel-details {
    padding: 40px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
}

.hotel-main-info {
    flex: 1;
}

.title-section {
    margin-bottom: 24px;
}

.hotel-name {
    font-size: 32px;
    font-weight: 700;
    color: #1a202c;
    margin: 0 0 16px 0;
    letter-spacing: -0.5px;
    line-height: 1.2;
}

.rating-section {
    display: flex;
    align-items: center;
    gap: 16px;
}

.hotel-rating :deep(.el-rate) {
    --el-rate-font-size: 20px;
    --el-rate-icon-size: 20px;
    --el-rate-fill-color: #ff6b35;
}

.rating-stats {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.rating-count,
.booking-count {
    font-size: 13px;
    color: #64748b;
    font-weight: 500;
}

.hotel-description {
    margin-bottom: 32px;
}

.description-text {
    font-size: 16px;
    line-height: 1.6;
    color: #374151;
    margin: 0;
}

.hotel-contact {
    display: flex;
    flex-direction: column;
    gap: 16px;
}

.contact-item {
    display: flex;
    align-items: center;
    gap: 12px;
}

.contact-icon {
    width: 40px;
    height: 40px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    font-size: 16px;
}

.contact-content {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.contact-label {
    font-size: 12px;
    color: #64748b;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.contact-value {
    font-size: 15px;
    color: #374151;
    font-weight: 500;
}

.phone-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.phone-number {
    font-size: 15px;
    color: #374151;
    font-weight: 500;
}

/* 价格和操作区域 */
.hotel-price-action {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    padding-top: 24px;
    border-top: 1px solid #f1f5f9;
}

.price-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.price-label {
    font-size: 14px;
    color: #64748b;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.price-display {
    display: flex;
    align-items: baseline;
    gap: 4px;
}

.currency {
    font-size: 20px;
    color: #667eea;
    font-weight: 600;
}

.price-amount {
    font-size: 36px;
    font-weight: 800;
    color: #667eea;
    line-height: 1;
}

.price-suffix {
    font-size: 16px;
    color: #64748b;
    font-weight: 500;
}

.select-room-btn {
    padding: 16px 32px;
    border-radius: 16px;
    font-size: 16px;
    font-weight: 600;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    border: none;
    box-shadow: 0 8px 24px rgba(102, 126, 234, 0.3);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    letter-spacing: 0.5px;
}

.select-room-btn:hover {
    transform: translateY(-2px);
    box-shadow: 0 12px 32px rgba(102, 126, 234, 0.4);
    background: linear-gradient(135deg, #764ba2 0%, #667eea 100%);
}

/* 详细信息区域 */
.details-section {
    max-width: 1200px;
    margin: 0 auto;
}

.details-container {
    display: grid;
    grid-template-columns: 1fr 420px;
    gap: 30px;
}

/* 标签页容器 */
.tabs-container {
    background: rgba(255, 255, 255, 0.98);
    backdrop-filter: blur(20px);
    border-radius: 24px;
    box-shadow: 
        0 20px 40px rgba(0, 0, 0, 0.1),
        0 0 0 1px rgba(255, 255, 255, 0.2);
    overflow: hidden;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.tabs-container:hover {
    transform: translateY(-2px);
    box-shadow: 
        0 24px 48px rgba(0, 0, 0, 0.12),
        0 0 0 1px rgba(255, 255, 255, 0.3);
}

.modern-tabs {
    padding: 0;
}

.modern-tabs :deep(.el-tabs__header) {
    margin: 0;
    background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
    padding: 0 32px;
}

.modern-tabs :deep(.el-tabs__nav-wrap) {
    padding: 16px 0;
}

.modern-tabs :deep(.el-tabs__item) {
    font-size: 16px;
    font-weight: 600;
    color: #64748b;
    padding: 12px 24px;
    border-radius: 12px;
    transition: all 0.2s;
    margin-right: 8px;
}

.modern-tabs :deep(.el-tabs__item:hover) {
    color: #667eea;
}

.modern-tabs :deep(.el-tabs__item.is-active) {
    color: #667eea;
}

.modern-tabs :deep(.el-tabs__active-bar) {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    height: 3px;
    border-radius: 2px;
}

.modern-tabs :deep(.el-tabs__content) {
    padding: 32px;
}

/* 房间预订区域 */
.room-booking-section {
    display: flex;
    flex-direction: column;
    gap: 24px;
}

/* 日期选择器卡片 */
.date-selector-card {
    background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
    border-radius: 16px;
    padding: 24px;
    border: 1px solid #e2e8f0;
}

.date-selector-header {
    margin-bottom: 20px;
}

.selector-title {
    font-size: 18px;
    font-weight: 700;
    color: #1a202c;
    margin: 0 0 4px 0;
}

.selector-subtitle {
    font-size: 14px;
    color: #64748b;
    margin: 0;
}

.date-picker-container {
    display: flex;
    align-items: center;
    gap: 20px;
}

.date-display {
    flex: 1;
}

.date-labels {
    display: flex;
    justify-content: space-between;
    margin-bottom: 8px;
}

.date-label {
    font-size: 12px;
    color: #64748b;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.nights-count {
    font-size: 12px;
    color: #667eea;
    font-weight: 700;
    background: rgba(102, 126, 234, 0.1);
    padding: 2px 8px;
    border-radius: 8px;
}

.modern-date-picker {
    width: 100%;
}

.modern-date-picker :deep(.ant-picker) {
    border: 2px solid #e2e8f0;
    border-radius: 12px;
    padding: 12px 16px;
    font-size: 15px;
    font-weight: 600;
    transition: all 0.2s;
}

.modern-date-picker :deep(.ant-picker:hover) {
    border-color: #667eea;
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

.modern-date-picker :deep(.ant-picker-input > input) {
    font-size: 15px;
    font-weight: 600;
    color: #374151;
}

.search-rooms-btn {
    padding: 12px 24px;
    border-radius: 12px;
    font-weight: 600;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    border: none;
    box-shadow: 0 4px 16px rgba(102, 126, 234, 0.3);
    transition: all 0.2s;
    white-space: nowrap;
}

.search-rooms-btn:hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4);
}

/* 房间列表卡片 */
.rooms-list-card {
    background: #fff;
    border-radius: 16px;
    border: 1px solid #f1f5f9;
    overflow: hidden;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
}

.rooms-header {
    padding: 24px 24px 16px;
    background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
    border-bottom: 1px solid #e2e8f0;
}

.rooms-title {
    font-size: 18px;
    font-weight: 700;
    color: #1a202c;
    margin: 0 0 4px 0;
}

.rooms-subtitle {
    font-size: 14px;
    color: #64748b;
    margin: 0;
}

.rooms-table-container {
    padding: 0;
    overflow: hidden; /* 防止出现滚动条 */
}

.modern-table {
    border: none;
    width: 100% !important;
    table-layout: fixed; /* 固定表格布局 */
}

.modern-table :deep(.el-table__header) {
    background: #f8fafc;
}

.modern-table :deep(.el-table__header-wrapper) {
    overflow: hidden; /* 防止表头出现滚动条 */
}

.modern-table :deep(.el-table__body-wrapper) {
    overflow: hidden; /* 防止表体出现滚动条 */
}

.modern-table :deep(.el-table__row) {
    transition: all 0.2s ease;
    background: transparent;
}

.modern-table :deep(.el-table__row:hover) {
    background: linear-gradient(135deg, rgba(102, 126, 234, 0.04) 0%, rgba(118, 75, 162, 0.04) 100%);
    transform: translateY(-1px);
}

.modern-table :deep(.el-table td) {
    border: none;
    padding: 16px 8px;
    vertical-align: middle;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.modern-table :deep(.el-table th) {
    border: none;
    padding: 16px 8px;
    background: #f8fafc;
    font-weight: 600;
    color: #374151;
    vertical-align: middle;
}

.modern-table :deep(.el-table th.is-leaf) {
    border-bottom: 2px solid rgba(241, 245, 249, 0.8);
}

.modern-table :deep(.el-table__fixed),
.modern-table :deep(.el-table__fixed-right) {
    display: none; /* 隐藏固定列 */
}

/* 表格单元格样式 */
.room-type-cell {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 0 4px;
}

.room-icon {
    font-size: 16px;
    flex-shrink: 0;
}

.room-name {
    font-weight: 600;
    color: #374151;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.capacity-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 0 4px;
}

.capacity-icons {
    display: flex;
    gap: 3px;
    align-items: center;
    justify-content: center;
}

.person-icon {
    color: #667eea;
    font-size: 16px;
}

.capacity-text {
    font-size: 11px;
    color: #64748b;
    font-weight: 600;
    white-space: nowrap;
}

.remain-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 0 4px;
}

.remain-badge {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 2px;
    padding: 4px 8px;
    border-radius: 6px;
    font-weight: 700;
    font-size: 12px;
    border: 1px solid;
    transition: all 0.3s ease;
    min-width: 50px;
}

.remain-badge.in-stock {
    background: linear-gradient(135deg, rgba(16, 185, 129, 0.1) 0%, rgba(5, 150, 105, 0.1) 100%);
    color: #10b981;
    border-color: rgba(16, 185, 129, 0.3);
}

.remain-badge.low-stock {
    background: linear-gradient(135deg, rgba(245, 158, 11, 0.1) 0%, rgba(217, 119, 6, 0.1) 100%);
    color: #f59e0b;
    border-color: rgba(245, 158, 11, 0.3);
}

.remain-badge.sold-out {
    background: linear-gradient(135deg, rgba(239, 68, 68, 0.1) 0%, rgba(220, 38, 38, 0.1) 100%);
    color: #ef4444;
    border-color: rgba(239, 68, 68, 0.3);
}

.remain-number {
    font-size: 13px;
    line-height: 1;
}

.remain-unit {
    font-size: 10px;
}

.remain-status {
    font-size: 10px;
    color: #94a3b8;
    font-weight: 500;
    white-space: nowrap;
}

.price-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 0 4px;
}

.price-main {
    display: flex;
    align-items: baseline;
    justify-content: center;
    gap: 2px;
}

.price-currency {
    font-size: 11px;
    color: #667eea;
    font-weight: 600;
}

.price-number {
    font-size: 16px;
    font-weight: 700;
    color: #667eea;
    line-height: 1;
}

.price-total {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1px;
    padding: 2px 6px;
    background: rgba(102, 126, 234, 0.1);
    border-radius: 4px;
    border: 1px solid rgba(102, 126, 234, 0.2);
}

.total-text {
    font-size: 9px;
    color: #64748b;
    font-weight: 500;
    white-space: nowrap;
}

.total-amount {
    font-size: 10px;
    color: #667eea;
    font-weight: 700;
    white-space: nowrap;
}

.action-cell {
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 0 4px;
}

.book-btn {
    padding: 8px 12px;
    border-radius: 8px;
    font-weight: 600;
    font-size: 12px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    border: none;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 0 2px 8px rgba(102, 126, 234, 0.3);
    min-width: 80px;
    position: relative;
    overflow: hidden;
}

.book-btn::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.3), transparent);
    transition: left 0.5s;
}

.book-btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.book-btn:hover:not(:disabled)::before {
    left: 100%;
}

.book-btn:disabled,
.book-btn.sold-out {
    background: linear-gradient(135deg, #e5e7eb 0%, #d1d5db 100%);
    color: #9ca3af;
    box-shadow: none;
    cursor: not-allowed;
    transform: none;
}

.btn-text {
    position: relative;
    z-index: 1;
    white-space: nowrap;
}

/* 确保表格不出现滚动条 */
.modern-table :deep(.el-table__inner-wrapper) {
    overflow: hidden !important;
}

.modern-table :deep(.el-scrollbar) {
    overflow: hidden !important;
}

.modern-table :deep(.el-scrollbar__wrap) {
    overflow: hidden !important;
}

/* 响应式调整 */
@media (max-width: 768px) {
    .modern-table :deep(.el-table td),
    .modern-table :deep(.el-table th) {
        padding: 12px 4px;
    }
    
    .room-type-cell {
        flex-direction: column;
        gap: 4px;
    }
    
    .capacity-cell,
    .remain-cell,
    .price-cell {
        gap: 2px;
    }
    
    .book-btn {
        padding: 6px 8px;
        font-size: 11px;
        min-width: 60px;
    }
    
    .price-number {
        font-size: 14px;
    }
}

/* 表格行动画 */
@keyframes slideInUp {
    from {
        opacity: 0;
        transform: translateY(20px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.modern-table :deep(.el-table__row) {
    animation: slideInUp 0.4s cubic-bezier(0.4, 0, 0.2, 1) forwards;
}

.modern-table :deep(.el-table__row:nth-child(1)) { animation-delay: 0.1s; }
.modern-table :deep(.el-table__row:nth-child(2)) { animation-delay: 0.2s; }
.modern-table :deep(.el-table__row:nth-child(3)) { animation-delay: 0.3s; }
.modern-table :deep(.el-table__row:nth-child(4)) { animation-delay: 0.4s; }

/* 滚动条美化 */
.comments-scrollbar :deep(.el-scrollbar__thumb) {
    background: linear-gradient(180deg, #667eea, #764ba2);
    border-radius: 6px;
}

.comments-scrollbar :deep(.el-scrollbar__thumb:hover) {
    background: linear-gradient(180deg, #764ba2, #667eea);
}

.comments-scrollbar :deep(.el-scrollbar__bar) {
    right: 4px;
    width: 8px;
    border-radius: 4px;
}
</style>
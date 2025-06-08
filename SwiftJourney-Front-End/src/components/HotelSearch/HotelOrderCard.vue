<template>
    <div class="order-card-container">
        <div class="order-card">
            <!-- 头部标题区域 -->
            <div class="card-header">
                <div class="header-content">
                    <h2 class="page-title">已选房间</h2>
                    <p class="page-subtitle">
                        <span v-if="hotelOrderStore.hotelOrderInfoList.length > 0">
                            共 {{ hotelOrderStore.hotelOrderInfoList.length }} 种房型
                        </span>
                        <span v-else>暂无选择</span>
                    </p>
                </div>
                <div class="total-price" v-if="hotelOrderStore.hotelOrderInfoList.length > 0">
                    <span class="price-label">总价</span>
                    <span class="price-value">SC {{ totalMoney }}</span>
                </div>
            </div>

            <!-- 分割线 -->
            <div class="divider"></div>

            <!-- 订单内容 -->
            <div class="order-content">
                <el-scrollbar ref="scrollbar" height="320px" class="order-scrollbar">
                    <div ref="innerRef">
                    <!-- 空状态 -->
                    <div v-if="hotelOrderStore.hotelOrderInfoList.length === 0" class="empty-state">
                        <div class="empty-icon">
                            <el-icon size="48"><House /></el-icon>
                        </div>
                        <h3 class="empty-title">还没有选择房间</h3>
                        <p class="empty-subtitle">在酒店列表中选择心仪的房型</p>
                    </div>

                    <!-- 订单列表 -->
                    <div v-else class="order-list">
                        <div 
                            v-for="(item, index) in hotelOrderStore.hotelOrderInfoList" 
                            :key="index"
                            class="order-item"
                            @mouseenter="showDeleteIcon(index)" 
                            @mouseleave="hideDeleteIcon(index)"
                        >
                            <div class="order-card-inner">
                                <div class="hotel-info">
                                    <h4 class="hotel-name">{{ item.name }}</h4>
                                    <div class="room-type">{{ item.roomType }}</div>
                                </div>

                                <div class="date-info">
                                    <div class="date-item">
                                        <span class="date-label">入住</span>
                                        <span class="date-value">{{ formatDate(item.beginDate) }}</span>
                                    </div>
                                    <div class="date-separator">→</div>
                                    <div class="date-item">
                                        <span class="date-label">退房</span>
                                        <span class="date-value">{{ formatDate(item.endDate) }}</span>
                                    </div>
                                </div>

                                <div class="quantity-section">
                                    <span class="quantity-label">数量</span>
                                    <el-input-number 
                                        v-model="item.amount" 
                                        :min="1" 
                                        :max="item.maxCount"
                                        size="small"
                                        class="quantity-input"
                                    />
                                </div>

                                <div class="price-section">
                                    <div class="unit-price">单价 SC {{ item.price }}</div>
                                    <div class="total-price-item">
                                        <span class="total-label">小计</span>
                                        <span class="total-value">SC {{ item.amount * item.price }}</span>
                                    </div>
                                </div>

                                <!-- 删除按钮 -->
                                <div 
                                    v-if="deleteIconsVisible[index]" 
                                    class="delete-btn"
                                    @click="deleteRoomFromOrder(item.hotelId, item.name, item.roomType, item.beginDate, item.endDate)"
                                    @mouseenter="showDeleteIcon(index)" 
                                    @mouseleave="hideDeleteIcon(index)"
                                >
                                    <el-icon><Close /></el-icon>
                                </div>
                            </div>
                        </div>
                    </div>
                    </div>
                </el-scrollbar>
            </div>

            <!-- 底部操作区域 -->
            <div class="card-footer">
                <el-button 
                    class="order-button" 
                    type="primary"
                    size="large"
                    :disabled="hotelOrderStore.hotelOrderInfoList.length === 0"
                    @click="createTransaction"
                >
                    <template #icon>
                        <el-icon><ShoppingCart /></el-icon>
                    </template>
                    生成订单
                </el-button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onBeforeMount, onMounted, onBeforeUnmount } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus';
import type { ScrollbarInstance } from 'element-plus'
import { useHotelOrderStore } from '@/stores/hotelOrder';
import { useRouter } from 'vue-router';

const hotelOrderStore = useHotelOrderStore();

import { watch, nextTick } from 'vue'
// 滚动条组件的引用
const innerRef = ref<HTMLDivElement>()
const scrollbar = ref<ScrollbarInstance>()
// 获取 hotelOrderInfoList
const hotelOrderInfoListLength = computed(() => hotelOrderStore.hotelOrderInfoList.length);
// 监听 hotelOrderInfoList 的长度变化
watch(hotelOrderInfoListLength, (newLength: number, oldLength: number) => {
    if (newLength > oldLength) {
        nextTick(() => { 
            scrollbar.value!.scrollTo({ top: innerRef.value!.clientHeight - 320, behavior: 'smooth' });
        });
    }
});

onBeforeMount(() => {
    hotelOrderStore.loadFromLocalStorage();
})

// 定义一个函数来处理页面可见性变化的事件
const handleVisibilityChange = () => {
    if (document.visibilityState === 'visible') {
        hotelOrderStore.loadFromLocalStorage();
    }
};

// 在组件挂载时添加事件监听器
onMounted(() => {
    document.addEventListener('visibilitychange', handleVisibilityChange);
});

// 在组件卸载时移除事件监听器
onBeforeUnmount(() => {
    document.removeEventListener('visibilitychange', handleVisibilityChange);
});

const totalMoney = computed(() => {
    let sum = 0;
    hotelOrderStore.hotelOrderInfoList.forEach((key) => {
        sum += key.amount * key.price;
    })
    return sum;
})

const deleteIconsVisible = ref(hotelOrderStore.hotelOrderInfoList.map(() => false));

function showDeleteIcon(index: number) {
    deleteIconsVisible.value[index] = true;
}

function hideDeleteIcon(index: number) {
    deleteIconsVisible.value[index] = false;
}

function deleteRoomFromOrder(hotelId: string, hotelName: string, roomType: string, beginDate: string, endDate: string) {
    ElMessageBox.confirm(
        `确定要取消选择"${hotelName}"的"${roomType}"吗？`,
        '取消确认',
        {
            confirmButtonText: '确定取消',
            cancelButtonText: '保留',
            type: 'warning'
        }
    )
    .then(() => {
        hotelOrderStore.delete(hotelId, roomType, beginDate, endDate);
        ElMessage.success(`已取消选择"${hotelName}"的"${roomType}"`);
    })
}

// 格式化日期
function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return `${date.getMonth() + 1}/${date.getDate()}`;
}

//---------------------------------生成订单-----------------------------------
import type { HotelOrderRequest } from '@/interface/hotelInterface';
import type { TransactionInfo } from '@/interface/interface';
import { hotelApi } from '@/api/HotelApi/hotelApi';
import { useUserStore } from '@/stores/user';

const nowUser = useUserStore();

function createTransaction() {
    ElMessageBox.confirm(
        `您选择的房型总价为 SC${totalMoney.value}，核对无误后请点击确定`,
        '确认生成订单',
        {
            confirmButtonText: '确定生成',
            cancelButtonText: '再看看',
            type: 'info'
        }
    )
    .then(() => {
        confirmCreateTransaction();
    })
}

async function confirmCreateTransaction() {
    let hotelOrderRequestList: HotelOrderRequest[] = [];
    hotelOrderStore.hotelOrderInfoList.forEach((key: any) => {
        let hotelOrderRequest: HotelOrderRequest = {
            hotelId: key.hotelId,
            roomType: key.roomType,
            beginDate: key.beginDate,
            endDate: key.endDate,
            personalId: nowUser.personalId,
            amount: key.amount,
        };
        hotelOrderRequestList.push(hotelOrderRequest);
    })
    await hotelApi.hotelOrder(hotelOrderRequestList)
    .then((res: any) => {
        if(res.status == 200) {
            if(res.data.code == 200) {
                successCreateTransaction(res.data.data as TransactionInfo);
            } else {
                throw new Error(res.data.message);
            }
        }
    }) .catch ((error: any) => {
        ElMessage.error('生成订单失败 ' + error);
    })
}

function successCreateTransaction(transactionInfo: TransactionInfo) {
    hotelOrderStore.deleteAll();
    ElMessageBox.confirm(
        `您的订单号为 ${transactionInfo.transactionId}，总价 SC${transactionInfo.amount}，可在订单系统中查看具体信息，是否立即支付？`,
        '订单生成成功',
        {
            confirmButtonText: '立即支付',
            cancelButtonText: '稍后支付',
            type: 'success',
        }
    ) .then(() =>{
        //处理支付逻辑
        goToPay(transactionInfo.transactionId, 'SC ' + transactionInfo.amount);
    })
}

const router = useRouter();

function goToPay(transactionId: string, money: string) {
    router.push({
        name: 'paypage',
        params: { transactionId: transactionId },
        query: {
            money: money,
        }
    });
}
</script>

<style scoped>
/* 容器样式 */
.order-card-container {
    position: relative;
    padding: 8px;
}

/* 主卡片 */
.order-card {
    width: 380px;
    background: rgba(255, 255, 255, 0.98);
    backdrop-filter: blur(20px);
    border-radius: 20px;
    box-shadow: 
        0 20px 40px rgba(0, 0, 0, 0.12),
        0 0 0 1px rgba(255, 255, 255, 0.2);
    overflow: hidden;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    border: 1px solid rgba(59, 130, 246, 0.1);
}

.order-card:hover {
    transform: translateY(-2px);
    box-shadow: 
        0 32px 64px rgba(0, 0, 0, 0.16),
        0 0 0 1px rgba(255, 255, 255, 0.3);
}

/* 卡片头部 */
.card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 24px 24px 16px;
    background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
}

.header-content {
    flex: 1;
}

.page-title {
    font-size: 20px;
    font-weight: 700;
    color: #1a202c;
    margin: 0 0 4px 0;
    letter-spacing: -0.3px;
}

.page-subtitle {
    font-size: 13px;
    color: #64748b;
    margin: 0;
    font-weight: 500;
}

.total-price {
    text-align: right;
    margin-left: 16px;
}

.price-label {
    display: block;
    font-size: 12px;
    color: #64748b;
    margin-bottom: 2px;
    font-weight: 500;
}

.price-value {
    display: block;
    font-size: 18px;
    font-weight: 700;
    color: #1e40af;
    text-shadow: 0 1px 2px rgba(30, 64, 175, 0.1);
}

/* 分割线 */
.divider {
    height: 1px;
    background: linear-gradient(90deg, transparent, #e2e8f0, transparent);
    margin: 0 24px;
}

/* 订单内容 */
.order-content {
    padding: 20px 24px 16px;
}

.order-scrollbar :deep(.el-scrollbar__view) {
    padding-right: 8px;
}

/* 空状态 */
.empty-state {
    text-align: center;
    padding: 40px 20px;
}

.empty-icon {
    color: #cbd5e0;
    margin-bottom: 16px;
}

.empty-title {
    font-size: 16px;
    font-weight: 600;
    color: #4a5568;
    margin: 0 0 8px 0;
}

.empty-subtitle {
    font-size: 13px;
    color: #718096;
    margin: 0;
}

/* 订单列表 */
.order-list {
    display: flex;
    flex-direction: column;
    gap: 16px;
}

/* 订单项 */
.order-item {
    position: relative;
    transition: all 0.2s ease;
}

.order-card-inner {
    background: #fff;
    border: 2px solid #f1f5f9;
    border-radius: 16px;
    padding: 20px;
    position: relative;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    overflow: hidden;
}

.order-card-inner::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: linear-gradient(135deg, #3b82f6 0%, #1e40af 100%);
    transform: scaleX(0);
    transform-origin: left;
    transition: transform 0.3s;
}

.order-item:hover .order-card-inner {
    border-color: #e2e8f0;
    transform: translateY(-2px);
    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.08);
}

.order-item:hover .order-card-inner::before {
    transform: scaleX(1);
}

/* 酒店信息 */
.hotel-info {
    margin-bottom: 16px;
}

.hotel-name {
    font-size: 16px;
    font-weight: 700;
    color: #1a202c;
    margin: 0 0 8px 0;
    line-height: 1.3;
}

.room-type {
    display: inline-block;
    padding: 4px 12px;
    background: linear-gradient(135deg, rgba(59, 130, 246, 0.1) 0%, rgba(30, 64, 175, 0.1) 100%);
    color: #1e40af;
    font-size: 12px;
    font-weight: 600;
    border-radius: 8px;
    border: 1px solid rgba(59, 130, 246, 0.2);
}

/* 日期信息 */
.date-info {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
    padding: 12px;
    background: #f8fafc;
    border-radius: 10px;
    border: 1px solid #e2e8f0;
}

.date-item {
    flex: 1;
    text-align: center;
}

.date-label {
    display: block;
    font-size: 11px;
    color: #64748b;
    margin-bottom: 4px;
    font-weight: 500;
}

.date-value {
    display: block;
    font-size: 14px;
    color: #1e40af;
    font-weight: 600;
}

.date-separator {
    margin: 0 12px;
    color: #94a3b8;
    font-size: 14px;
    font-weight: 600;
}

/* 数量选择 */
.quantity-section {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
}

.quantity-label {
    font-size: 14px;
    color: #374151;
    font-weight: 600;
}

.quantity-input {
    width: 100px;
}

.quantity-input :deep(.el-input-number__decrease),
.quantity-input :deep(.el-input-number__increase) {
    background: #f8fafc;
    border-color: #e2e8f0;
    color: #64748b;
}

.quantity-input :deep(.el-input-number__decrease):hover,
.quantity-input :deep(.el-input-number__increase):hover {
    background: #3b82f6;
    border-color: #3b82f6;
    color: white;
}

.quantity-input :deep(.el-input__wrapper) {
    border-color: #e2e8f0;
}

.quantity-input :deep(.el-input__wrapper):hover {
    border-color: #3b82f6;
}

/* 价格区域 */
.price-section {
    border-top: 1px solid #f1f5f9;
    padding-top: 12px;
}

.unit-price {
    font-size: 12px;
    color: #64748b;
    margin-bottom: 8px;
    font-weight: 500;
}

.total-price-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.total-label {
    font-size: 14px;
    color: #374151;
    font-weight: 600;
}

.total-value {
    font-size: 16px;
    color: #1e40af;
    font-weight: 700;
    text-shadow: 0 1px 2px rgba(30, 64, 175, 0.1);
}

/* 删除按钮 */
.delete-btn {
    position: absolute;
    top: 12px;
    right: 12px;
    width: 28px;
    height: 28px;
    background: rgba(239, 68, 68, 0.1);
    color: #dc2626;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
    backdrop-filter: blur(10px);
    border: 1px solid rgba(239, 68, 68, 0.2);
}

.delete-btn:hover {
    background: #dc2626;
    color: white;
    transform: scale(1.1);
    box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
}

.delete-btn .el-icon {
    font-size: 14px;
}

/* 底部操作区域 */
.card-footer {
    padding: 20px 24px 24px;
    border-top: 1px solid #f1f5f9;
    background: linear-gradient(135deg, #fafbfc 0%, #f1f5f9 100%);
}

.order-button {
    width: 100%;
    height: 48px;
    border-radius: 12px;
    font-size: 16px;
    font-weight: 600;
    background: linear-gradient(135deg, #3b82f6 0%, #1e40af 100%);
    border: none;
    box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    letter-spacing: 0.5px;
}

.order-button:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 8px 25px rgba(59, 130, 246, 0.4);
    background: linear-gradient(135deg, #1e40af 0%, #3b82f6 100%);
}

.order-button:disabled {
    background: #e2e8f0;
    color: #94a3b8;
    box-shadow: none;
    cursor: not-allowed;
}

.order-button .el-icon {
    font-size: 18px;
    margin-right: 8px;
}

/* 响应式设计 */
@media (max-width: 480px) {
    .order-card {
        width: 100%;
        max-width: 350px;
    }

    .card-header {
        flex-direction: column;
        align-items: flex-start;
        gap: 12px;
    }

    .total-price {
        margin-left: 0;
        text-align: left;
    }

    .date-info {
        flex-direction: column;
        gap: 8px;
    }

    .date-separator {
        transform: rotate(90deg);
        margin: 4px 0;
    }
}

.order-item {
    animation: slideInUp 0.4s cubic-bezier(0.4, 0, 0.2, 1) forwards;
}

.order-item:nth-child(1) { animation-delay: 0.1s; }
.order-item:nth-child(2) { animation-delay: 0.2s; }
.order-item:nth-child(3) { animation-delay: 0.3s; }
.order-item:nth-child(4) { animation-delay: 0.4s; }

/* 滚动条美化 */
.order-scrollbar :deep(.el-scrollbar__thumb) {
    background: rgba(59, 130, 246, 0.3);
    border-radius: 4px;
}

.order-scrollbar :deep(.el-scrollbar__thumb:hover) {
    background: rgba(59, 130, 246, 0.5);
}

.order-scrollbar :deep(.el-scrollbar__bar) {
    right: 2px;
    width: 6px;
}

/* 交互反馈 */
.order-card-inner {
    cursor: default;
}

.quantity-input :deep(.el-input__inner) {
    text-align: center;
    font-weight: 600;
    color: #1e40af;
}

/* 加载状态 */
.order-card.loading {
    pointer-events: none;
    opacity: 0.7;
}

.order-card.loading::after {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(255, 255, 255, 0.8);
    backdrop-filter: blur(2px);
    z-index: 10;
}

/* 成功状态 */
.order-card.success .order-card-inner::before {
    background: linear-gradient(135deg, #10b981 0%, #059669 100%);
    transform: scaleX(1);
}

/* 微交互 */
.order-button:active {
    transform: translateY(0) scale(0.98);
}

.delete-btn:active {
    transform: scale(0.95);
}
</style>
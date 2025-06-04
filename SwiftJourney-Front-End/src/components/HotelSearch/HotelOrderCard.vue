<template>
    <el-card class="OrderInfo">
        <p class="title">已选房间</p>
        <el-scrollbar height="320px" style="display: flex; justify-content: center;">
            <div v-for="(key, index) in hotelOrderStore.hotelOrderInfoList" :key="index">
                <div class="OrderInfoCardContainer">
                    <el-descriptions class="OrderInfoCard" border :column="1" size="default"
                    @mouseenter="showDeleteIcon(index)" @mouseleave="hideDeleteIcon(index)">
                        <el-descriptions-item label="酒店名称">{{ key.name }}</el-descriptions-item>
                        <el-descriptions-item label="房型">{{ key.roomType }}</el-descriptions-item>
                        <el-descriptions-item label="数量">
                            <el-input-number v-model="key.amount" min="1" :max="key.maxCount" style="width: 100px;"/>
                        </el-descriptions-item>
                        <el-descriptions-item label="总价">{{ key.amount * key.price }}</el-descriptions-item>
                    </el-descriptions>
                    <el-icon v-if="deleteIconsVisible[index]" class="DeleteIcon"
                    @mouseenter="showDeleteIcon(index)" @mouseleave="hideDeleteIcon(index)"
                    @click="deleteRoomFromOrder(key.hotelId, key.name, key.roomType)">
                    <CircleCloseFilled /></el-icon>
                </div>
            </div>
            <div v-if="hotelOrderStore.hotelOrderInfoList.length == 0">
                {{ '您还没有选择任何酒店房型' }}
            </div>
        </el-scrollbar>
        <el-button class="OrderOkButton" type="success" :disabled="hotelOrderStore.hotelOrderInfoList.length == 0"
        @click="createTransaction">生成订单</el-button>
    </el-card>
</template>

<script setup lang="ts">
import { ref, computed, onBeforeMount, onMounted, onBeforeUnmount } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus';
import { useHotelOrderStore } from '@/stores/hotelOrder';
import { useRouter } from 'vue-router';
const hotelOrderStore = useHotelOrderStore();

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

function deleteRoomFromOrder(hotelId: string,hotelName: string, roomType: string) {
    ElMessageBox.confirm(
        '是否取消选择' + hotelName + '的' + roomType,
        '警告',
        {
            confirmButtonText: '确定',
            cancelButtonText: '取消',
            type: 'warning'
        }
    )
    .then(() => {
        hotelOrderStore.delete(hotelId, roomType);
        ElMessage.success('成功取消选择' + hotelName + '的' + roomType);
    })
}

//---------------------------------生成订单-----------------------------------
import type { HotelOrderRequest } from '@/interface/hotelInterface';
import type { TransactionInfo } from '@/interface/interface';
import { hotelApi } from '@/api/HotelApi/hotelApi';

function createTransaction() {
    ElMessageBox.confirm(
        '您选择的房型总价为 SC' + totalMoney.value + '，核对无误后请点击确定',
        '确认生成订单',
        {
            confirmButtonText: '确定',
            cancelButtonText: '取消',
            type: 'warning'
        }
    )
    .then(() => {
        confirmCreateTransaction();
    })
}

async function confirmCreateTransaction() {
    let hotelOrderRequestList: HotelOrderRequest[] = [];
    hotelOrderStore.hotelOrderInfoList.forEach((key) => {
        let hotelOrderRequest: HotelOrderRequest = {
            hotelId: key.hotelId,
            roomType: key.roomType,
            beginDate: key.beginDate,
            endDate: key.endDate,
            personalId: key.personalId,
            amount: key.amount,
        };
        hotelOrderRequestList.push(hotelOrderRequest);
    })
    await hotelApi.hotelOrder(hotelOrderRequestList)
    .then((res) => {
        if(res.status == 200) {
            if(res.data.code == 200) {
                successCreateTransaction(res.data as TransactionInfo);
            } else {
                throw new Error(res.data.message);
            }
        }
    }) .catch ((error) => {
        ElMessage.error('生成订单失败 ' + error);
    })
}

function successCreateTransaction(transactionInfo: TransactionInfo) {
    hotelOrderStore.deleteAll();
    ElMessageBox.confirm(
        '您的订单号为 ' + transactionInfo.transactionId + ' ,总价 SC' + transactionInfo.amount + '，可在订单系统中查看具体信息，是否立即支付',
        '生成订单成功',
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

function goToPay(transactionaId: string, money: string) {
    const routeUrl = router.resolve({
        name: 'paypage',
        params: { transactionId: transactionaId },
        query: {
          money: money,
        }
      });
    window.open(routeUrl.href, '_blank');
}

</script>

<style scoped>
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
</style>
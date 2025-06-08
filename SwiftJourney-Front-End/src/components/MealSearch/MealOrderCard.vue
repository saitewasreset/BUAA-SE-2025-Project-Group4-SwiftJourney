<template>
    <el-card class="OrderInfo">
        <p class="title">已选餐品</p>
        <el-scrollbar height="320px" style="display: flex; justify-content: center;">
            <div v-for="(key, index) in mealOrderStore.mealOrderInfoList" :key="index">
                <div class="OrderInfoCardContainer">
                    <el-descriptions class="OrderInfoCard" border :column="1" size="default"
                    @mouseenter="showDeleteIcon(index)" @mouseleave="hideDeleteIcon(index)">
                        <el-descriptions-item label="店铺名称">{{ key.shopName }}</el-descriptions-item>
                        <el-descriptions-item label="餐品名称">{{ key.name }}</el-descriptions-item>
                        <el-descriptions-item v-if="key.dishTime" label="就餐时间">{{ lunchChangeTab[key.dishTime] }}</el-descriptions-item>
                        <el-descriptions-item label="数量">
                            <el-input-number v-model="key.amount" min="1" style="width: 100px;"/>
                        </el-descriptions-item>
                        <el-descriptions-item label="总价">{{ key.amount * key.price }}</el-descriptions-item>
                    </el-descriptions>
                    <el-icon v-if="deleteIconsVisible[index]" class="DeleteIcon"
                    @mouseenter="showDeleteIcon(index)" @mouseleave="hideDeleteIcon(index)"
                    @click="deleteRoomFromOrder(key.shopName, key.name, key.dishTime)">
                    <CircleCloseFilled /></el-icon>
                </div>
            </div>
            <div v-if="mealOrderStore.mealOrderInfoList.length == 0">
                {{ '您还没有选择任何餐品' }}
            </div>
        </el-scrollbar>
        <el-button class="OrderOkButton" type="success" :disabled="mealOrderStore.mealOrderInfoList.length == 0"
        @click="createTransaction">生成订单</el-button>
    </el-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus';
import { useMealOrderStore } from '@/stores/mealOrder';
import { useRouter } from 'vue-router';
const mealOrderStore = useMealOrderStore();

const lunchChangeTab = {
    lunch: '午餐',
    dinner: '晚餐',
}

const totalMoney = computed(() => {
    let sum = 0;
    mealOrderStore.mealOrderInfoList.forEach((key) => {
        sum += key.amount * key.price;
    })
    return sum;
})

const deleteIconsVisible = ref(mealOrderStore.mealOrderInfoList.map(() => false));

function showDeleteIcon(index: number) {
    deleteIconsVisible.value[index] = true;
}

function hideDeleteIcon(index: number) {
    deleteIconsVisible.value[index] = false;

}

function deleteRoomFromOrder(shopName: string, foodName: string, time?: 'lunch' | 'dinner') {
    ElMessageBox.confirm(
        '是否取消选择' + shopName + '的' + foodName,
        '警告',
        {
            confirmButtonText: '确定',
            cancelButtonText: '取消',
            type: 'warning'
        }
    )
    .then(() => {
        mealOrderStore.delete(shopName, foodName, time);
        ElMessage.success('成功取消选择' + shopName + '的' + foodName);
    })
}

//---------------------------------生成订单-----------------------------------
import { useUserStore } from '@/stores/user';

const nowUser = useUserStore();

function createTransaction() {
    ElMessageBox.confirm(
        '您选择的餐品总价为 SC' + totalMoney.value + '，核对无误后请点击确定',
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

import type { TrainDishOrderRequest, TakeawayOrder, DishOrder } from '@/interface/mealInterface';
import { mealApi } from '@/api/MealApi/mealApi';
import type { TransactionInfo } from '@/interface/interface';

async function confirmCreateTransaction() {
    const trainDishOrderRequest: TrainDishOrderRequest = {
        trainNumber: mealOrderStore.trainNumber,
        originDepartureTime: mealOrderStore.originDepartureTime,
        takeaway: [],
        dishes: [],
    };

    mealOrderStore.mealOrderInfoList.forEach((value) => {
        if(value.shopName == '餐车') {
            const tepInfo: DishOrder = {
                name: value.name,
                personalId: value.personalId,
                amount: value.amount,
                dishTime: value.dishTime as 'lunch' | 'dinner',
            }
            trainDishOrderRequest.dishes.push(tepInfo);
        } else {
            const tepInfo: TakeawayOrder = {
                station: value.station as string,
                shopName: value.shopName,
                name: value.name,
                personalId: value.personalId,
                amount: value.amount,
            }
            trainDishOrderRequest.takeaway.push(tepInfo);
        }
    });

    await mealApi.dishOrder(trainDishOrderRequest)
    .then((res) => {
        if(res.status == 200) {
            if(res.data.code == 200) {
                successCreateTransaction(res.data.data as TransactionInfo);
            } else if (res.data.code == 22006) {
                ElMessage.error('没有对应的车次订单/对应的车次订单未支付/对应的车次订单已完成（失败/已取消）');
            }
            else {
                throw new Error(res.data.message);
            }
        }
    }) .catch ((error) => {
        ElMessage.error('生成订单失败 ' + error);
    })
}

function successCreateTransaction(transactionInfo: TransactionInfo) {
    mealOrderStore.deleteAll();
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
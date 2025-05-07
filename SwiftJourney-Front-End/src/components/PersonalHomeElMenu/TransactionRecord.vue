<template>
    <div>
        <div class="container">
            <div class="select">
                <el-checkbox v-model="selectedTrain" label="火车订票" size="large" />
                <el-checkbox v-model="selectedHotel" label="酒店预订" size="large" />
                <el-checkbox v-model="selectedFood" label="火车餐预订" size="large" />
                <el-checkbox v-model="isPayed" label="已支付" size="large" />
                <el-checkbox v-model="unPayed" label="未支付" size="large" />
                <el-date-picker class="date-picker" v-model="selectedDate" type="daterange" range-separator="-" start-placeholder="开始日期" end-placeholder="结束日期" @change="handleDateChange"/>
            </div>
        </div>
        <div class="container">
            <div v-for="(transactionDetail, index) in transactionDetailList" :key="index" style="width: 100%">
                <el-card v-if="selected(transactionDetail)" shadow="hover">
                    <div class="transaction-first-line">
                        <div>交易编号: {{ transactionDetail.id }}</div>
                        <div :class="{'green-text': transactionDetail.status === '已支付', 'red-text': transactionDetail.status === '未支付'}">{{ transactionDetail.status }}</div>
                    </div>
                    <div class="transaction-second-line">
                        <div class="transaction-detail" @click="showTransactionDetail(transactionDetail.id)">
                            <span>交易详情</span>
                            <el-icon color="gray" size="16px">
                                <div v-if="isShowTransactionDetail(transactionDetail.id)"><CaretBottom /></div>
                                <div v-else><CaretRight /></div>
                            </el-icon>
                        </div>
                        <div style="min-width: 150px">交易金额: {{ transactionDetail.money }}</div>
                        <div>创建时间: {{ transactionDetail.time }}</div>
                        <div style="min-width: 185px">付款时间: {{ transactionDetail.payTime == null ? '待付款' : transactionDetail.payTime }}</div>
                    </div>
                    <div v-if="isShowTransactionDetail(transactionDetail.id)">
                        <el-table :data="transactionDetail.orderDetail" stripe style="width: 100%">
                            <el-table-column fixed type="expand">
                                <template #default="props">
                                    <div v-if="props.row.type == '火车订票'" class="train-order-detail-container">
                                        <el-card class="InfoCard" shadow="always">
                                            <div class="TravelInfo">
                                                <div class="TravelDate">
                                                    <p style="margin-bottom: 0;">{{ orderMap.get(props.row.id).date }}</p>
                                                </div>
                                            </div>
                                            <div class="TicketInfo">
                                                <div class="Info-BigName">
                                                    <span style="font-weight: bolder;">{{ orderMap.get(props.row.id).depatureStation }}</span>
                                                    <span style="font-size: 14px;"> 站 </span>
                                                </div>
                                                <div class="Arrow">
                                                    <span class="Arrow-span">{{ orderMap.get(props.row.id).trainNumber }}</span>
                                                    <img class="Arrow-img" src="@/assets/TicketArrow.svg" />
                                                    <span class="Arrow-span">{{ orderMap.get(props.row.id).depatureTime }} 开</span>
                                                </div>
                                                <div class="Info-BigName">
                                                    <span style="font-weight: bolder;">{{ orderMap.get(props.row.id).reachStation }}</span>
                                                    <span style="font-size: 14px;"> 站 </span>
                                                </div>
                                                <div class="Info-BigName">
                                                    <p style="margin-bottom: 0;">{{ orderMap.get(props.row.id).seatInfo }}</p>
                                                </div>
                                                <div class="Info-BigName">
                                                    <p style="margin-bottom: 0;">{{ orderMap.get(props.row.id).name }}</p>
                                                </div>
                                            </div>
                                        </el-card>
                                    </div>
                                    <div v-if="props.row.type == '酒店预订'" class="hotel-order-detail-container">
                                        <el-card class="InfoCard" shadow="always">
                                            <div class="RoomInfo">
                                                <div class="RoomNumber">
                                                    <p style="margin-bottom: 0;">{{ orderMap.get(props.row.id).number }} 间</p>
                                                </div>
                                            </div>
                                            <div class="HotelInfo">
                                                <div class="Info-BigName">
                                                    <span style="font-weight: bolder;">{{ orderMap.get(props.row.id).hotelName }}</span>
                                                </div>
                                                <div class="Info-BigName">
                                                    <p style="margin-bottom: 0;">{{ orderMap.get(props.row.id).roomType }}</p>
                                                </div>
                                                <div class="Info-SmaleName">
                                                    <p style="margin-bottom: 0;">入住日期: {{ orderMap.get(props.row.id).beginDate }}</p>
                                                    <p style="margin-bottom: 0;">离店日期: {{ orderMap.get(props.row.id).endDate }}</p>
                                                </div>
                                                <div class="Info-BigName">
                                                    <p style="margin-bottom: 0;">{{ orderMap.get(props.row.id).name }}</p>
                                                </div>
                                            </div>
                                        </el-card>
                                    </div>
                                    <div v-if="props.row.type == '火车餐预订'" class="food-order-detail-container">
                                        <el-card class="InfoCard" shadow="always">
                                            <div class="TravelInfo">
                                                <div class="TravelDate">
                                                    <p style="margin-bottom: 0;">{{ orderMap.get(props.row.id).date }}</p>
                                                </div>
                                            </div>
                                            <div class="FoodInfo">
                                                <div class="Info-BigName">
                                                    <span style="font-weight: bolder;">{{ orderMap.get(props.row.id).shopName }}</span>
                                                </div>
                                                <div class="Info-BigName">
                                                    <p style="margin-bottom: 0;">{{ orderMap.get(props.row.id).foodName }}</p>
                                                </div>
                                                <div class="Info-BigName">
                                                    <p style="margin-bottom: 0;">{{ orderMap.get(props.row.id).trainNumber }}</p>
                                                </div>
                                                <div class="Info-SmaleName">
                                                    <p style="margin-bottom: 0;">预计时间: {{ orderMap.get(props.row.id).time }}</p>
                                                    <p v-if="orderMap.get(props.row.id).station != ''" style="margin-bottom: 0;">{{ orderMap.get(props.row.id).station }}站</p>
                                                </div>
                                                <div class="Info-BigName">
                                                    <p style="margin-bottom: 0;">{{ orderMap.get(props.row.id).name }}</p>
                                                </div>
                                            </div>
                                        </el-card>
                                    </div>
                                </template>
                            </el-table-column>
                            <el-table-column prop="id" label="订单ID" width="200" />
                            <el-table-column prop="type" label="订单类型" width="200" />
                            <el-table-column prop="status" label="订单状态" width="200" />
                            <el-table-column prop="money" label="订单金额" width="200"/>
                            <el-table-column fixed="right" label="操作" min-width="150">
                                <template #default>
                                    <el-button text type="danger" size="16px">取消订单</el-button>
                                </template>
                            </el-table-column>
                        </el-table>
                    </div>
                </el-card>
            </div>
        </div>
    </div>
</template>

<script>
import dayjs from 'dayjs';
import 'dayjs/locale/zh-cn';
import { useTransactionStore } from '@/stores/transaction'
dayjs.locale('zh-cn');

const transaction = useTransactionStore();

export default {
    data(){
        return {
            selectedTrain: true,
            selectedHotel: true,
            selectedFood: true,
            isPayed: true,
            unPayed: true,
            selectedDate: "",
            selectedStartDate: "",
            selectedEndDate: "",
            isShowTransactionDetailMap: new Map(), //是否展示交易详情
            transactionDetailList: transaction.transactionDetailList, 
            orderMap: transaction.orderMap,
        }
    },
    created: function() {
        this.create();
    },
    methods: {
        create() {
            //生成debug数据
            this.createDataForTest();



            transaction.initTransactionDetailList();
        },
        isShowTransactionDetail (transactionId) {
            return this.isShowTransactionDetailMap.get(transactionId);
        },
        showTransactionDetail(transactionId) {
            this.isShowTransactionDetailMap.set(transactionId, !this.isShowTransactionDetailMap.get(transactionId));
        },
        selected(transactionDetail) {
            let selectedPay =  ((transactionDetail.status == '已支付') && this.isPayed) || ((transactionDetail.status == '未支付') && this.unPayed);
            let selectedType = false;
            for(let order of transactionDetail.orderDetail) {
                if(((order.type == '火车订票') && this.selectedTrain) || ((order.type == '酒店预订') && this.selectedHotel) || ((order.type == '火车餐预订') && this.selectedFood)){
                    selectedType = true;
                    break;
                }
            }
            let selectedTime = ((transactionDetail.time >= this.selectedStartDate) && (transactionDetail.time <= this.selectedEndDate)) || this.selectedDate == '';
            return selectedPay && selectedType  && selectedTime;
        },
        handleDateChange(dateRange) {
            if(dateRange){
                const [startDate, endDate] = this.selectedDate;
                this.selectedStartDate = dayjs(startDate).format('YYYY-MM-DD HH:mm');
                this.selectedEndDate = dayjs(endDate).format('YYYY-MM-DD HH:mm');
            } else {
                this.selectedDate = '';
            }
        },
        //测试数据
        createDataForTest() {
            transaction.setTransactionMap({
                id: "3x124gh234", 
                status: "已支付",
                time: "2025-04-09 10:20",
                payTime: "2025-04-09 10:21", 
                money: "SC 500", 
                orderDetail: [{ 
                    id: "0x0001", //订单编号
                    status: "已完成", //订单状态
                    money: "SC 400", //订单金额
                    type: "火车订票", 
                },
                {
                    id: "0x0002",
                    status: "未完成",
                    money: "SC 80",
                    type: "火车订票", 
                },
                {
                    id: "0x0003",
                    status: "进行中",
                    money: "SC 20",
                    type: "火车订票", 
                }], 
            });
            transaction.setOrderMap({
                id: "0x0001",
                depatureStation: "北京南",
                reachStation: "南京南",
                trainNumber: "G1",
                date: "2025-04-20",
                depatureTime: "07:20",
                name: "张三",
                seatInfo: "03车12A 二等座", 
            });
            transaction.setOrderMap({
                id: "0x0002",
                depatureStation: "南京南",
                reachStation: "上海虹桥",
                trainNumber: "G287",
                date: "2025-04-20",
                depatureTime: "11:35",
                name: "张三",
                seatInfo: "05车01E 二等座",
            });
            transaction.setOrderMap({
                id: "0x0003",
                depatureStation: "上海虹桥",
                reachStation: "上海",
                trainNumber: "D1088",
                date: "2025-04-20",
                depatureTime: "14:08",
                name: "张三",
                seatInfo: "05车10A 二等座",
            });
            transaction.setTransactionMap({
                id: "68954cdf75", 
                status: "未支付",
                time: "2025-04-20 20:06", 
                money: "SC 300", 
                orderDetail: [{ 
                    id: "0x0089", //订单编号
                    status: "未支付", //订单状态
                    money: "SC 300", //订单金额
                    type: "酒店预订", 
                }], 
            });
            transaction.setOrderMap({
                id: "0x0089",
                hotelName: "日升大酒店",
                roomType: "大床房",
                beginDate: "2025-04-20",
                endDate: "2025-04-21",
                name: "张三",
                number: "1",
            });
            transaction.setTransactionMap({
                id: "05we7df58w", 
                status: "已支付",
                time: "2025-04-15 07:48",
                payTime: "2025-04-15 09:22", 
                money: "SC 80", 
                orderDetail: [{ 
                    id: "0x8888", //订单编号
                    status: "已完成", //订单状态
                    money: "SC 30", //订单金额
                    type: "火车餐预订", 
                },
                {
                    id: "0x8756",
                    status: "已完成",
                    money: "SC 50",
                    type: "火车餐预订", 
                }], 
            });
            transaction.setOrderMap({
                id: "0x8888",
                shopName: "餐车",
                foodName: "牛肉饭",
                trainNumber: "G1",
                station: "",
                date: "YYYY-MM-DD",
                time: "HH:mm",
                name: "张三",
            });
            transaction.setOrderMap({
                id: "0x8756",
                shopName: "KFC",
                foodName: "套餐A",
                trainNumber: "G1",
                station: "天津南",
                date: "YYYY-MM-DD",
                time: "HH:mm",
                name: "张三",
            });
            this.isShowTransactionDetailMap.set('3x124gh234', false);
            this.isShowTransactionDetailMap.set('68954cdf75', false);
            this.isShowTransactionDetailMap.set('05we7df58w', false);
        }
    }
}
</script>

<style scoped>
.container {
    margin-left: 12%;
    margin-bottom: 10px;
    max-width: 70%;
    display: flex;
    justify-content: center;
    flex-wrap: wrap;
}
.select > * {
    margin-right: 15px; 
    margin-bottom: 5px;
}
.el-card {
    border: 1px solid #409EFF; 
    margin-bottom: 10px;
}
.transaction-first-line {
    display: flex; /* 使用flex布局 */
    justify-content: space-between; /* 左右两端对齐 */
    width: 100%; /* 容器宽度占满父元素 */
    padding-right: 10px; /* 可选：添加一些内边距 */
    padding-left: 0;
    padding-top: 0;
    box-sizing: border-box; /* 可选：包含内边距和边框在内的宽度 */
}
.green-text {
    color: green;
}
.red-text {
    color: red;
}
.transaction-second-line{
    display: flex; /* 使用flex布局 */
    justify-content: space-between; /* 左右两端对齐 */
    width: 100%; /* 容器宽度占满父元素 */
    padding-right: 10px; /* 可选：添加一些内边距 */
    padding-left: 0;
    padding-top: 0;
    box-sizing: border-box; /* 可选：包含内边距和边框在内的宽度 */
}
.order-detail-container {
    display: flex; /* 使用flex布局 */
    justify-content: space-between; /* 左右两端对齐 */
    width: 100%; /* 容器宽度占满父元素 */
    padding-right: 30px; /* 可选：添加一些内边距 */
    padding-left: 50px;
    padding-top: 0;
    box-sizing: border-box; /* 可选：包含内边距和边框在内的宽度 */
}
.train-order-detail-container {
    display: flex;
    justify-content: center;
    width: 100%;
    padding-top: 0;
}
.hotel-order-detail-container {
    display: flex;
    justify-content: center;
    width: 100%;
    padding-top: 0;
}
.food-order-detail-container {
    display: flex;
    justify-content: center;
    width: 100%;
    padding-top: 0;
}
.transaction-detail {
    display: flex;
    align-items: center; /* 垂直居中对齐 */
    gap: 0; /* 水平间距 */
    cursor: pointer;
}
.stage-text {
    display: flex;
    align-items: center; /* 垂直居中对齐 */
    gap: 0; /* 水平间距 */
    min-width: 250px;
}

.InfoCard {
    width: 90%;
    margin-top: 5px;
    border-radius: 15px;
    height: 140px;
}

.el-card__body {
    width: 100%;
    height: 100%;
    padding-left: 40px;
    padding-top: 40px; 
    padding-bottom: 40px;
}

.TravelInfo {
    display: flex;
    justify-content: space-between; /* 子元素两端对齐 */
    align-items: center; /* 垂直居中 */
}

.RoomInfo {
    display: flex;
    justify-content: right; /* 子元素两端对齐 */
    align-items: center; /* 垂直居中 */
}

.TravelDate {
    text-align: left;
    font-size: 16px;
    margin-left: 10px;
}
.RoomNumber {
    text-align: left;
    font-size: 16px;
    margin-right: 10px;
}

.TicketInfo {
    display: grid;
    grid-template-columns: 100px 250px 100px 250px 150px;
    justify-content: space-between; /* 子元素两端对齐 */
    margin-left: 40px;
    align-items: center; /* 垂直居中 */
}

.HotelInfo {
    display: grid;
    grid-template-columns: 200px 100px 200px 150px;
    justify-content: space-between; /* 子元素两端对齐 */
    margin-left: 0;
    margin-right: 20px;
    align-items: center; /* 垂直居中 */
    margin-top: 5px;
}

.FoodInfo {
    display: grid;
    grid-template-columns: 200px 200px 50px 200px 150px;
    justify-content: space-between; /* 子元素两端对齐 */
    align-items: center; /* 垂直居中 */
    margin-top: 15px;
}

.Arrow {
    display: flex;
    flex-direction: column;
}

.Arrow-span {
    text-align: center;
    font-size: 1.25rem;
    font-weight: bold;
    padding: 0;
    height: 25px;
    color: rgb(142, 143, 146);
}

.Arrow-img {
    width: 200px;
    height: auto;
    margin-left: 20px;
    margin-right: 20px;
}

.Info-BigName {
    text-align: center;
    font-size: 1.25rem;
}

.Info-SmaleName {
    text-align: center;
    font-size: 16px;
}

</style>

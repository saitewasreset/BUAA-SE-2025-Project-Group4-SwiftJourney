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
                        <el-table :data="transactionDetail.orderInfo" stripe style="width: 100%">
                            <el-table-column fixed type="expand">
                                <template #default="props">
                                    <div v-if="props.row.type == '火车订票'" class="train-order-detail-container">
                                        <el-card class="InfoCard" shadow="always">
                                            <div class="TravelInfo">
                                                <div class="TravelDate">
                                                    <p style="margin-bottom: 0;">{{ getOrderMap(props.row.id, "date") }}</p>
                                                </div>
                                            </div>
                                            <div class="TicketInfo">
                                                <div class="Info-BigName">
                                                    <span style="font-weight: bolder;">{{ getOrderMap(props.row.id, "depatureStation") }}</span>
                                                    <span style="font-size: 14px;"> 站 </span>
                                                </div>
                                                <div class="Arrow">
                                                    <span class="Arrow-span">{{ getOrderMap(props.row.id, "trainNumber") }}</span>
                                                    <img class="Arrow-img" src="@/assets/TicketArrow.svg" />
                                                    <span class="Arrow-span">{{ getOrderMap(props.row.id, "depatureTime") }} 开</span>
                                                </div>
                                                <div class="Info-BigName">
                                                    <span style="font-weight: bolder;">{{ getOrderMap(props.row.id, "reachStation") }}</span>
                                                    <span style="font-size: 14px;"> 站 </span>
                                                </div>
                                                <div class="Info-BigName">
                                                    <p style="margin-bottom: 0;">{{ getOrderMap(props.row.id, "seatInfo") }}</p>
                                                </div>
                                                <div class="Info-BigName">
                                                    <p style="margin-bottom: 0;">{{ getOrderMap(props.row.id, "name") }}</p>
                                                </div>
                                            </div>
                                        </el-card>
                                    </div>
                                    <div v-if="props.row.type == '酒店预订'" class="hotel-order-detail-container">
                                        <el-card class="InfoCard" shadow="always">
                                            <div class="RoomInfo">
                                                <div class="RoomNumber">
                                                    <p style="margin-bottom: 0;">{{ getOrderMap(props.row.id, "number") }} 间</p>
                                                </div>
                                            </div>
                                            <div class="HotelInfo">
                                                <div class="Info-BigName">
                                                    <span style="font-weight: bolder;">{{ getOrderMap(props.row.id, "hotelName") }}</span>
                                                </div>
                                                <div class="Info-BigName">
                                                    <p style="margin-bottom: 0;">{{ getOrderMap(props.row.id, "roomType") }}</p>
                                                </div>
                                                <div class="Info-SmaleName">
                                                    <p style="margin-bottom: 0;">入住日期: {{ getOrderMap(props.row.id, "beginDate") }}</p>
                                                    <p style="margin-bottom: 0;">离店日期: {{ getOrderMap(props.row.id, "endDate") }}</p>
                                                </div>
                                                <div class="Info-BigName">
                                                    <p style="margin-bottom: 0;">{{ getOrderMap(props.row.id, "name") }}</p>
                                                </div>
                                            </div>
                                        </el-card>
                                    </div>
                                    <div v-if="props.row.type == '火车餐预订'" class="food-order-detail-container">
                                        <el-card class="InfoCard" shadow="always">
                                            <div class="TravelInfo">
                                                <div class="TravelDate">
                                                    <p style="margin-bottom: 0;">{{ getOrderMap(props.row.id, "date") }}</p>
                                                </div>
                                            </div>
                                            <div class="FoodInfo">
                                                <div class="Info-BigName">
                                                    <span style="font-weight: bolder;">{{ getOrderMap(props.row.id, "shopName") }}</span>
                                                </div>
                                                <div class="Info-BigName">
                                                    <p style="margin-bottom: 0;">{{ getOrderMap(props.row.id, "foodName") }}</p>
                                                </div>
                                                <div class="Info-BigName">
                                                    <p style="margin-bottom: 0;">{{ getOrderMap(props.row.id, "trainNumber") }}</p>
                                                </div>
                                                <div class="Info-SmaleName">
                                                    <p style="margin-bottom: 0;">预计时间: {{ getOrderMap(props.row.id, "time") }}</p>
                                                    <p v-if="getOrderMap(props.row.id, 'station') != ''" style="margin-bottom: 0;">{{ getOrderMap(props.row.id, "station") }}站</p>
                                                </div>
                                                <div class="Info-BigName">
                                                    <p style="margin-bottom: 0;">{{ getOrderMap(props.row.id, "name") }}</p>
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

<script lang="ts">
import dayjs from 'dayjs';
import 'dayjs/locale/zh-cn';
import { orderApi } from "@/api/orderApi/orderApi";
import type { ResponseData, TransactionData, OrderInfo, SeatLocationInfo, TrainOrderInfo, HotelOrderInfo, DishOrderInfo, TakeawayOrderInfo,
    OrderInform, TransactionDetail, OrderDetail, TrainOrderDetail, HotelOrderDetail, FoodOrderDetail } from '@/interface/interface';
dayjs.locale('zh-cn');

const statusChangeTab = {
    unpaid: "未支付",
    paid: "已支付",
    ongoing: "进行中",
    active: "",
    completed: "已完成",
    failed: "失败",
    canceled: "已取消",
}

const typeChangeTab = {
    train: "火车订票",
    hotel: "酒店预订",
    dish: "火车餐预订",
    takeaway: "火车餐预订",
}

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
            isShowTransactionDetailMap: new Map<string, Boolean>(), //是否展示交易详情
            transactionMap: new Map<string, TransactionDetail>(),
            transactionList: [] as string[],
            orderMap: new Map<string, OrderDetail>(),
            transactionDetailList: [] as TransactionDetail [],
        }
    },
    created: function() {
        //测试数据
        //this.debugInit();
        
        //访问后端
        this.init();
        for(let tr of this.transactionDetailList) {
            this.isShowTransactionDetailMap.set(tr.id, false);
        }
    },
    methods: {
        isShowTransactionDetail (transactionId: string) {
            return this.isShowTransactionDetailMap.get(transactionId);
        },
        showTransactionDetail(transactionId: string) {
            this.isShowTransactionDetailMap.set(transactionId, !this.isShowTransactionDetailMap.get(transactionId));
        },
        selected(transactionDetail: TransactionDetail) {
            let selectedPay =  ((transactionDetail.status == '已支付') && this.isPayed) || ((transactionDetail.status == '未支付') && this.unPayed);
            let selectedType = false;
            for(let order of transactionDetail.orderInfo) {
                if(((order.type == '火车订票') && this.selectedTrain) || ((order.type == '酒店预订') && this.selectedHotel) || ((order.type == '火车餐预订') && this.selectedFood)){
                    selectedType = true;
                    break;
                }
            }
            let selectedTime = ((transactionDetail.time >= this.selectedStartDate) && (transactionDetail.time <= this.selectedEndDate)) || this.selectedDate == '';
            return selectedPay && selectedType  && selectedTime;
        },
        handleDateChange(dateRange: any) {
            if(dateRange){
                const [startDate, endDate] = this.selectedDate;
                this.selectedStartDate = dayjs(startDate).format('YYYY-MM-DD HH:mm');
                this.selectedEndDate = dayjs(endDate).format('YYYY-MM-DD HH:mm');
            } else {
                this.selectedDate = '';
            }
        },

        getOrderMap(id: string, name: string): any {
            let order = this.orderMap.get(id);
            if(order && name in order){
                return (order as any)[name];
            }
        },

        //-----------init相关----------------//
        setTransactionMap(transactionDetail: TransactionDetail): void {
            this.transactionMap.set(transactionDetail.id, transactionDetail);
        },

        setOrderMap(orderDetail: OrderDetail): void {
            this.orderMap.set(orderDetail.id, orderDetail);
        },

        initTransactionDetailList(): void {
            this.transactionMap.forEach((value, key) => {
                this.transactionList.push(key);
            });
            this.transactionListSort();
            this.transactionWithMaps();
        },

        transactionListSort(): string[] {
            return this.transactionList.sort((a, b) => {
                const transactionA = this.transactionMap.get(a);
                const transactionB = this.transactionMap.get(b);

                if (!transactionA || !transactionB) {
                    // 如果某个交易不存在，则将其视为较后
                    if (!transactionA) return 1;
                    if (!transactionB) return -1;
                }

                const time1 = transactionA.time;
                const time2 = transactionB.time;
                if(time1 < time2) return 1;
                if(time1 > time2) return -1;
                return 0;
            });
        },

        transactionWithMaps(): void {
            for(let id of this.transactionList) {
                const transactionDetail = this.transactionMap.get(id);
                if (transactionDetail) { // 检查是否为 undefined
                    this.transactionDetailList.push(transactionDetail);
                }
            }
        },

        async init() {
            await orderApi.orderList()
            .then((res) => {
                if(res.status == 200){
                    const resData: ResponseData = res.data;
                    this.dataHandle(resData);
                } else {
                    throw new Error(res.statusText);
                }
            })
            .catch((error) => {
                console.error(error);
            })
        },

        dataHandle(resData: ResponseData): void{
            for(const transactionData of resData){
                let transactionDetail: TransactionDetail = {
                    id: transactionData.transactionId,
                    status: statusChangeTab[transactionData.status],
                    time: transactionData.createTime,
                    payTime: transactionData.payTime,
                    money: String(transactionData.amount),
                    orderInfo: [] as OrderInform [],
                };
                for(const orderInfo of transactionData.orders){
                    let orderInform: OrderInform = {
                        id: orderInfo.orderId,
                        status: statusChangeTab[orderInfo.status],
                        type: "",
                        money: String(orderInfo.unitPrice),
                        canCanceled: orderInfo.canCancel,
                    };
                    switch(orderInfo.orderType){
                        case "train":
                            const trainOrderInfo = orderInfo as TrainOrderInfo;
                            let trainOrderDetail: TrainOrderDetail = {
                                id: trainOrderInfo.orderId,
                                name: trainOrderInfo.name,
                                depatureStation: trainOrderInfo.departureStation,
                                reachStation: trainOrderInfo.terminalStation,
                                trainNumber: trainOrderInfo.trainNumber,
                                date: trainOrderInfo.departureTime.substring(0, 10),
                                depatureTime: trainOrderInfo.departureTime.substring(11),
                                seatInfo: this.getSeat(trainOrderInfo.seat)
                            };
                            this.setOrderMap(trainOrderDetail);
                            break;
                        case "dish":
                            const dishOrderInfo = orderInfo as DishOrderInfo;
                            let dishOrederDetail: FoodOrderDetail = {
                                id: dishOrderInfo.orderId,
                                shopName: "餐车",
                                foodName: dishOrderInfo.dishName,
                                trainNumber: dishOrderInfo.trainNumber,
                                station: '',
                                date: dishOrderInfo.depatureTime.substring(0, 10),
                                time: dishOrderInfo.dishTime == "dinner" ? "晚餐" : "午餐",
                                name: dishOrderInfo.name,
                            }
                            this.setOrderMap(dishOrederDetail);
                            break;
                        case "hotel":
                            const hotelOrderInfo = orderInfo as HotelOrderInfo;
                            let hotelOrderDetail: HotelOrderDetail = {
                                id: hotelOrderInfo.orderId,
                                hotelName: hotelOrderInfo.hotelName,
                                roomType: hotelOrderInfo.roomType,
                                beginDate: hotelOrderInfo.beginDate,
                                endDate: hotelOrderInfo.endDate,
                                name: hotelOrderInfo.name,
                                number: hotelOrderInfo.amount,
                            }
                            this.setOrderMap(hotelOrderDetail);
                            break;
                        case "takeaway":
                            const takeawayOrderInfo = orderInfo as TakeawayOrderInfo;
                            let takeawayOrederDetail: FoodOrderDetail = {
                                id: takeawayOrderInfo.orderId,
                                shopName: takeawayOrderInfo.shopName,
                                foodName: takeawayOrderInfo.takeawayName,
                                trainNumber: takeawayOrderInfo.trainNumber,
                                station: takeawayOrderInfo.station,
                                date: takeawayOrderInfo.depatureTime.substring(0,10),
                                time: takeawayOrderInfo.dishTime,
                                name: takeawayOrderInfo.name,
                            }
                            this.setOrderMap(takeawayOrederDetail);
                            break;
                        }
                    orderInform.type = typeChangeTab[orderInfo.orderType];
                    transactionDetail.orderInfo.push(orderInform);
                }
                this.setTransactionMap(transactionDetail);
            }
            this.initTransactionDetailList();
        },

        getSeat(seat: SeatLocationInfo): string {
            return (seat.carriage < 10 ? `0${seat.carriage}` : `${seat.carriage}`) + "车" + seat.row + seat.location + " " + seat.type;
        },

        //测试数据
        debugInit(): void {
            const res: ResponseData = this.DebugList();
            this.dataHandle(res);
        },
        DebugList(): ResponseData {
            const debugData = [] as ResponseData;
            const trainTransactionData: TransactionData = {
                transactionId: "3x124gh234",
                status: "paid",
                createTime: "2025-04-09 10:20",
                payTime: "2025-04-09 10:21",
                amount: 500,
                orders: [
                    {
                        orderId: "0x0001",
                        status: "completed",
                        unitPrice: 400,
                        amount: 1,
                        orderType: "train",
                        canCancel: false,
                        trainNumber: "G1",
                        departureStation: "北京南",
                        terminalStation: "南京南",
                        departureTime: "2025-04-20 07:20",
                        terminalTime: '',
                        name: "张三",
                        seat: {
                            carriage: 3,
                            row: 12,
                            location: "A",
                            type: "二等座",
                        },
                    } as TrainOrderInfo,
                    {
                        orderId: "0x0002",
                        status: "ongoing",
                        unitPrice: 80,
                        amount: 1,
                        orderType: "train",
                        canCancel: false,
                        trainNumber: "G287",
                        departureStation: "南京南",
                        terminalStation: "上海虹桥",
                        departureTime: "2025-04-20 11:35",
                        terminalTime: '',
                        name: "张三",
                        seat: {
                            carriage: 5,
                            row: 1,
                            location: "E",
                            type: "二等座",
                        },
                    } as TrainOrderInfo,
                    {
                        orderId: "0x0003",
                        status: "ongoing",
                        unitPrice: 20,
                        amount: 1,
                        orderType: "train",
                        canCancel: false,
                        trainNumber: "D1088",
                        departureStation: "上海虹桥",
                        terminalStation: "上海",
                        departureTime: "2025-04-20 14:08",
                        terminalTime: '',
                        name: "张三",
                        seat: {
                            carriage: 12,
                            row: 3,
                            location: "B",
                            type: "二等座",
                        },
                    } as TrainOrderInfo,
                ]
            };
            debugData.push(trainTransactionData);

            const hotelTransactionData: TransactionData = {
                transactionId: "68954cdf75",
                status: "unpaid",
                createTime: "2025-04-20 20:06",
                amount: 300,
                orders: [
                    {
                        orderId: "0x0089",
                        status: "unpaid",
                        unitPrice: 300,
                        amount: 1,
                        orderType: "hotel",
                        canCancel: false,
                        hotelName: "日升大酒店",
                        roomType: "大床房",
                        hotelId: '',
                        beginDate: "2025-04-20",
                        endDate: "2025-04-21",
                        name: "张三"
                    } as HotelOrderInfo
                ]
            }
            debugData.push(hotelTransactionData);

            const foodTransactionData: TransactionData = {
                transactionId: "05we7df58w",
                status: "paid",
                createTime: "2025-04-15 07:48",
                payTime: "2025-04-15 09:22",
                amount: 80,
                orders: [
                    {
                        orderId: "0x8888",
                        status: "completed",
                        unitPrice: 50,
                        amount: 1,
                        orderType: "dish",
                        canCancel: false,
                        trainNumber: "G1",
                        depatureTime: "2025-04-20 07:20",
                        dishName: "牛肉饭",
                        dishTime: "lunch",
                        name: "张三"
                    } as DishOrderInfo,
                    {
                        orderId: "0x8756",
                        status: "completed",
                        unitPrice: 30,
                        amount: 1,
                        orderType: "takeaway",
                        canCancel: false,
                        trainNumber: "G1",
                        station: "天津南",
                        depatureTime: "2025-04-20 07:20",
                        dishTime: "2025-04-20 08:40",
                        shopName: "KFC",
                        takeawayName: "套餐B",
                        name: "张三"
                    } as TakeawayOrderInfo,
                ]
            }
            debugData.push(foodTransactionData);

            return debugData;
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

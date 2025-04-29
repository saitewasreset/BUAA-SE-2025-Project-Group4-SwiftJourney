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
                        <div style="min-width: 150px">交易类型: {{ transactionDetail.type }}</div>
                        <div style="min-width: 150px">交易金额: {{ transactionDetail.money }}</div>
                        <div>交易时间: {{ transactionDetail.time }}</div>
                    </div>
                    <div v-if="isShowTransactionDetail(transactionDetail.id)">
                        <el-table :data="transactionDetail.orderDetail" stripe style="width: 100%">
                            <el-table-column fixed type="expand">
                                <template #default="props">
                                    <div v-if="transactionDetail.type == '火车订票'" class="train-order-detail-container">
                                        <div class="stage-text" style="font-size: 16px; font-weight: bold;">
                                            {{ orderMap.get(props.row.id).depatureStation }}站
                                            <el-icon size="24px"><Right /></el-icon>
                                            {{ orderMap.get(props.row.id).reachStation }}站   {{ orderMap.get(props.row.id).trainNumber }}
                                        </div>
                                        <div>{{ orderMap.get(props.row.id).seatInfo }}</div>
                                        <div>{{ orderMap.get(props.row.id).depatureTime }} 开</div>
                                        <div>{{ orderMap.get(props.row.id).name }}</div>
                                    </div>
                                    <div v-if="transactionDetail.type == '酒店预订'" class="hotel-order-detail-container">
                                        <div class="stage-text" style="font-size: 16px; font-weight: bold;">
                                            {{ orderMap.get(props.row.id).hotelName }}
                                        </div>
                                        <div>{{ orderMap.get(props.row.id).roomType }}</div>
                                        <div>{{ orderMap.get(props.row.id).number }} 间</div>
                                        <div>{{ orderMap.get(props.row.id).beginDate }} - {{ orderMap.get(props.row.id).endDate }}</div>
                                        <div>{{ orderMap.get(props.row.id).name }}</div>
                                    </div>
                                    <div v-if="transactionDetail.type == '火车餐预订'" class="food-order-detail-container">
                                        <div class="stage-text" style="font-size: 16px; font-weight: bold;">
                                            {{ orderMap.get(props.row.id).shopName }}
                                        </div>
                                        <div>{{ orderMap.get(props.row.id).foodName }}</div>
                                        <div>{{ orderMap.get(props.row.id).trainNumber }}</div>
                                        <div v-if="orderMap.get(props.row.id).station != ''">{{ orderMap.get(props.row.id).station }}站</div>
                                        <div v-else></div>
                                        <div>{{ orderMap.get(props.row.id).time }}</div>
                                        <div>{{ orderMap.get(props.row.id).name }}</div>
                                    </div>
                                </template>
                            </el-table-column>
                            <el-table-column prop="id" label="订单ID" width="250" />
                            <el-table-column prop="status" label="订单状态" width="250" />
                            <el-table-column prop="money" label="订单金额" width="250"/>
                            <el-table-column fixed="right" label="操作" min-width="200">
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
dayjs.locale('zh-cn');

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
            transactionMap: new Map(),
            isShowTransactionDetailMap: new Map(), //是否展示交易详情
            transactionList: [], //按照时间降序排列transaction_id
            transactionDetailList: [], //由transactionWithMaps()生成
            orderMap: new Map(),
            /*
            transactionDetail: {
                id: "", //交易编号
                status: "", //交易状态
                type: "", //交易类型
                time: "", //交易时间
                money: "", //交易金额
                orderDetail: [], //订单表
            },*/
            /*
            orderDetail: [{
                id: "xxxxxxxx", //订单编号
                status: "已完成", //订单状态
                money: "SC 11111111", //订单金额
                canCanceled: "", //是否可以被取消
                //其他待添加信息
            },
            {
                id: "1",
                status: "?",
                money: "SC 1000",
            },
            {
                id: "2",
                status: "!",
                money: "SC 0000",
            }
            ],*/
            /*
            trainOrderInfo: {
                id: "",
                depatureStation: "",
                reachStation: "",
                trainNumber: "",
                depatureTime: "",
                name: "",
                seatInfo: ""
            },
            hotelOrderInfo: {
                id: "",
                hotelName: "",
                roomType: "",
                beginDate: "",
                endDate: "",
                name: "",
                number: "",
            },
            foodOrderInfo: {
                id: "",
                type: "",
                shopName: "",
                foodName: "",
                trainNumber: "",
                station: "",
                time: "",
                name: "",
            }*/
        }
    },
    created: function() {
        this.create();
    },
    methods: {
        create() {
            this.transactionMap.set('3x124gh234', {
                id: "3x124gh234", 
                status: "已支付",
                type: "火车订票", 
                time: "2025-04-09 10:20", 
                money: "SC 500", 
                orderDetail: [{ 
                    id: "0x0001", //订单编号
                    status: "已完成", //订单状态
                    money: "SC 400", //订单金额
                },
                {
                    id: "0x0002",
                    status: "未完成",
                    money: "SC 80",
                },
                {
                    id: "0x0003",
                    status: "进行中",
                    money: "SC 20",
                }], 
            });
            this.orderMap.set('0x0001', {
                id: "0x0001",
                depatureStation: "北京南",
                reachStation: "南京南",
                trainNumber: "G1",
                depatureTime: "2025-04-20 07:20",
                name: "张三",
                seatInfo: "03车12A 二等座"
            });
            this.orderMap.set('0x0002', {
                id: "0x0002",
                depatureStation: "南京南",
                reachStation: "上海虹桥",
                trainNumber: "G287",
                depatureTime: "2025-04-20 11:35",
                name: "张三",
                seatInfo: "05车01E 二等座"
            });
            this.orderMap.set('0x0003', {
                id: "0x0003",
                depatureStation: "上海虹桥",
                reachStation: "上海",
                trainNumber: "D1088",
                depatureTime: "2025-04-20 14:08",
                name: "张三",
                seatInfo: "05车10A 二等座"
            });
            this.transactionMap.set('68954cdf75', {
                id: "68954cdf75", 
                status: "未支付",
                type: "酒店预订", 
                time: "2025-04-20 20:06", 
                money: "SC 300", 
                orderDetail: [{ 
                    id: "0x0089", //订单编号
                    status: "未支付", //订单状态
                    money: "SC 300", //订单金额
                }], 
            });
            this.orderMap.set('0x0089', {
                id: "0x0089",
                hotelName: "日升大酒店",
                roomType: "大床房",
                beginDate: "2025-04-20",
                endDate: "2025-04-21",
                name: "张三",
                number: "1",
            });
            this.transactionMap.set('05we7df58w', {
                id: "05we7df58w", 
                status: "已支付",
                type: "火车餐预订", 
                time: "2025-04-15 07:48", 
                money: "SC 80", 
                orderDetail: [{ 
                    id: "0x8888", //订单编号
                    status: "已完成", //订单状态
                    money: "SC 30", //订单金额
                },
                {
                    id: "0x8756",
                    status: "已完成",
                    money: "SC 50",
                }], 
            });
            this.orderMap.set('0x8888', {
                id: "0x8888",
                shopName: "餐车",
                foodName: "牛肉饭",
                trainNumber: "G1",
                station: "",
                time: "YYYY-MM-DD HH:mm",
                name: "张三",
            });
            this.orderMap.set('0x8756', {
                id: "0x8756",
                shopName: "KFC",
                foodName: "套餐A",
                trainNumber: "G1",
                station: "天津南",
                time: "YYYY-MM-DD HH:mm",
                name: "张三",
            });
            this.isShowTransactionDetailMap.set('3x124gh234', false);
            this.isShowTransactionDetailMap.set('68954cdf75', false);
            this.isShowTransactionDetailMap.set('05we7df58w', false);
            this.transactionList.push('3x124gh234');
            this.transactionList.push('68954cdf75');
            this.transactionList.push('05we7df58w');
            this.transactionListSort();
            this.transactionWithMaps();
        },
        transactionListSort() {
            return this.transactionList.sort((a, b) => {
                const time1 = this.transactionMap.get(a).time;
                const time2 = this.transactionMap.get(b).time;
                if(time1 < time2) return 1;
                if(time1 > time2) return -1;
                return 0;
            });
        },
        transactionWithMaps() {
            for(let id of this.transactionList) {
                this.transactionDetailList.push(this.transactionMap.get(id));
            }
        },
        isShowTransactionDetail (transactionId) {
            return this.isShowTransactionDetailMap.get(transactionId);
        },
        showTransactionDetail(transactionId) {
            this.isShowTransactionDetailMap.set(transactionId, !this.isShowTransactionDetailMap.get(transactionId));
        },
        selected(transactionDetail) {
            let selectedPay =  ((transactionDetail.status == '已支付') && this.isPayed) || ((transactionDetail.status == '未支付') && this.unPayed);
            let selectedType = ((transactionDetail.type == '火车订票') && this.selectedTrain) || ((transactionDetail.type == '酒店预订') && this.selectedHotel) || ((transactionDetail.type == '火车餐预订') && this.selectedFood);
            let selectedTime = ((transactionDetail.time >= this.selectedStartDate) && (transactionDetail.time <= this.selectedEndDate)) || this.selectedDate == '';
            return selectedPay && selectedType && selectedTime;
        },
        handleDateChange(dateRange) {
            if(dateRange){
                const [startDate, endDate] = this.selectedDate;
                this.selectedStartDate = dayjs(startDate).format('YYYY-MM-DD HH:mm');
                this.selectedEndDate = dayjs(endDate).format('YYYY-MM-DD HH:mm');
            } else {
                this.selectedDate = '';
            }
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
    display: grid;
    grid-template-columns: 250px 150px 150px 50px;
    justify-content: space-between;
    width: 100%;
    padding-right: 10px;
    padding-left: 50px;
    padding-top: 0;
}
.hotel-order-detail-container {
    display: grid;
    grid-template-columns: 150px 100px 50px 200px 50px;
    justify-content: space-between;
    width: 100%;
    padding-right: 10px;
    padding-left: 50px;
    padding-top: 0;
}
.food-order-detail-container {
    display: grid;
    grid-template-columns: 150px 150px 50px 100px 150px 50px;
    justify-content: space-between;
    width: 100%;
    padding-right: 10px;
    padding-left: 50px;
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
</style>

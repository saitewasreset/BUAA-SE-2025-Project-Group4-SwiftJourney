import { defineStore } from "pinia";

interface OrderInfo {
    id: string, //订单编号
    status: string, //订单状态
    type: string, //订单类型
    money: string, //订单金额
    canCanceled: string, //是否可以被取消
}

interface TransactionDetail {
    id: string, // 交易编号
    status: string, // 交易状态
    time: string, // 交易时间
    payTime: string, //付款时间
    money: string, // 交易金额
    orderInfo: OrderInfo[], // 订单表
}

interface OrderDetail {
    id: string, //订单号
    name: string, //姓名
}

interface TrainOrderDetail extends OrderDetail {
    depatureStation: string, //出发车站
    reachStation: string, //到达车站
    trainNumber: string, //车次
    date: string, //日期
    depatureTime: string, //出发时间
    seatInfo: string, //座位号 
}

interface HotelOrderDetail extends OrderDetail {
    hotelName: string, //酒店名
    roomType: string, //房型
    beginDate: string, //入住日期
    endDate: string, //退房日期
    number: number, //数量
}

interface FoodOrderDetail extends OrderDetail {
    shopName: string, //店铺名
    foodName: string, //食物名
    trainNumber: string, //车次
    station: string, //送餐车站(外卖)
    date: string, //日期
    time: string, //送餐时间
}



export const useTransactionStore = defineStore('transaction',{
    state: () => ({
        transactionMap: new Map<string, TransactionDetail>(),
        transactionList: [] as string [],
        orderMap: new Map<string, OrderDetail>(),
        transactionDetailList: [] as TransactionDetail [], //由transactionWithMaps()生成
    }),
    getters: {

    },
    actions: {
        setTransactionMap(transactionDetail: TransactionDetail) {
            this.transactionMap.set(transactionDetail.id, transactionDetail);
        },
        setOrderMap(orderDetail: OrderDetail) {
            this.orderMap.set(orderDetail.id, orderDetail);
        },
        initTransactionDetailList() {
            this.transactionMap.forEach((value, key) => {
                this.transactionList.push(key);
            });
            this.transactionListSort();
            this.transactionWithMaps();
        },
        transactionListSort() {
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
        transactionWithMaps() {
            for(let id of this.transactionList) {
                const transactionDetail = this.transactionMap.get(id);
                if (transactionDetail) { // 检查是否为 undefined
                    this.transactionDetailList.push(transactionDetail);
                }
            }
        },
    }
});
<template>
  <div class="transaction-container">
    <div class="transaction-card">
      <!-- 头部标题区域 -->
      <div class="card-header">
        <div class="header-content">
          <h1 class="page-title">交易记录</h1>
          <p class="page-subtitle">查看您的所有订单交易信息</p>
        </div>
        <!-- 添加统计信息 -->
        <div class="stats-info">
          <span class="stats-text">共 {{ filteredTransactionDetailList.length }} 条记录</span>
        </div>
      </div>

      <!-- 筛选器区域 -->
      <div class="filter-section">
        <div class="filter-grid">
          <div class="filter-group">
            <label class="filter-label">订单类型</label>
            <div class="checkbox-group">
              <el-checkbox v-model="selectedTrain" label="火车订票" />
              <el-checkbox v-model="selectedHotel" label="酒店预订" />
              <el-checkbox v-model="selectedFood" label="火车餐预订" />
            </div>
          </div>
          
          <div class="filter-group">
            <label class="filter-label">支付状态</label>
            <div class="checkbox-group">
              <el-checkbox v-model="isPayed" label="已支付" />
              <el-checkbox v-model="unPayed" label="未支付" />
            </div>
          </div>
          
          <div class="filter-group">
            <label class="filter-label">时间范围</label>
                <a-range-picker 
                    v-model:value="selectedDate" 
                    start-placeholder="开始日期" 
                    end-placeholder="结束日期" 
                    @change="handleDateChange"
                    class="date-picker"
                    :locale="locale"
                />
          </div>
        </div>
      </div>

      <!-- 分割线 -->
      <div class="divider"></div>

      <!-- 交易列表内容 -->
      <div class="transactions-content">
        <!-- 加载状态 -->
        <div v-if="loading" class="loading-state">
          <div class="skeleton-transactions">
            <div 
              v-for="n in 3" 
              :key="n" 
              class="skeleton-transaction-card"
              :style="{ animationDelay: `${(n - 1) * 0.1}s` }"
            >
              <!-- 骨架屏头部 -->
              <div class="skeleton-header">
                <div class="skeleton-transaction-info">
                  <div class="skeleton-id">
                    <div class="skeleton-text skeleton-id-label"></div>
                    <div class="skeleton-text skeleton-id-value"></div>
                  </div>
                </div>
                <div class="skeleton-status">
                  <div class="skeleton-badge"></div>
                </div>
              </div>

              <!-- 骨架屏详情 -->
              <div class="skeleton-details">
                <div class="skeleton-detail-row">
                  <div class="skeleton-detail-item">
                    <div class="skeleton-text skeleton-detail-label"></div>
                    <div class="skeleton-text skeleton-detail-value skeleton-amount"></div>
                  </div>
                  <div class="skeleton-detail-item">
                    <div class="skeleton-text skeleton-detail-label"></div>
                    <div class="skeleton-text skeleton-detail-value"></div>
                  </div>
                </div>
                <div class="skeleton-detail-row">
                  <div class="skeleton-detail-item">
                    <div class="skeleton-text skeleton-detail-label"></div>
                    <div class="skeleton-text skeleton-detail-value"></div>
                  </div>
                  <div class="skeleton-detail-item">
                    <div class="skeleton-text skeleton-detail-label"></div>
                    <div class="skeleton-text skeleton-detail-value"></div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-else-if="!loading && filteredTransactionDetailList.length === 0" class="empty-state">
          <div class="empty-icon">
            <el-icon size="64"><DocumentRemove /></el-icon>
          </div>
          <h3 class="empty-title">暂无交易记录</h3>
          <p class="empty-subtitle">当前筛选条件下没有找到相关交易记录</p>
        </div>

        <!-- 实际数据 -->
        <div v-else>
          <!-- 分页后的交易网格 -->
          <div class="transactions-grid">
            <div 
              v-for="(transactionDetail, index) in paginatedTransactionList" 
              :key="transactionDetail.id"
              class="transaction-card-item"
              :style="{ animationDelay: `${index * 0.1}s` }"
            >
              <!-- 交易卡片内容保持不变 -->
              <!-- 交易卡片头部 -->
              <div class="transaction-header">
                <div class="transaction-info">
                  <div class="transaction-id">
                    <span class="id-label">交易编号</span>
                    <span class="id-value">{{ transactionDetail.id }}</span>
                  </div>
                  <div class="transaction-status">
                    <span 
                      class="status-badge"
                      :class="{ 
                        'status-paid': transactionDetail.status === '已支付', 
                        'status-unpaid': transactionDetail.status === '未支付' 
                      }"
                    >
                      {{ transactionDetail.status }}
                    </span>
                  </div>
                </div>
              </div>

              <!-- 交易详情 -->
              <div class="transaction-details">
                <div class="detail-row">
                  <div class="detail-item">
                    <span class="detail-label">交易金额</span>
                    <span class="detail-value amount">{{ transactionDetail.money }}</span>
                  </div>
                  <div class="detail-item">
                    <span class="detail-label">创建时间</span>
                    <span class="detail-value">{{ transactionDetail.time }}</span>
                  </div>
                </div>
                <div class="detail-row">
                  <div class="detail-item">
                    <span class="detail-label">付款时间</span>
                    <span class="detail-value">{{ transactionDetail.payTime || '待付款' }}</span>
                  </div>
                  <div class="detail-item expand-control" @click="showTransactionDetail(transactionDetail.id)">
                    <span class="expand-text">订单详情</span>
                    <el-icon class="expand-icon" :class="{ 'expanded': isShowTransactionDetail(transactionDetail.id) }">
                      <CaretRight />
                    </el-icon>
                  </div>
                </div>
              </div>

              <!-- 展开的订单详情 -->
              <div v-if="isShowTransactionDetail(transactionDetail.id)" class="order-details-section">
                <div class="orders-table-wrapper">
                  <el-table :data="transactionDetail.orderInfo" class="orders-table">
                    <el-table-column type="expand" width="50">
                      <template #default="props">
                        <div class="order-expand-content">
                          <!-- 火车订票详情 -->
                          <div v-if="props.row.type === '火车订票'" class="order-detail-card train-order">
                            <div class="order-card-header">
                              <h4 class="order-title">火车票信息</h4>
                              <span class="travel-date">{{ getOrderMap(props.row.id, "date") }}</span>
                            </div>
                            <div class="train-info">
                              <div class="station-info">
                                <div class="station departure">
                                  <span class="station-name">{{ getOrderMap(props.row.id, "depatureStation") }}</span>
                                  <span class="station-label">出发站</span>
                                </div>
                                <div class="train-route">
                                  <div class="train-number">{{ getOrderMap(props.row.id, "trainNumber") }}</div>
                                  <div class="route-line"></div>
                                  <div class="departure-time">{{ getOrderMap(props.row.id, "depatureTime") }} 开</div>
                                </div>
                                <div class="station arrival">
                                  <span class="station-name">{{ getOrderMap(props.row.id, "reachStation") }}</span>
                                  <span class="station-label">到达站</span>
                                </div>
                              </div>
                              <div class="passenger-info">
                                <div class="info-item">
                                  <span class="info-label">座位信息</span>
                                  <span class="info-value">{{ getOrderMap(props.row.id, "seatInfo") }}</span>
                                </div>
                                <div class="info-item">
                                  <span class="info-label">乘车人</span>
                                  <span class="info-value">{{ getOrderMap(props.row.id, "name") }}</span>
                                </div>
                              </div>
                            </div>
                          </div>

                          <!-- 酒店预订详情 -->
                          <div v-else-if="props.row.type === '酒店预订'" class="order-detail-card hotel-order">
                            <div class="order-card-header">
                              <h4 class="order-title">酒店信息</h4>
                              <span class="room-count">{{ getOrderMap(props.row.id, "number") }} 间</span>
                            </div>
                            <div class="hotel-info">
                              <div class="hotel-details">
                                <div class="info-item">
                                  <span class="info-label">酒店名称</span>
                                  <span class="info-value hotel-name">{{ getOrderMap(props.row.id, "hotelName") }}</span>
                                </div>
                                <div class="info-item">
                                  <span class="info-label">房型</span>
                                  <span class="info-value">{{ getOrderMap(props.row.id, "roomType") }}</span>
                                </div>
                                <div class="info-item">
                                  <span class="info-label">入住日期</span>
                                  <span class="info-value">{{ getOrderMap(props.row.id, "beginDate") }}</span>
                                </div>
                                <div class="info-item">
                                  <span class="info-label">离店日期</span>
                                  <span class="info-value">{{ getOrderMap(props.row.id, "endDate") }}</span>
                                </div>
                                <div class="info-item">
                                  <span class="info-label">预订人</span>
                                  <span class="info-value">{{ getOrderMap(props.row.id, "name") }}</span>
                                </div>
                              </div>
                            </div>
                          </div>

                          <!-- 火车餐预订详情 -->
                          <div v-else-if="props.row.type === '火车餐预订'" class="order-detail-card food-order">
                            <div class="order-card-header">
                              <h4 class="order-title">火车餐信息</h4>
                              <span class="order-date">{{ getOrderMap(props.row.id, "date") }}</span>
                            </div>
                            <div class="food-info">
                              <div class="food-details">
                                <div class="info-item">
                                  <span class="info-label">商家</span>
                                  <span class="info-value">{{ getOrderMap(props.row.id, "shopName") }}</span>
                                </div>
                                <div class="info-item">
                                  <span class="info-label">餐品</span>
                                  <span class="info-value">{{ getOrderMap(props.row.id, "foodName") }}</span>
                                </div>
                                <div class="info-item">
                                  <span class="info-label">车次</span>
                                  <span class="info-value">{{ getOrderMap(props.row.id, "trainNumber") }}</span>
                                </div>
                                <div class="info-item">
                                  <span class="info-label">预计时间</span>
                                  <span class="info-value">{{ getOrderMap(props.row.id, "time") }}</span>
                                </div>
                                <div class="info-item" v-if="getOrderMap(props.row.id, 'station')">
                                  <span class="info-label">车站</span>
                                  <span class="info-value">{{ getOrderMap(props.row.id, "station") }}站</span>
                                </div>
                                <div class="info-item">
                                  <span class="info-label">订餐人</span>
                                  <span class="info-value">{{ getOrderMap(props.row.id, "name") }}</span>
                                </div>
                              </div>
                            </div>
                          </div>
                        </div>
                      </template>
                    </el-table-column>
                    <el-table-column prop="id" label="订单ID" width="370" />
                    <el-table-column prop="type" label="订单类型" width="180" />
                    <el-table-column label="订单状态" width="170">
                      <template #default="props">
                        <span 
                          class="order-status-badge"
                          :style="{ color: getStatusColor(props.row.status) }"
                        >
                          {{ props.row.status }}
                        </span>
                      </template>
                    </el-table-column>
                    <el-table-column prop="money" label="订单金额" width="170" />
                    <el-table-column fixed="right" label="操作" width="170">
                      <template #default="props">
                        <el-tooltip :content="props.row.reason" :disabled="props.row.canCanceled">
                          <el-button 
                            text 
                            type="danger" 
                            size="small" 
                            :disabled="!props.row.canCanceled"
                            @click="cancelOrder(props.row.id, props.row.canCanceled, props.row.reason)"
                            class="cancel-order-btn"
                          >
                            取消订单
                          </el-button>
                        </el-tooltip>
                      </template>
                    </el-table-column>
                  </el-table>
                </div>
              </div>

              <!-- 支付按钮 -->
              <div v-if="transactionDetail.status === '未支付'" class="payment-section">
                <el-button 
                  type="primary" 
                  class="pay-button"
                  @click="goToPay(transactionDetail.id, transactionDetail.money)"
                >
                  立即支付
                </el-button>
              </div>
            </div>
          </div>

          <!-- 分页组件 -->
          <div class="pagination-section">
            <el-pagination
              v-model:current-page="currentPage"
              v-model:page-size="pageSize"
              :page-sizes="[5, 10, 20, 50]"
              :total="filteredTransactionDetailList.length"
              layout="total, sizes, prev, pager, next, jumper"
              @size-change="handlePageSizeChange"
              @current-change="handleCurrentPageChange"
              class="custom-pagination"
              :disabled="filteredTransactionDetailList.length === 0"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import dayjs from 'dayjs';
import 'dayjs/locale/zh-cn';
import { orderApi } from "@/api/orderApi/orderApi";
import type { ResponseData, TransactionData, SeatLocationInfo, TrainOrderInfo, HotelOrderInfo, DishOrderInfo, TakeawayOrderInfo,
    OrderInform, TransactionDetail, OrderDetail, TrainOrderDetail, HotelOrderDetail, FoodOrderDetail } from '@/interface/interface';
import { ElMessage, ElMessageBox } from 'element-plus';
import { useRouter } from 'vue-router';
import router from '@/router';
import type { UserApiBalanceData } from '@/interface/userInterface';
import { useUserStore } from '@/stores/user';
import { userApi } from '@/api/UserApi/userApi';
import { CaretRight, DocumentRemove } from '@element-plus/icons-vue';

const user = useUserStore();

import locale from 'ant-design-vue/es/date-picker/locale/zh_CN';
import { Dayjs } from 'dayjs';
import 'dayjs/locale/zh-cn';
import type { Moment } from 'moment';
import { ref } from 'vue';
dayjs.locale('zh-cn');

const statusChangeTab = {
    unpaid: "未支付",
    paid: "已支付",
    ongoing: "未出行",
    active: "行程中",
    completed: "已完成",
    failed: "失败",
    cancelled: "已取消",
}

const typeChangeTab = {
    train: "火车订票",
    hotel: "酒店预订",
    dish: "火车餐预订",
    takeaway: "火车餐预订",
}

export default {
    components: {
        CaretRight,
        DocumentRemove,
    },
    data(){
        return {
            selectedTrain: true,
            selectedHotel: true,
            selectedFood: true,
            isPayed: true,
            unPayed: true,
            selectedDate: ref<Moment[]>([]),
            selectedStartDate: "",
            selectedEndDate: "",
            isShowTransactionDetailMap: new Map<string, Boolean>(),
            transactionMap: new Map<string, TransactionDetail>(),
            transactionList: [] as string[],
            orderMap: new Map<string, OrderDetail>(),
            transactionDetailList: [] as TransactionDetail [],
            router: useRouter(),
            // 分页相关数据
            currentPage: 1,
            pageSize: 10,
            locale: locale,
            loading: true, // 添加加载状态
        }
    },
    computed: {
        filteredTransactionDetailList() {
            return this.transactionDetailList.filter(transaction => this.selected(transaction));
        },
        // 分页后的交易列表
        paginatedTransactionList() {
            const start = (this.currentPage - 1) * this.pageSize;
            const end = start + this.pageSize;
            return this.filteredTransactionDetailList.slice(start, end);
        },
        // 总页数
        totalPages() {
            return Math.ceil(this.filteredTransactionDetailList.length / this.pageSize);
        }
    },
    watch: {
        // 监听筛选条件变化，重置到第一页
        selectedTrain() {
            this.resetPagination();
        },
        selectedHotel() {
            this.resetPagination();
        },
        selectedFood() {
            this.resetPagination();
        },
        isPayed() {
            this.resetPagination();
        },
        unPayed() {
            this.resetPagination();
        },
        selectedDate() {
            this.resetPagination();
        }
    },
    created: function() {
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
            let selectedTime = ((transactionDetail.time >= this.selectedStartDate) && (transactionDetail.time <= this.selectedEndDate)) || !this.selectedDate || this.selectedDate.length === 0;
            return selectedPay && selectedType  && selectedTime;
        },
        handleDateChange(dateRange: any) {
            if(dateRange){
                const [startDate, endDate] = this.selectedDate;
                this.selectedStartDate = dayjs(startDate.toDate()).format('YYYY-MM-DD HH:mm');
                this.selectedEndDate = dayjs(endDate.toDate()).format('YYYY-MM-DD HH:mm');
            } else {
                this.selectedDate = [];
            }
        },

        getOrderMap(id: string, name: string): any {
            let order = this.orderMap.get(id);
            if(order && name in order){
                return (order as any)[name];
            }
        },

        getStatusColor(status: string) {
            if(status == '行程中') {
                return '#10b981';
            } else if (status == '未支付') {
                return '#ef4444';
            } else if (status == '已支付') {
                return '#10b981';
            } else if (status == '已完成') {
                return '#6b7280';
            } else if (status == '已取消') {
                return '#f59e0b';
            }
            return '#6b7280';
        },

        cancelOrder(id: string, canCancel: boolean, reason: string) {
            if(canCancel){
                ElMessageBox.confirm(
                    '确认取消该订单？',
                    '警告！',
                    {
                        confirmButtonText: '确认',
                        cancelButtonText: '取消',
                        type: 'warning',
                    }
                ).then(() => {
                    this.apiOrderCancel(id);
                })
            } else {
                ElMessage.error('不可取消该订单 ' + reason);
            }
        },

        async apiOrderCancel(id: string) {
            await orderApi.orderCancel(id)
            .then((res) => {
                if(res.status == 200) {
                    if(res.data.code == 200) {
                        this.cancelOrderSuccess();
                    }  else if(res.data.code == 403) {
                        ElMessage.error('会话无效');
                    } else if(res.data.code == 404) {
                        ElMessage.error('订单号不存在，或没有权限访问该订单');
                    } else if(res.data.code == 14001) {
                        ElMessage.error('订单已被取消');
                    } else if(res.data.code == 14002) {
                        ElMessage.error('订单不满足取消条件');
                    }
                }
            }).catch((error) => {
                ElMessage.error(error);
            })
        },

        cancelOrderSuccess(){
            ElMessage.success('成功取消该订单');
            this.setBalance();
            this.refresh();
        },

        async setBalance() {
            try {
                const balRes: UserApiBalanceData = (await userApi.queryUserBalance()).data;
                if(balRes.code === 200) {
                    const balance: number = balRes.data.balance;
                    user.setUserBalance(balance);
                }
                else
                    throw new Error('invalid session id');
            } catch(e: any) {
                console.log(e);
            }
        },

        refresh() {
            this.transactionMap.clear();
            this.transactionList.length = 0;
            this.orderMap.clear();
            this.transactionDetailList.length = 0;
            this.resetPagination(); // 重置分页
            this.init();
            for(let tr of this.transactionDetailList) {
                if(!this.isShowTransactionDetailMap.has(tr.id)) {
                    this.isShowTransactionDetailMap.set(tr.id, false);
                }
            }
        },

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
                if (transactionDetail) {
                    this.transactionDetailList.push(transactionDetail);
                }
            }
        },

        async init() {
            try {
                this.loading = true; // 开始加载
                await orderApi.orderList()
                .then((res) => {
                    if(res.status == 200){
                        if(res.data.code == 200) {
                            const resData: ResponseData = res.data.data;
                            this.dataHandle(resData);
                        } else {
                            throw new Error(res.statusText);
                        }
                    }
                })
                .catch((error) => {
                    console.error(error);
                })
            } finally {
                this.loading = false; // 结束加载
            }
        },

        dataHandle(resData: ResponseData): void{
            for(const transactionData of resData){
                let transactionDetail: TransactionDetail = {
                    id: transactionData.transactionId,
                    status: statusChangeTab[transactionData.status],
                    time: dayjs(transactionData.createTime).format("YYYY-MM-DD HH:mm:ss"),
                    payTime: transactionData.payTime ? dayjs(transactionData.payTime).format("YYYY-MM-DD HH:mm:ss") : undefined,
                    money: 'SC ' + String(transactionData.amount),
                    orderInfo: [] as OrderInform [],
                };
                for(const orderInfo of transactionData.orders){
                    let orderInform: OrderInform = {
                        id: orderInfo.orderId,
                        status: statusChangeTab[orderInfo.status],
                        type: "",
                        money: 'SC ' + String(orderInfo.unitPrice),
                        canCanceled: orderInfo.canCancel,
                        reason: orderInfo.reason,
                    };
                    switch(orderInfo.orderType){
                        case "train":
                            const trainOrderInfo = orderInfo as TrainOrderInfo;
                            trainOrderInfo.departureTime = dayjs(trainOrderInfo.departureTime).format("YYYY-MM-DD HH:mm")
                            let trainOrderDetail: TrainOrderDetail = {
                                id: trainOrderInfo.orderId,
                                name: trainOrderInfo.name,
                                depatureStation: trainOrderInfo.departureStation,
                                reachStation: trainOrderInfo.arrivalStation,
                                trainNumber: trainOrderInfo.trainNumber,
                                date: trainOrderInfo.departureTime.substring(0, 10),
                                depatureTime: trainOrderInfo.departureTime.substring(11),
                                seatInfo: trainOrderInfo.seat ? this.getSeat(trainOrderInfo.seat) : '待分配座位'
                            };
                            this.setOrderMap(trainOrderDetail);
                            break;
                        case "dish":
                            const dishOrderInfo = orderInfo as DishOrderInfo;
                            dishOrderInfo.depatureTime = dayjs(dishOrderInfo.depatureTime).format("YYYY-MM-DD HH:mm:ss")
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
                                beginDate: dayjs(hotelOrderInfo.beginDate).format("YYYY-MM-DD"),
                                endDate: dayjs(hotelOrderInfo.endDate).format("YYYY-MM-DD"),
                                name: hotelOrderInfo.name,
                                number: hotelOrderInfo.amount,
                            }
                            this.setOrderMap(hotelOrderDetail);
                            break;
                        case "takeaway":
                            const takeawayOrderInfo = orderInfo as TakeawayOrderInfo;
                            takeawayOrderInfo.depatureTime = dayjs(takeawayOrderInfo.depatureTime).format("YYYY-MM-DD HH:mm:ss");
                            takeawayOrderInfo.dishTime = dayjs(takeawayOrderInfo.dishTime).format("YYYY-MM-DD HH:mm");
                            let takeawayOrederDetail: FoodOrderDetail = {
                                id: takeawayOrderInfo.orderId,
                                shopName: takeawayOrderInfo.shopName,
                                foodName: takeawayOrderInfo.takeawayName,
                                trainNumber: takeawayOrderInfo.trainNumber,
                                station: takeawayOrderInfo.station,
                                date: takeawayOrderInfo.depatureTime.substring(0,10),
                                time: takeawayOrderInfo.dishTime.substring(11),
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

        goToPay(transactionId: string, money: string) {
            router.push({
                name: 'paypage',
                params: { transactionId: transactionId },
                query: {
                    money: money,
                }
            });
        },

        // 分页相关方法
        handlePageSizeChange(newPageSize: number) {
            this.pageSize = newPageSize;
            this.currentPage = 1; // 重置到第一页
        },

        handleCurrentPageChange(newCurrentPage: number) {
            this.currentPage = newCurrentPage;
            // 滚动到顶部
            this.scrollToTop();
        },

        resetPagination() {
            this.currentPage = 1;
        },

        scrollToTop() {
            // 滚动到交易内容顶部
            this.$nextTick(() => {
                const transactionContent = document.querySelector('.transactions-content');
                if (transactionContent) {
                    transactionContent.scrollIntoView({ 
                        behavior: 'smooth', 
                        block: 'start' 
                    });
                }
            });
        },
    }
}
</script>

<style scoped>
/* 容器样式 */
.transaction-container {
  min-height: 50vh;
  padding: 16px;
  display: flex;
  justify-content: center;
}

/* 主卡片 */
.transaction-card {
  width: 100%;
  max-width: 1200px;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border-radius: 12px;
  box-shadow: 
    0 6px 20px rgba(0, 0, 0, 0.06),
    0 0 0 1px rgba(255, 255, 255, 0.2);
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* 卡片头部 - 增大字体 */
.card-header {
  padding: 20px 28px 14px;
  background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
}

.page-title {
  font-size: 26px;
  font-weight: 700;
  color: #1a202c;
  margin: 0 0 4px 0;
  letter-spacing: -0.2px;
  line-height: 1.2;
}

.page-subtitle {
  font-size: 16px;
  color: #64748b;
  margin: 0;
  font-weight: 400;
  line-height: 1.3;
}

/* 头部统计信息 - 增大字体 */
.stats-text {
  font-size: 14px;
  color: #64748b;
  font-weight: 500;
  padding: 6px 12px;
  background: rgba(255, 255, 255, 0.8);
  border-radius: 6px;
  border: 1px solid rgba(226, 232, 240, 0.5);
  line-height: 1.2;
}

/* 筛选器区域 - 增大字体 */
.filter-section {
  padding: 16px 28px;
  background: rgba(248, 250, 252, 0.5);
  border-bottom: 1px solid rgba(226, 232, 240, 0.5);
}

.filter-label {
  font-size: 15px;
  font-weight: 600;
  color: #374151;
  margin-bottom: 4px;
  line-height: 1.2;
}

.checkbox-group :deep(.el-checkbox__label) {
  font-size: 14px;
  color: #4b5563;
  font-weight: 500;
  line-height: 1.3;
  padding-left: 8px;
}

.date-picker :deep(.el-input__wrapper) {
  height: 36px;
}

.date-picker :deep(.el-input__inner) {
  font-size: 14px;
  line-height: 1.3;
}

/* 交易列表内容 - 增大间距 */
.transactions-content {
  padding: 20px 28px;
}

/* 空状态 - 增大字体 */
.empty-title {
  font-size: 22px;
  font-weight: 600;
  color: #374151;
  margin: 0 0 8px 0;
  line-height: 1.3;
}

.empty-subtitle {
  font-size: 16px;
  color: #6b7280;
  margin: 0;
  line-height: 1.4;
}

/* 交易卡片项 - 增大间距 */
.transactions-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
  position: relative;
}

/* 交易头部 - 增大字体 */
.transaction-header {
  padding: 16px 20px 12px;
  border-bottom: 1px solid #f1f5f9;
}

.id-label {
  font-size: 12px;
  color: #6b7280;
  font-weight: 500;
  line-height: 1.2;
}

.id-value {
  font-size: 16px;
  color: #1a202c;
  font-weight: 600;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  line-height: 1.2;
}

.status-badge {
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.3px;
  line-height: 1.2;
}

/* 交易详情 - 增大字体 */
.transaction-details {
  padding: 12px 20px;
}

.detail-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  margin-bottom: 10px;
}

.detail-label {
  font-size: 13px;
  color: #6b7280;
  font-weight: 500;
  line-height: 1.2;
}

.detail-value {
  font-size: 14px;
  color: #374151;
  font-weight: 500;
  line-height: 1.2;
}

.detail-value.amount {
  font-size: 18px;
  font-weight: 700;
  color: #667eea;
}

.expand-control {
  padding: 4px 10px;
  border-radius: 6px;
}

.expand-text {
  color: #667eea;
  font-weight: 600;
  font-size: 13px;
  line-height: 1.2;
}

.expand-icon {
  font-size: 16px;
}

/* 订单详情区域 - 增大字体 */
.orders-table-wrapper {
  padding: 16px;
}

.orders-table :deep(.el-table th) {
  background: #f8fafc;
  color: #374151;
  font-weight: 600;
  border-bottom: 1px solid #e5e7eb;
  padding: 10px 0;
  font-size: 14px;
  line-height: 1.2;
}

.orders-table :deep(.el-table td) {
  border-bottom: 1px solid #f1f5f9;
  padding: 10px 0;
  font-size: 13px;
  line-height: 1.2;
}

.order-status-badge {
  font-weight: 600;
  font-size: 12px;
  line-height: 1.2;
}

.cancel-order-btn {
  font-size: 12px;
  padding: 4px 8px;
  line-height: 1.2;
}

/* 订单展开内容 - 增大字体 */
.order-expand-content {
  padding: 12px;
  background: #fff;
  border-radius: 6px;
  margin: 8px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.04);
}

.order-card-header {
  padding: 12px 16px;
  background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
  border-bottom: 1px solid #e2e8f0;
}

.order-title {
  font-size: 16px;
  font-weight: 600;
  color: #1a202c;
  line-height: 1.2;
}

.travel-date,
.room-count,
.order-date {
  font-size: 13px;
  color: #6b7280;
  font-weight: 500;
  line-height: 1.2;
}

/* 火车订单样式 - 增大字体 */
.train-info {
  padding: 12px;
}

.station-name {
  font-size: 16px;
  font-weight: 700;
  color: #1a202c;
  margin-bottom: 2px;
  line-height: 1.2;
}

.station-label {
  font-size: 12px;
  color: #6b7280;
  font-weight: 500;
  line-height: 1.2;
}

.train-number {
  padding: 3px 8px;
  background: #3b82f6;
  color: white;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  line-height: 1.2;
}

.departure-time {
  font-size: 12px;
  color: #6b7280;
  font-weight: 500;
  line-height: 1.2;
}

/* 酒店和火车餐订单 - 增大字体 */
.info-label {
  font-size: 12px;
  color: #6b7280;
  font-weight: 500;
  line-height: 1.2;
}

.info-value {
  font-size: 14px;
  color: #374151;
  font-weight: 600;
  line-height: 1.2;
}

/* 支付区域 - 增大字体 */
.payment-section {
  padding: 12px 20px 16px;
  border-top: 1px solid #f1f5f9;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.05) 0%, rgba(118, 75, 162, 0.05) 100%);
  display: flex;
  justify-content: flex-end;
}

.pay-button {
  padding: 8px 20px;
  border-radius: 6px;
  font-weight: 600;
  font-size: 14px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  box-shadow: 0 2px 6px rgba(102, 126, 234, 0.2);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  line-height: 1.2;
}

/* 分页区域 - 增大字体 */
.pagination-section {
  margin-top: 20px;
  padding: 16px 0;
  border-top: 1px solid rgba(226, 232, 240, 0.3);
  display: flex;
  justify-content: center;
}

.custom-pagination {
  background: rgba(248, 250, 252, 0.5);
  padding: 12px 20px;
  border-radius: 8px;
  border: 1px solid rgba(226, 232, 240, 0.3);
}

.custom-pagination :deep(.el-pagination__total) {
  color: #64748b;
  font-weight: 500;
  font-size: 14px;
  line-height: 1.2;
}

.custom-pagination :deep(.el-select .el-input__wrapper) {
  height: 32px;
}

.custom-pagination :deep(.el-select .el-input__inner) {
  font-size: 13px;
  line-height: 1.2;
}

.custom-pagination :deep(.btn-prev),
.custom-pagination :deep(.btn-next) {
  height: 32px;
  width: 32px;
  font-size: 14px;
}

.custom-pagination :deep(.el-pager li) {
  height: 32px;
  min-width: 32px;
  font-size: 13px;
  line-height: 1.2;
}

.custom-pagination :deep(.el-pagination__jump) {
  color: #64748b;
  font-size: 13px;
  line-height: 1.2;
}

.custom-pagination :deep(.el-input__wrapper) {
  height: 32px;
}

.custom-pagination :deep(.el-input__inner) {
  font-size: 13px;
  line-height: 1.2;
}

/* 骨架屏样式 */
.loading-state {
  padding: 20px 0;
}

.skeleton-transactions {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.skeleton-transaction-card {
  background: #fff;
  border-radius: 8px;
  border: 1px solid #f1f5f9;
  overflow: hidden;
  animation: fadeInUp 0.6s cubic-bezier(0.4, 0, 0.2, 1) forwards;
  opacity: 0;
}

.skeleton-header {
  padding: 16px 20px 12px;
  border-bottom: 1px solid #f1f5f9;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.skeleton-transaction-info {
  flex: 1;
}

.skeleton-id {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.skeleton-status {
  display: flex;
  align-items: center;
}

.skeleton-badge {
  width: 60px;
  height: 24px;
  border-radius: 6px;
  background: linear-gradient(90deg, #f1f5f9, #e2e8f0, #f1f5f9);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

.skeleton-details {
  padding: 12px 20px;
}

.skeleton-detail-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  margin-bottom: 10px;
}

.skeleton-detail-row:last-child {
  margin-bottom: 0;
}

.skeleton-detail-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.skeleton-text {
  background: linear-gradient(90deg, #f1f5f9, #e2e8f0, #f1f5f9);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: 4px;
}

.skeleton-id-label {
  width: 60px;
  height: 12px;
}

.skeleton-id-value {
  width: 120px;
  height: 16px;
}

.skeleton-detail-label {
  width: 50px;
  height: 12px;
}

.skeleton-detail-value {
  width: 80px;
  height: 14px;
}

.skeleton-detail-value.skeleton-amount {
  width: 100px;
  height: 18px;
}

/* 骨架屏动画 */
@keyframes shimmer {
  0% {
    background-position: -200% 0;
  }
  100% {
    background-position: 200% 0;
  }
}

/* 响应式骨架屏 */
@media (max-width: 768px) {
  .skeleton-detail-row {
    grid-template-columns: 1fr;
    gap: 8px;
  }
  
  .skeleton-header {
    padding: 12px 16px;
  }
  
  .skeleton-details {
    padding: 8px 16px;
  }
}

/* 保持其他样式不变 */
.transaction-card:hover {
  transform: translateY(-1px);
  box-shadow: 
    0 8px 24px rgba(0, 0, 0, 0.08),
    0 0 0 1px rgba(255, 255, 255, 0.3);
}

.header-content {
  flex: 1;
}

.stats-info {
  display: flex;
  align-items: center;
}

.filter-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
  align-items: start;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.checkbox-group {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.checkbox-group :deep(.el-checkbox) {
  margin-right: 0;
  height: auto;
}

.checkbox-group :deep(.el-checkbox__input) {
  line-height: 1;
}

.date-picker {
  width: 100%;
  max-width: 280px;
}

.date-picker :deep(.el-input__wrapper) {
  border-radius: 6px;
  transition: all 0.2s;
}

.date-picker :deep(.el-input__wrapper:hover) {
  border-color: #667eea;
}

.divider {
  height: 1px;
  background: linear-gradient(90deg, transparent, #e2e8f0, transparent);
  margin: 0 28px;
}

.empty-state {
  text-align: center;
  padding: 40px 24px;
}

.empty-icon {
  color: #d1d5db;
  margin-bottom: 16px;
}

.transaction-card-item {
  background: #fff;
  border-radius: 8px;
  border: 1px solid #f1f5f9;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  animation: fadeInUp 0.6s cubic-bezier(0.4, 0, 0.2, 1) forwards;
  opacity: 0;
}

.transaction-card-item::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  transform: scaleX(0);
  transform-origin: left;
  transition: transform 0.3s;
}

.transaction-card-item:hover {
  border-color: #e2e8f0;
  transform: translateY(-1px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
}

.transaction-card-item:hover::before {
  transform: scaleX(1);
}

.transaction-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.transaction-id {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.transaction-status {
  display: flex;
  align-items: center;
}

.status-badge.status-paid {
  background: #d1fae5;
  color: #047857;
  border: 1px solid #10b981;
}

.status-badge.status-unpaid {
  background: #fee2e2;
  color: #b91c1c;
  border: 1px solid #f87171;
}

.detail-row:last-child {
  margin-bottom: 0;
}

.detail-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 3px 0;
}

.expand-control {
  cursor: pointer;
  transition: all 0.2s;
}

.expand-control:hover {
  background: rgba(102, 126, 234, 0.05);
}

.expand-icon {
  transition: transform 0.3s;
  color: #667eea;
}

.expand-icon.expanded {
  transform: rotate(90deg);
}

.order-details-section {
  border-top: 1px solid #f1f5f9;
  background: #f8fafc;
}

.orders-table :deep(.el-table) {
  border-radius: 6px;
  overflow: hidden;
}

.orders-table :deep(.el-table__header-wrapper) {
  background: #f1f5f9;
}

.order-detail-card {
  border-radius: 8px;
  overflow: hidden;
}

.order-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.train-order .order-card-header {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.1) 0%, rgba(147, 51, 234, 0.1) 100%);
}

.station-info {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  gap: 16px;
  align-items: center;
  margin-bottom: 12px;
}

.station {
  text-align: center;
}

.station-name {
  display: block;
}

.train-route {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  position: relative;
}

.route-line {
  width: 60px;
  height: 2px;
  background: linear-gradient(90deg, #3b82f6, #8b5cf6);
  border-radius: 1px;
  position: relative;
}

.passenger-info {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.hotel-order .order-card-header {
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.1) 0%, rgba(5, 150, 105, 0.1) 100%);
}

.hotel-info {
  padding: 12px;
}

.hotel-details {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: 12px;
}

.food-order .order-card-header {
  background: linear-gradient(135deg, rgba(245, 158, 11, 0.1) 0%, rgba(217, 119, 6, 0.1) 100%);
}

.food-info {
  padding: 12px;
}

.food-details {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 12px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.info-value.hotel-name {
  color: #059669;
}

.pay-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 3px 8px rgba(102, 126, 234, 0.25);
}

.custom-pagination :deep(.btn-prev:hover),
.custom-pagination :deep(.btn-next:hover) {
  background: #667eea;
  border-color: #667eea;
  color: #fff;
}

.custom-pagination :deep(.el-pager li:hover) {
  background: rgba(102, 126, 234, 0.1);
  border-color: #667eea;
  color: #667eea;
}

.custom-pagination :deep(.el-pager li.is-active) {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-color: #667eea;
  color: #fff;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.transaction-card-item:nth-child(1),
.skeleton-transaction-card:nth-child(1) { animation-delay: 0.05s; }
.transaction-card-item:nth-child(2),
.skeleton-transaction-card:nth-child(2) { animation-delay: 0.1s; }
.transaction-card-item:nth-child(3),
.skeleton-transaction-card:nth-child(3) { animation-delay: 0.15s; }
.transaction-card-item:nth-child(4),
.skeleton-transaction-card:nth-child(4) { animation-delay: 0.2s; }
.transaction-card-item:nth-child(5),
.skeleton-transaction-card:nth-child(5) { animation-delay: 0.25s; }

/* 响应式设计 */
@media (max-width: 768px) {
  .transaction-container {
    padding: 12px;
  }

  .card-header {
    padding: 16px 20px 12px;
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .page-title {
    font-size: 22px;
  }

  .page-subtitle {
    font-size: 14px;
  }

  .stats-info {
    align-self: flex-end;
  }

  .filter-section {
    padding: 12px 20px;
  }

  .filter-grid {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .transactions-content {
    padding: 16px 20px;
  }

  .detail-row {
    grid-template-columns: 1fr;
    gap: 8px;
  }

  .station-info {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .passenger-info,
  .hotel-details,
  .food-details {
    grid-template-columns: 1fr;
  }

  .orders-table-wrapper {
    padding: 12px;
    overflow-x: auto;
  }

  .pagination-section {
    margin-top: 16px;
    padding: 12px 0;
  }

  .custom-pagination {
    padding: 8px 12px;
    margin: 0 -12px;
    border-radius: 0;
    border-left: none;
    border-right: none;
  }
}
</style>

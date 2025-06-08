<template>
    <!-- 只在有火车票订单时显示 -->
    <el-card v-if="hasTrainOrder" class="TravelInfoCard" shadow="never">
        <div class="TravelInfo">
            <div class="TravelDate">
                <p>{{ formatDate(orderDetails.date) }}</p>
            </div>
            <div class="TravelStatus">
                <el-tag :type="getStatusType(orderDetails.status)" size="large" round>
                    {{ orderDetails.status }}
                </el-tag>
            </div>
        </div>
        <div class="TicketInfo">
            <div class="RouteSection">
                <div class="Departure">
                    <div class="Time">
                        <span>{{ orderDetails.departureTime }}</span>
                    </div>
                    <div class="City">
                        <span>{{ orderDetails.departureStation }}</span>
                    </div>
                </div>
                <div class="Arrow">
                    <div class="TrainInfo">
                        <span class="TrainNumber">{{ orderDetails.trainNumber }}</span>
                        <span class="Duration">{{ orderDetails.duration }}</span>
                    </div>
                    <div class="ArrowLine">
                        <div class="Line"></div>
                        <svg class="ArrowIcon" viewBox="0 0 24 24" fill="none">
                            <path d="M5 12h14m-7-7l7 7-7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                        </svg>
                    </div>
                </div>
                <div class="Arrival">
                    <div class="Time">
                        <span>{{ orderDetails.arrivalTime }}</span>
                    </div>
                    <div class="City">
                        <span>{{ orderDetails.arrivalStation }}</span>
                    </div>
                </div>
            </div>
            <div class="SeatInfo">
                <div class="SeatDetails">
                    <div class="SeatNumberGroup">
                        <span class="CarNumber">{{ orderDetails.carNumber }}</span>
                        <span class="SeatNumber">{{ orderDetails.seatNumber }}</span>
                    </div>
                    <div class="SeatType">
                        <el-tag type="info" size="large">{{ orderDetails.seatType }}</el-tag>
                    </div>
                </div>
            </div>
        </div>
    </el-card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import dayjs from 'dayjs';
import { orderApi } from "@/api/orderApi/orderApi";
import type { 
  ResponseData, 
  TrainOrderInfo, 
  SeatLocationInfo 
} from '@/interface/interface';
import { ElMessage } from 'element-plus';
import { useUserStore } from '@/stores/user';

// 状态映射
const statusChangeTab = {
  unpaid: "未支付",
  paid: "已支付", 
  ongoing: "未出行",
  active: "行程中",
  completed: "已完成",
  failed: "失败",
  cancelled: "已取消",
};

// 响应式数据
const orderList = ref<TrainOrderInfo[]>([]);
const loading = ref(false);

const user = useUserStore();

// 订单详情接口
interface OrderDetails {
  id: string;
  status: string;
  trainNumber: string;
  departureStation: string;
  arrivalStation: string;
  departureTime: string;
  arrivalTime: string;
  date: string;
  duration: string;
  carNumber: string;
  seatNumber: string;
  seatType: string;
  name: string;
}

// 计算属性
const hasTrainOrder = computed(() => {
  return orderList.value.length > 0;
});

// 获取最近的火车票订单详情
const orderDetails = computed((): OrderDetails => {
  if (orderList.value.length === 0) {
    return {} as OrderDetails;
  }

  const nearestOrder = orderList.value[0];
  const departureTime = dayjs(nearestOrder.departureTime);
  const arrivalTime = dayjs(nearestOrder.arrivalTime);
  
  // 计算行程时长
  const duration = calculateDuration(nearestOrder.departureTime, nearestOrder.arrivalTime);
  
  // 格式化座位信息
  const seatInfo = formatSeatInfo(nearestOrder.seat);

  return {
    id: nearestOrder.orderId,
    status: statusChangeTab[nearestOrder.status],
    trainNumber: nearestOrder.trainNumber,
    departureStation: nearestOrder.departureStation,
    arrivalStation: nearestOrder.arrivalStation,
    departureTime: departureTime.format('HH:mm'),
    arrivalTime: arrivalTime.format('HH:mm'),
    date: departureTime.format('YYYY-MM-DD'),
    duration: duration,
    carNumber: seatInfo.carNumber,
    seatNumber: seatInfo.seatNumber,
    seatType: seatInfo.seatType,
    name: nearestOrder.name
  };
});

// 方法
const formatDate = (dateStr: string): string => {
  return dayjs(dateStr).format('YYYY年MM月DD日');
};

const getStatusType = (status: string): string => {
  const statusTypeMap: { [key: string]: string } = {
    '未支付': 'warning',
    '已支付': 'info',
    '未出行': 'success',
    '行程中': 'primary',
    '已完成': 'success',
    '失败': 'danger',
    '已取消': 'info'
  };
  return statusTypeMap[status] || 'info';
};

const calculateDuration = (departureTime: string, arrivalTime: string): string => {
  const departure = dayjs(departureTime);
  const arrival = dayjs(arrivalTime);
  const diffMinutes = arrival.diff(departure, 'minute');
  
  const hours = Math.floor(diffMinutes / 60);
  const minutes = diffMinutes % 60;
  
  if (hours > 0) {
    return `${hours} 小时 ${minutes} 分钟`;
  }
  return `${minutes} 分钟`;
};

const formatSeatInfo = (seat: SeatLocationInfo) => {
  const carNumber = seat.carriage < 10 ? `0${seat.carriage}车` : `${seat.carriage}车`;
  const seatNumber = `${seat.row}${seat.location}`;
  const seatType = seat.type;
  
  return {
    carNumber,
    seatNumber,
    seatType
  };
};

const initOrderList = async () => {
  try {
    loading.value = true;
    const res = await orderApi.orderList();
    
    if (res.status === 200 && res.data.code === 200) {
      const resData: ResponseData = res.data.data;
      processTrainOrders(resData);
    } else {
      throw new Error(res.statusText);
    }
  } catch (error) {
    console.error('获取订单列表失败:', error);
    ElMessage.error('获取行程信息失败');
  } finally {
    loading.value = false;
  }
};

const processTrainOrders = (resData: ResponseData) => {
  const trainOrders: TrainOrderInfo[] = [];
  const now = dayjs();
  
  for (const transactionData of resData) {
    for (const orderInfo of transactionData.orders) {
      // 只处理火车票订单，且状态不是未支付或已取消
      if (orderInfo.orderType === 'train' && 
          orderInfo.status !== 'unpaid' && 
          orderInfo.status !== 'cancelled') {
        
        const trainOrder = orderInfo as TrainOrderInfo;
        
        // 只添加属于当前用户的订单
        if (trainOrder.name === user.name) {
          const departureTime = dayjs(trainOrder.departureTime);
          
          // 只显示未来的行程或当天的行程
          if (departureTime.isAfter(now) || departureTime.isSame(now, 'day')) {
            trainOrders.push(trainOrder);
          }
        }
      }
    }
  }

  // 按出发时间排序，最近的在前
  trainOrders.sort((a, b) => {
    return dayjs(a.departureTime).valueOf() - dayjs(b.departureTime).valueOf();
  });

  orderList.value = trainOrders;
};

// 生命周期
onMounted(() => {
  initOrderList();
});
</script>

<style lang="scss" scoped>
.TravelInfoCard {
    width: 900px;
    margin: 30px auto;
    border-radius: 20px;
    height: auto;
    min-height: 220px;
    background: linear-gradient(135deg, #f8fafc 0%, #ffffff 100%);
    border: 1px solid rgba(0, 0, 0, 0.06);
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
    transition: all 0.3s ease;
    
    &:hover {
        transform: translateY(-2px);
        box-shadow: 0 15px 35px rgba(0, 0, 0, 0.15);
    }
}

:deep(.el-card__body) {
    width: 100%;
    height: 100%;
    padding: 35px 45px;
}

.TravelInfo {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 25px;
    padding-bottom: 20px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.08);
}

.TravelDate {
    font-size: 18px;
    font-weight: 600;
    color: #2c3e50;
    
    p {
        margin: 0;
        letter-spacing: 0.5px;
    }
}

.TravelStatus {
    :deep(.el-tag) {
        font-weight: 500;
        padding: 8px 16px;
        font-size: 14px;
    }
}

.TicketInfo {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.RouteSection {
    display: flex;
    align-items: center;
    flex: 1;
}

.Departure, .Arrival {
    display: flex;
    flex-direction: column;
    align-items: center;
    
    .Time {
        font-size: 2.5rem;
        font-weight: 700;
        color: #2c3e50;
        margin-bottom: 8px;
        
        span {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }
    }
    
    .City {
        font-size: 1.3rem;
        color: #7f8c8d;
        font-weight: 500;
    }
}

.Arrow {
    display: flex;
    flex-direction: column;
    align-items: center;
    margin: 0 60px;
    
    .TrainInfo {
        display: flex;
        flex-direction: column;
        align-items: center;
        margin-bottom: 15px;
        
        .TrainNumber {
            font-size: 1.4rem;
            font-weight: 700;
            color: #3498db;
            margin-bottom: 5px;
        }
        
        .Duration {
            font-size: 0.9rem;
            color: #95a5a6;
            font-weight: 500;
        }
    }
    
    .ArrowLine {
        display: flex;
        align-items: center;
        width: 180px;
        
        .Line {
            flex: 1;
            height: 2px;
            background: linear-gradient(90deg, #3498db, #2ecc71);
            border-radius: 1px;
        }
        
        .ArrowIcon {
            width: 24px;
            height: 24px;
            color: #2ecc71;
            margin-left: 8px;
        }
    }
}

.SeatInfo {
    display: flex;
    align-items: center;
    
    .SeatDetails {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
        
        .SeatNumberGroup {
            display: flex;
            align-items: center;
            gap: 15px;
            
            .CarNumber {
                font-size: 1.8rem;
                font-weight: 700;
                color: #e74c3c;
                background: linear-gradient(135deg, #ff6b6b, #ee5a52);
                -webkit-background-clip: text;
                -webkit-text-fill-color: transparent;
                background-clip: text;
            }
            
            .SeatNumber {
                font-size: 1.8rem;
                font-weight: 700;
                color: #e74c3c;
                background: linear-gradient(135deg, #ff6b6b, #ee5a52);
                -webkit-background-clip: text;
                -webkit-text-fill-color: transparent;
                background-clip: text;
            }
        }
        
        .SeatType {
            :deep(.el-tag) {
                font-weight: 500;
                padding: 6px 12px;
            }
        }
    }
}

// 响应式设计
@media (max-width: 768px) {
    .TravelInfoCard {
        width: 95%;
        margin: 20px auto;
    }
    
    .Arrow {
        margin: 0 30px;
        
        .ArrowLine {
            width: 120px;
        }
    }
    
    .Departure .Time, .Arrival .Time {
        font-size: 2rem;
    }
}
</style>
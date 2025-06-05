<template>
    <el-card class="TravelInfoCard" shadow="never">
        <div class="TravelInfo">
            <div class="TravelDate">
                <p>2023年10月1日</p>
            </div>
            <div class="TravelStatus">
                <el-tag type="success" size="large" round>待出发</el-tag>
            </div>
        </div>
        <div class="TicketInfo">
            <div class="RouteSection">
                <div class="Departure">
                    <div class="Time">
                        <span>09:12</span>
                    </div>
                    <div class="City">
                        <span>北京</span>
                    </div>
                </div>
                <div class="Arrow">
                    <div class="TrainInfo">
                        <span class="TrainNumber">G1</span>
                        <span class="Duration">1 小时 20 分钟</span>
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
                        <span>11:30</span>
                    </div>
                    <div class="City">
                        <span>上海</span>
                    </div>
                </div>
            </div>
            <div class="SeatInfo">
                <div class="SeatDetails">
                    <div class="SeatNumberGroup">
                        <span class="CarNumber">07车</span>
                        <span class="SeatNumber">12C</span>
                    </div>
                    <div class="SeatType">
                        <el-tag type="info" size="large">二等座</el-tag>
                    </div>
                </div>
            </div>
        </div>
    </el-card>
</template>

<script setup lang="ts">

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
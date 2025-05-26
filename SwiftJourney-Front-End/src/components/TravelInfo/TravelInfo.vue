<template>
    <el-card class="TravelInfoCard" shadow="always">
        <div class="TravelInfo">
            <div class="TravelDate">
                <p>2023年10月1日</p>
            </div>
            <div class="TravelStatus">
                <p>待出发</p>
            </div>
        </div>
        <div class="TicketInfo">
            <div class="Departure">
                <div class="Time">
                    <span>09:12</span>
                </div>
                <div class="City">
                    <span>北京</span>
                </div>
            </div>
            <div class="Arrow">
                <span>G1</span>
                <img src="@/assets/TicketArrow.svg" />
                <span>1 小时 20 分钟</span>
            </div>
            <div class="Arrival">
                <div class="Time">
                    <span>11:30</span>
                </div>
                <div class="City">
                    <span>上海</span>
                </div>
            </div>
            <div class="SeatInfo">
                <div class="SeatNumber">
                    <p>07</p>
                </div>
                <p div="TrainNumber">
                    车
                </p>
                <div class="SeatNumber SeatIndex">
                    <p>12C</p>
                </div>
                <div class="SeatStatus">
                    <span> 二等座 </span>
                </div>
            </div>
        </div>
    </el-card>
</template>

<script setup lang="ts">

</script>

<style lang="css" scoped>
.TravelInfoCard {
    width: 700px;
    margin-top: 30px;
    border-radius: 15px;
    height: 200px;
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

.TravelDate {
    text-align: left;
    font-size: 20px;
    margin-left: 10px;
    p {
        margin-bottom: 0;
    }
}

.TravelStatus {
    text-align: right;
    font-size: 20px;
    margin-right: 5px;
    color: green;
}

.TicketInfo {
    display: flex;
    margin-left: 40px;
    align-items: center; /* 垂直居中 */
    .Time {
        font-size: 2rem;
    }
    .City {
        text-align: center;
        font-size: 1.25rem;
    }
    .Arrow {
        display: flex;
        flex-direction: column;
        span {
            text-align: center;
            font-size: 1.25rem;
            font-weight: bold;
            padding: 0;
            height: 25px;
            color: rgb(142, 143, 146);
        }
        img {
            width: 200px;
            height: auto;
            margin-left: 20px;
            margin-right: 20px;
        }
    }
}

.TicketInfo .Time span {
    font-weight: bold;
}

.SeatInfo {
    display: flex;
    flex-wrap: wrap;
    margin-left: 70px;
    font-size: 2rem;
    .SeatNumber {
        margin-bottom: 0;
        padding: 0;
    }
    .SeatIndex {
        margin-left: 18px;
    }
    .SeatStatus {
        text-align: center;
        font-size: 1.25rem;
    }
    p {
        margin-bottom: 0;
    }
}

.SeatInfo .SeatNumber p {
    font-weight: bolder;
}

</style>
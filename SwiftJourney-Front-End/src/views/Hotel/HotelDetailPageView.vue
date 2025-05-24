<template>
    <div>
        {{ hotelId }}
        {{ beginDate }}
        {{ endDate }}
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router';
import { hotelApi } from '@/api/HotelApi/hotelApi';
import { HotelOrderQuery, HotelRoomDetailInfo, HotelDetailInfo, HotelComment } from '@/interface/hotelInterface';
import { ElMessage } from 'element-plus';

const route = useRoute();
const hotelId = route.params.id as string;
const beginDate = computed(() => {
    const value = route.query.beginDate;
    if (typeof value === 'string') {
        return value;
    }
        return undefined;
});
const endDate = computed(() => {
    const value = route.query.endDate;
    if (typeof value === 'string') {
        return value;
    }
        return undefined;
});

const hotelOrderQuery: HotelOrderQuery = {
    hotelId: hotelId,
    beginDate: beginDate.value,
    endDate: endDate.value,
}

const hotelDetailInfo = ref<HotelDetailInfo>();
interface HotelRoomInfo extends HotelRoomDetailInfo {
    roomType: string,
}
const hotelRoomInfoList = ref<HotelRoomInfo[]>([]);

onMounted(async () => {
    getHotelDetailInfo();
    getHotelOrderInfo();
})

async function getHotelDetailInfo() {
    hotelApi.hotelInfo(hotelId)
    .then((res) => {
        if(res.status == 200) {
            hotelDetailInfo.value = res.data;
        } else {
            throw new Error(res.statusText);
        }
    }).catch((error) => {
        ElMessage.error(error);
        console.error(error);
    })
} 

async function getHotelOrderInfo() {
    hotelApi.hotelOrderInfo(hotelOrderQuery)
    .then((res) => {
        if(res.status == 200) {
            let myMap = new Map(Object.entries(res.data as { [key: string]: HotelRoomDetailInfo }));
            myMap.forEach((value, key) => {
                let tepHotelRoomInfo: HotelRoomInfo = {
                    ...value,
                    roomType: key,
                }
                hotelRoomInfoList.value.push(tepHotelRoomInfo);
            })
        } else {
            throw new Error(res.statusText);
        }
    }) .catch((error) => {
        ElMessage.error(error);
        console.error(error);
    })
}
</script>
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
import type { HotelOrderQuery, HotelRoomDetailInfo, HotelDetailInfo, HotelComment } from '@/interface/hotelInterface';

const route = useRoute();
const hotelId = route.params.id as string;
const beginDate = computed(() => route.query.beginDate);
const endDate = computed(() => route.query.endDate);

onMounted(async () => {

})
</script>
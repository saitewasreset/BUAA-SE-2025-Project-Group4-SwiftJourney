<template>
    <div style="width: 100%;">
        <!-- TODO 首页API -->
        <div style="display: flex; flex-direction: column; align-items: center;">
            <TicketSearch />
            <TravelInfo />
        </div>
    </div>
</template>

<script setup lang="ts">
    import TicketSearch from '@/components/TicketSearch/TicketSearch.vue';
    import TravelInfo from '@/components/TravelInfo/TravelInfo.vue';
</script>

<style lang="css">
</style>
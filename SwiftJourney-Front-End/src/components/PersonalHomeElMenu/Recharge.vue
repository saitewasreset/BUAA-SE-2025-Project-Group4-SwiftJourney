<template>
    <h2>您的余额: {{ user.remainingMoney }}</h2>
    <a-button type="primary" @click="handleUserRecharge">
        充值
    </a-button>
</template>
<script lang="ts" setup>
    import { useUserStore } from '@/stores/user';
import { ref } from 'vue';

    const user = useUserStore();

    const userRechargeModalVisible = ref(false);

    function handleUserRecharge() {
        userRechargeModalVisible.value = true;
    }

</script>
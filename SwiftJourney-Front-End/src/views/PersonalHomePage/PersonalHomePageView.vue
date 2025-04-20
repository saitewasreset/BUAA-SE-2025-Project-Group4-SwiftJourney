<template>
    <div class="tabs-container">
    <!-- 使用 el-tabs -->
    <el-tabs v-model="activeIndex" tab-position="left">
        <el-tab-pane label="个人资料" name="personaldata">
            <PersonalData />
        </el-tab-pane>
        <el-tab-pane label="账户安全" name="accountsecurity">
            <AccountSecurity />
        </el-tab-pane>
        <el-tab-pane label="预填信息" name="prefilledinformation">
            <PrefilledInformation />
        </el-tab-pane>
        <el-tab-pane label="余额充值" name="recharge">
            <Recharge />
        </el-tab-pane>
        <el-tab-pane label="交易记录" name="transactionrecord">
            <TransactionRecord />
        </el-tab-pane>
    </el-tabs>
    </div>
</template>

<script setup>
import { ref } from 'vue';

import AccountSecurity from '@/components/PersonalHomeElMenu/AccountSecurity.vue';
import PersonalData from '@/components/PersonalHomeElMenu/PersonalData.vue';
import PrefilledInformation from '@/components/PersonalHomeElMenu/PrefilledInformation.vue';
import Recharge from '@/components/PersonalHomeElMenu/Recharge.vue';
import TransactionRecord from '@/components/PersonalHomeElMenu/TransactionRecord.vue';

const activeIndex = ref('personaldata');

</script>

<style scoped>
.tabs-container {
    position: fixed;
    top: 10%;
    left: 60px;
    width: 100%; /* 根据需要调整宽度 */
    height: 90%; /* 根据需要调整高度 */
}
</style>
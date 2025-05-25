<template>
    <div class="TitleBar">
        <el-menu
        :default-active="activeIndex"
        class="el-menu"
        mode="horizontal"
        :ellipsis="false"
        router
        >
            <div class="webside_title">
                <img src="@/assets/railway.svg" width="30" height="30" />
                <p style="font-size: larger">风行旅途</p>
            </div>
            <div style="border-left: 1px solid #ccc; height: 24px; margin: auto 10px;"></div>
            <el-menu-item index="homepage">首页</el-menu-item>
            <el-menu-item index="trainTicket">车票</el-menu-item>
            <el-menu-item index="hotel">酒店</el-menu-item>
            <el-menu-item index="trainmeal">火车餐</el-menu-item>
        </el-menu>
        <div class="TitleBarButton">
            <div v-if="!user.isLogin">
                <el-button @click="goToLoginPage" type="primary" round>登录</el-button>
                <el-button @click="goToRegisterPage" class="TailButton" type="success" round>注册</el-button>
            </div>
            <div v-else>
                    <el-popover
                        placement="top"
                        trigger="hover"
                        width="220px"
                    >
                    <div class="Popover">
                        <div class="PopoverTitle">
                            {{ user.name }}
                        </div>
                        <div class="PopoverSubTitle">
                            {{ user.phone }}
                        </div>
                        <div class="PopoverContent">
                            
                            <div class="RemainingMoney">
                                <p>账户余额</p>
                                <div class="Money"> {{ user.remainingMoney }} </div>
                            </div>

                            <div class="UserButtonLine">
                                <el-button link plain @click="goToPersonalDataPage">个人资料</el-button>
                                <el-button link plain @click="gotoRechargePage">余额充值</el-button>
                            </div>

                            <div class="UserButtonLine">
                                <el-button link plain @click="goToPrefilledIforPage">预填信息</el-button>
                                <el-button link plain @click="goToAccountSecurityPage">账户安全</el-button>
                            </div>

                            <div class="UserButtonLine">
                                <el-button link plain @click="goToTransactionRecordPage">交易记录</el-button>
                            </div>
                        </div>
                    </div>
                    <template #reference>
                        <el-button type="dashed" class="UserInfoButton" round @click="goToPersonalDataPage">
                          {{ user.name }}
                        </el-button>
                    </template>
                </el-popover>
                <el-button class="LogoutButton" @click="confirmLogout" type="danger" round>登出</el-button>
            </div>
        </div>
    </div>
    <a-modal
        v-model:visible="rechargeModalVisible"
        width="500px"
        @cancel="rechargeModalVisible = false; rechargeAmount = 0"
    >
        <template #title>
            <h3 style="text-align: center;">充值</h3>
        </template>
        <a-form>
            <a-form-item label="当前余额">
                <span>{{ user.remainingMoney }}</span>
            </a-form-item>
            <a-form-item label="充值金额">
                <a-input-number @change="checkRechargeAmount" v-model:value="rechargeAmount" style="width: 80%" />
            </a-form-item>
            <a-form-item label="充值方式">
                <a-select v-model:value="rechargeMethod" style="width: 80%" placeholder="请选择充值方式">
                    <a-select-option value="alipay">支付宝</a-select-option>
                    <a-select-option value="wechat">微信</a-select-option>
                    <a-select-option value="bank">银行卡</a-select-option>
                </a-select>
            </a-form-item>
        </a-form>
        <template #footer>
            <a-button type="primary" @click="handleUserRecharge">充值</a-button>
            <a-button @click="rechargeModalVisible = false; rechargeAmount = 0">取消</a-button>
        </template>
    </a-modal>
</template>
  
<script lang="ts" setup>
    import { paymentApi } from '@/api/PaymentApi/paymentApi';
import type { PaymentApiResponseData, RechargeRequest } from '@/interface/paymentInterface';
import { useUserStore } from '@/stores/user';
    import { message, Modal } from 'ant-design-vue';
    import { ref } from 'vue';
    import { RouterLink, RouterView } from 'vue-router';
    import { useRouter } from 'vue-router';

    const user = useUserStore();

    const router = useRouter();

    const activeIndex = ref('homepage');

    function goToLoginPage() {
        router.push({ name: 'login' });
    }

    function goToRegisterPage() {
        router.push({ name: 'register' });
    }

    function goToAccountSecurityPage() {
        router.push({name: 'personalhomepage', params: { activeIndex: 'accountsecurity' }});
    }

    function goToPersonalDataPage() {
        router.push({name: 'personalhomepage', params: { activeIndex: 'personaldata' }});
    }

    function goToPrefilledIforPage() {
        router.push({name: 'personalhomepage', params: { activeIndex: 'prefilledinformation' }});
    }

    function goToTransactionRecordPage() {
        router.push({ name: 'personalhomepage', params: { activeIndex: 'transactionrecord' }});
    }

    function confirmLogout() {
        Modal.confirm({
            title: '确认登出',
            content: '您确定要登出吗？',
            okText: '确认',
            cancelText: '取消',
            onOk: async () => {
                await user.logout(router);
            }
        });
    }


    const rechargeModalVisible = ref<Boolean>(false);
    const rechargeAmount = ref<number>(0);
    const rechargeMethod = ref<string>('alipay');

    function gotoRechargePage() {
        rechargeModalVisible.value = true;
    }

    function checkRechargeAmount() {
        if (rechargeAmount.value <= 0) {
            message.warning('充值金额必须大于0');
            rechargeAmount.value = 0;
        }
    }

    async function handleUserRecharge() {
        try {
            const params: RechargeRequest = {
                amount: rechargeAmount.value,
                externalPaymentId: null,
            };
            const res: PaymentApiResponseData = (await paymentApi.recharge(params)).data;
            if(res.code === 200)
                message.success('充值成功');
            else {
                message.error('登录信息过期，请重新登录');
            }
            await user.restoreUserFromCookie(router);
        } catch(error) {
            console.log(error);
        }
    }
</script>

<style lang="css" scoped>

.TitleBar {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    z-index: 1000;
    background-color: #fff;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.el-menu {
    display: flex;
}
.webside_title {
    margin: auto 10px auto 30px;
    display: flex;
    align-items: center;
    gap: 10px;
}

.webside_title img {
    display: inline-block;
}

.webside_title p {
    margin: 0;
    font-size: larger;
    font-weight: bold;
    color: #333;
}

.TitleBarButton {
    display: flex;
    align-items: center;
    margin-right: 10px;
}

.Popover {
    text-align: center;
}

.PopoverTitle {
    font-size: 20px;
    font-weight: bold;
    color: #333;
}

.PopoverSubTitle {
    font-size: 13px;
}

.PopoverContent {
    .RemainingMoney {
        display: flex;
        justify-content: center;
        align-items: center;
        margin: 10px auto;
        p {
            font-weight: bold;
            margin: 0;
            font-size: 16px;
        }
        .Money {
            margin-left: 30px;
            font-size: 16px;
        }
    }
    .UserButtonLine {
        display: flex;
        justify-content: center;
        margin-top: 10px;
        margin-left: 20px;
        margin-right: 20px;
        .el-button {
            font-size: 16px;
        }
    }
}


.LogoutButton {
    margin-left: 20px;
    margin-right: 20px;
}

</style>
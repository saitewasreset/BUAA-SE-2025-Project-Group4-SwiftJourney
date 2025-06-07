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
                        placement="bottom-end"
                        trigger="hover"
                        width="240px"
                    >
                    <div class="Popover">
                        <div class="PopoverTitle">
                            {{ user.username }}
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
                                <el-button link plain @click="goToTravelPlanPage">行程</el-button>
                                <el-button link plain @click="goToTransactionRecordPage">交易记录</el-button>
                            </div>
                        </div>
                    </div>
                    <template #reference>
                        <el-button type="dashed" class="UserInfoButton" round @click="goToPersonalDataPage">
                          {{ user.username }}
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
    import { computed, onMounted, ref } from 'vue';
    import { RouterLink, RouterView } from 'vue-router';
    import { useRouter, useRoute } from 'vue-router';

    const user = useUserStore();
    //import { useDebugUserStore } from '@/stores/user';
    //const user = useDebugUserStore();

    const router = useRouter();
    const route = useRoute();

    const activeIndex = computed(() => {
        return route.name;
    });

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

    function goToTravelPlanPage() {
        router.push({ name: 'personalhomepage', params: { activeIndex: 'travelplan' }});
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
        if (rechargeAmount.value < 0) {
            message.warning('充值金额必须大于0');
            rechargeAmount.value = 0;
        }
    }

    async function handleUserRecharge() {
        if(rechargeAmount.value === 0) {
            message.warning('充值金额不能为0');
            return;
        }
        else if(rechargeAmount.value === null) {
            message.warning('充值金额不能为空');
            return;
        }
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
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(20px);
    box-shadow: 
        0 4px 24px rgba(0, 0, 0, 0.1),
        0 0 0 1px rgba(255, 255, 255, 0.2);
    display: flex;
    justify-content: space-between;
    align-items: center;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    border-bottom: 1px solid rgba(226, 232, 240, 0.3);
}

.TitleBar:hover {
    box-shadow: 
        0 8px 32px rgba(0, 0, 0, 0.15),
        0 0 0 1px rgba(255, 255, 255, 0.3);
}

.el-menu {
    display: flex;
    background: transparent !important;
    border: none !important;
}

.el-menu :deep(.el-menu-item) {
    padding: 0 24px;
    height: 60px;
    line-height: 60px;
    font-weight: 500;
    font-size: 15px;
    color: #64748b;
    border-radius: 12px;
    margin: 0 4px;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    overflow: hidden;
}

.el-menu :deep(.el-menu-item::before) {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(135deg, rgba(102, 126, 234, 0.1) 0%, rgba(118, 75, 162, 0.1) 100%);
    transform: scaleX(0);
    transform-origin: left;
    transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    z-index: -1;
}

.el-menu :deep(.el-menu-item:hover) {
    background: transparent;
    color: #667eea;
    transform: translateY(-2px);
}

.el-menu :deep(.el-menu-item:hover::before) {
    transform: scaleX(1);
}

.el-menu :deep(.el-menu-item.is-active) {
    background: linear-gradient(135deg, rgba(102, 126, 234, 0.15) 0%, rgba(118, 75, 162, 0.15) 100%);
    color: #667eea;
    font-weight: 600;
    border-bottom: 3px solid #667eea;
}

.webside_title {
    margin: auto 24px auto 40px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 16px;
    border-radius: 16px;
    background: linear-gradient(135deg, rgba(102, 126, 234, 0.05) 0%, rgba(118, 75, 162, 0.05) 100%);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.webside_title:hover {
    transform: scale(1.02);
    background: linear-gradient(135deg, rgba(102, 126, 234, 0.1) 0%, rgba(118, 75, 162, 0.1) 100%);
}

.webside_title img {
    display: inline-block;
    filter: drop-shadow(0 2px 4px rgba(102, 126, 234, 0.3));
}

.webside_title p {
    margin: 0;
    font-size: 20px;
    font-weight: 700;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    letter-spacing: -0.5px;
}

.TitleBarButton {
    display: flex;
    align-items: center;
    margin-right: 24px;
    gap: 16px;
}

/* 登录注册按钮样式 */
.TitleBarButton .el-button {
    padding: 12px 24px;
    border-radius: 12px;
    font-weight: 600;
    font-size: 15px;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.TitleBarButton .el-button:hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.15);
}

.TitleBarButton .el-button--primary {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    border: none;
}

.TitleBarButton .el-button--success {
    background: linear-gradient(135deg, #10b981 0%, #059669 100%);
    border: none;
}

.TitleBarButton .el-button--danger {
    background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
    border: none;
}

/* 用户信息按钮样式 */
.UserInfoButton {
    padding: 12px 20px !important;
    border-radius: 12px !important;
    font-weight: 600 !important;
    font-size: 15px !important;
    background: rgba(255, 255, 255, 0.8) !important;
    border: 2px solid rgba(102, 126, 234, 0.2) !important;
    color: #667eea !important;
    backdrop-filter: blur(10px) !important;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1) !important;
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.15) !important;
}

.UserInfoButton:hover {
    transform: translateY(-2px) !important;
    background: linear-gradient(135deg, rgba(102, 126, 234, 0.1) 0%, rgba(118, 75, 162, 0.1) 100%) !important;
    border-color: #667eea !important;
    box-shadow: 0 6px 20px rgba(102, 126, 234, 0.25) !important;
}

.LogoutButton {
    margin-left: 16px !important;
    margin-right: 0 !important;
}

/* Popover样式优化 */
.Popover {
    text-align: left;
    background: rgba(255, 255, 255, 0.98);
    backdrop-filter: blur(20px);
    border-radius: 12px;
    padding: 0;
    box-shadow: 
        0 8px 24px rgba(0, 0, 0, 0.08),
        0 0 0 1px rgba(255, 255, 255, 0.1);
    overflow: hidden;
    min-width: 200px;
    max-width: 220px;
}

/* 用户头部信息区域 */
.PopoverTitle {
    font-size: 15px;
    font-weight: 600;
    color: #1a202c;
    padding: 16px 16px 4px;
    margin: 0;
    letter-spacing: -0.2px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.PopoverSubTitle {
    font-size: 11px;
    color: #64748b;
    font-weight: 400;
    opacity: 0.7;
    padding: 0 16px 12px;
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    border-bottom: 1px solid rgba(226, 232, 240, 0.2);
}

/* 内容区域 */
.PopoverContent {
    padding: 0;
}

/* 余额显示区域 */
.PopoverContent .RemainingMoney {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin: 0;
    padding: 12px 16px;
    background: linear-gradient(135deg, rgba(102, 126, 234, 0.05) 0%, rgba(118, 75, 162, 0.05) 100%);
    border-bottom: 1px solid rgba(226, 232, 240, 0.15);
}

.PopoverContent .RemainingMoney p {
    font-weight: 500;
    margin: 0;
    font-size: 12px;
    color: #64748b;
}

.PopoverContent .RemainingMoney .Money {
    margin-left: 0;
    font-size: 16px;
    font-weight: 700;
    color: #667eea;
    display: flex;
    align-items: center;
}

.PopoverContent .RemainingMoney .Money::before {
    content: '¥';
    font-size: 11px;
    margin-right: 1px;
    opacity: 0.8;
}

/* 功能按钮区域 - 改为网格布局 */
.UserButtonLine {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    margin: 0 !important;
    padding: 12px 16px 8px;
    position: relative;
}

/* .UserButtonLine:last-child {
    grid-template-columns: 1fr;
    padding: 8px 16px 16px;
    border-top: 1px solid rgba(226, 232, 240, 0.15);
    margin-top: 4px !important;
} */
/* .UserButtonLine {
    grid-template-columns: 1fr;
    padding: 8px 16px 16px;
    border-top: 1px solid rgba(226, 232, 240, 0.15);
    margin-top: 4px !important;
} */

.UserButtonLine .el-button {
    font-size: 11px !important;
    font-weight: 500 !important;
    padding: 8px 10px !important;
    border-radius: 6px !important;
    color: #64748b !important;
    background: rgba(248, 250, 252, 0.5) !important;
    border: 1px solid rgba(226, 232, 240, 0.3) !important;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1) !important;
    width: 100% !important;
    text-align: center !important;
    justify-content: center !important;
    position: relative !important;
    overflow: hidden !important;
    height: 30px !important;
    display: flex !important;
    align-items: center !important;
}

.UserButtonLine .el-button::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(135deg, rgba(102, 126, 234, 0.1) 0%, rgba(118, 75, 162, 0.1) 100%);
    transform: scaleX(0);
    transform-origin: left;
    transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    z-index: -1;
}

.UserButtonLine .el-button:hover {
    color: #667eea !important;
    background: rgba(255, 255, 255, 0.8) !important;
    border-color: rgba(102, 126, 234, 0.3) !important;
    transform: translateY(-1px) !important;
    box-shadow: 0 2px 6px rgba(102, 126, 234, 0.15) !important;
}

.UserButtonLine .el-button:hover::before {
    transform: scaleX(1);
}

/* 单独一行的按钮（交易记录）*/
.UserButtonLine:last-child .el-button {
    background: linear-gradient(135deg, rgba(102, 126, 234, 0.08) 0%, rgba(118, 75, 162, 0.08) 100%) !important;
    border: 1px solid rgba(102, 126, 234, 0.2) !important;
    color: #667eea !important;
    font-weight: 600 !important;
    height: 32px !important;
}

.UserButtonLine:last-child .el-button:hover {
    background: linear-gradient(135deg, rgba(102, 126, 234, 0.15) 0%, rgba(118, 75, 162, 0.15) 100%) !important;
    border-color: #667eea !important;
    transform: translateY(-2px) !important;
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.2) !important;
}

/* Popover组件美化 */
:deep(.el-popover) {
    background: rgba(255, 255, 255, 0.98) !important;
    backdrop-filter: blur(20px) !important;
    border: 1px solid rgba(255, 255, 255, 0.1) !important;
    border-radius: 12px !important;
    box-shadow: 
        0 8px 24px rgba(0, 0, 0, 0.08),
        0 0 0 1px rgba(255, 255, 255, 0.1) !important;
    padding: 0 !important;
    overflow: hidden !important;
    max-width: 220px !important;
}

:deep(.el-popover .el-popover__title) {
    display: none;
}

:deep(.el-popover__reference) {
    display: inline-block;
}

/* 移除图标 - 保持简洁 */
.UserButtonLine .el-button[data-type]::after {
    display: none;
}

/* 移除分隔线 - 使用间距代替 */
.PopoverContent .UserButtonLine:not(:last-child)::after {
    display: none;
}

/* 响应式优化 */
@media (max-width: 768px) {
    .Popover {
        min-width: 180px;
        max-width: 200px;
    }
    
    .PopoverTitle {
        font-size: 14px;
        padding: 14px 14px 4px;
    }
    
    .PopoverSubTitle {
        padding: 0 14px 10px;
        font-size: 10px;
    }

    .PopoverContent .RemainingMoney {
        padding: 10px 14px;
    }
    
    .UserButtonLine {
        padding: 10px 14px 6px;
        gap: 6px;
    }
    
    .UserButtonLine:last-child {
        padding: 6px 14px 14px;
    }

    .UserButtonLine .el-button {
        font-size: 10px !important;
        padding: 6px 8px !important;
        height: 28px !important;
    }
    
    .UserButtonLine:last-child .el-button {
        height: 30px !important;
    }
}

/* 微动画效果 */
@keyframes popoverFadeIn {
    from {
        opacity: 0;
        transform: translateY(-8px) scale(0.95);
    }
    to {
        opacity: 1;
        transform: translateY(0) scale(1);
    }
}

:deep(.el-popover) {
    animation: popoverFadeIn 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
</style>
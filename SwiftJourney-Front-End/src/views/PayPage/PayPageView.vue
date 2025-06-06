<template>
    <div class="pay-container">
        <!-- 支付表单卡片 -->
        <div v-if="!successPay" class="pay-card">
            <!-- 卡片头部 -->
            <div class="card-header">
                <div class="header-content">
                    <h1 class="page-title">支付订单</h1>
                    <p class="page-subtitle">安全便捷的支付体验</p>
                </div>
                <div class="header-icon">
                    <div class="icon-circle">
                        <el-icon><CreditCard /></el-icon>
                    </div>
                </div>
            </div>

            <!-- 分割线 -->
            <div class="divider"></div>

            <!-- 订单信息 -->
            <div class="order-info">
                <div class="info-grid">
                    <div class="info-item">
                        <span class="info-label">订单编号</span>
                        <span class="info-value">{{ transactionId }}</span>
                    </div>
                    <div class="info-item">
                        <span class="info-label">订单金额</span>
                        <span class="info-value amount">¥{{ money }}</span>
                    </div>
                </div>
            </div>

            <!-- 支付方式选择 -->
            <div class="payment-method">
                <div class="method-header">
                    <label class="method-label">支付方式</label>
                </div>
                <div class="method-options">
                    <el-radio-group v-model="payType" class="custom-radio-group">
                        <div class="radio-option">
                            <el-radio value="1" size="large">
                                <div class="radio-content">
                                    <el-icon class="radio-icon"><Lock /></el-icon>
                                    <span>账户密码</span>
                                </div>
                            </el-radio>
                        </div>
                        <div class="radio-option">
                            <el-tooltip content="未设置支付密码" placement="right" :disabled="user.havePaymentPasswordSet">
                                <el-radio value="2" size="large" :disabled="!user.havePaymentPasswordSet">
                                    <div class="radio-content">
                                        <el-icon class="radio-icon"><Key /></el-icon>
                                        <span>支付密码</span>
                                    </div>
                                </el-radio>
                            </el-tooltip>
                        </div>
                    </el-radio-group>
                </div>
            </div>

            <!-- 密码输入区域 -->
            <div class="password-section">
                <!-- 账户密码输入 -->
                <div v-if="payType == '1'" class="password-input">
                    <label class="input-label">请输入账户密码</label>
                    <el-input 
                        type="password" 
                        v-model="userPassword" 
                        maxlength="20" 
                        show-password
                        placeholder="请输入您的账户密码"
                        class="custom-input"
                    />
                </div>

                <!-- 支付密码输入 -->
                <div v-if="payType == '2'" class="payment-password">
                    <label class="input-label">请输入六位支付密码</label>
                    <div class="password-digits">
                        <input 
                            v-for="(digit, index) in digits" 
                            :key="index" 
                            class="digit-input" 
                            type="password"
                            maxlength="1" 
                            v-model="digits[index]" 
                            @input="onInput(index)" 
                            @keydown="onKeyDown"
                        />
                    </div>
                </div>
            </div>

            <!-- 确认按钮 -->
            <div class="card-footer">
                <div class="action-buttons">
                    <el-button class="confirm-button" type="primary" @click="confirmPay">
                        <el-icon><Check /></el-icon>
                        确认支付
                    </el-button>
                </div>
            </div>
        </div>

        <!-- 支付成功卡片 -->
        <div v-else class="success-card">
            <div class="success-content">
                <div class="success-icon">
                    <el-icon class="check-icon"><SuccessFilled /></el-icon>
                </div>
                <h2 class="success-title">支付成功</h2>
                <p class="success-subtitle">订单已加入处理队列</p>
                <div class="success-details">
                    <div class="detail-item">
                        <span class="detail-label">订单编号</span>
                        <span class="detail-value">{{ transactionId }}</span>
                    </div>
                    <div class="detail-item">
                        <span class="detail-label">支付金额</span>
                        <span class="detail-value">¥{{ money }}</span>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ElMessage, ElMessageBox } from 'element-plus';
import { ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { paymentApi } from '@/api/PaymentApi/paymentApi';
import { useUserStore } from '@/stores/user';
import type { UserApiBalanceData } from '@/interface/userInterface'; 
import { userApi } from '@/api/UserApi/userApi';

const user = useUserStore();
const router = useRouter();

const payType = ref('1')

const route = useRoute();
const transactionId = route.params.transactionId as string;
const money = route.query.money;

const userPassword = ref<string>('');
const digits = ref(new Array(6).fill(''));
const payPassword = ref<string>('');

const successPay = ref<boolean>(false);
const countdown = ref<number>(2);

// 监听支付成功状态，开始倒计时
watch(successPay, (newValue) => {
    if (newValue) {
        startCountdown();
    }
});

function startCountdown() {
    const timer = setInterval(() => {
        countdown.value--;
        if (countdown.value <= 0) {
            clearInterval(timer);
            router.push('/homepage');
        }
    }, 1000);
}

function onInput(index: number) {
    if (index < digits.value.length - 1 && digits.value[index].length === 1) {
        (document.querySelectorAll('.digit-input')[index + 1] as HTMLElement).focus();
    }
}

function onKeyDown(event: any) {
    if (event.key === 'Backspace' && event.target.value === '') {
        const index = Array.from(document.querySelectorAll('.digit-input')).indexOf(event.target);
        if (index > 0) {
            (document.querySelectorAll('.digit-input')[index - 1] as HTMLElement).focus();
        }
    }
}

function confirmPay() {
    if(!checkInput()) {
        return;
    }
    ElMessageBox.confirm(
        '确定立即支付？',
        '警告',
        {
            confirmButtonText: '确定',
            cancelButtonText: '取消',
            type: 'warning',
        }
    ).then(() => {
        apiPay();
    })
}

interface PaymentConfirmation {
  userPassword?: string;
  paymentPassword?: string;
}

async function apiPay() {
    let payment: PaymentConfirmation = {};
    if(payType.value == '1') {
        payment.userPassword = userPassword.value;
    } else if (payType.value == '2') {
        payment.paymentPassword = payPassword.value;
    }
    await paymentApi.pay(transactionId, payment)
    .then((res) => {
        if(res.status == 200) {
            //成功支付
            if(res.data.code == 200) {
                successPay.value = true;
                setBalance();
            } else if (res.data.code == 11001) {
                ElMessage.error('支付密码错误');
            } else if (res.data.code == 11002) {
                ElMessage.error('账户密码错误');
            } else if (res.data.code == 11003) {
                ElMessage.error('支付密码错误次数过多，请稍后重试');
            } else if (res.data.code == 11004) {
                ElMessage.error('余额不足');
            } else if (res.data.code == 11006) {
                ElMessage.error('交易状态错误');
            } else {
                throw new Error(res.statusText);
            }
        }
    }).catch((err) => {
        ElMessage.error(err)
    })
}

async function setBalance() {
    try {
        const balRes: UserApiBalanceData = (await userApi.queryUserBalance()).data;
        if(balRes.code === 200) {
            const balance: number = balRes.data.balance;
            user.setUserBalance(balance);
        }
        else
            throw new Error('invalid session id');
    } catch(e: any) {
        console.log(e);
    }
}

function checkInput() {
    if(payType.value == '1') {
        if(userPassword.value.trim() == '') {
            ElMessage.error('请输入账户密码');
            return false;
        }
    } else if (payType.value == '2') {
        for(let i = 0; i < 6; i++) {
            if(digits.value[i] == '') {
                ElMessage.error('请输入六位支付密码');
                return false;
            }
        }
        for(let i = 0; i < 6; i++) {
            if(!/^\d$/.test(digits.value[i])){
                ElMessage.error('支付密码只能是数字');
                return false;
            }
        }
        payPassword.value = digits.value.join('');
    } 
    return true;
}
</script>

<style scoped>
/* 容器样式 */
.pay-container {
    min-height: 80vh;
    padding: 40px 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
}

/* 支付卡片 */
.pay-card {
    width: 100%;
    max-width: 600px;
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(20px);
    border-radius: 24px;
    box-shadow: 
        0 20px 40px rgba(0, 0, 0, 0.1),
        0 0 0 1px rgba(255, 255, 255, 0.2);
    overflow: hidden;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.pay-card:hover {
    transform: translateY(-4px);
    box-shadow: 
        0 32px 64px rgba(0, 0, 0, 0.15),
        0 0 0 1px rgba(255, 255, 255, 0.3);
}

/* 卡片头部 */
.card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 40px 40px 20px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
}

.header-content {
    flex: 1;
}

.page-title {
    font-size: 28px;
    font-weight: 700;
    margin: 0 0 8px 0;
    letter-spacing: -0.5px;
}

.page-subtitle {
    font-size: 16px;
    margin: 0;
    opacity: 0.9;
    font-weight: 400;
}

.header-icon {
    margin-left: 24px;
}

.icon-circle {
    width: 60px;
    height: 60px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 24px;
}

/* 分割线 */
.divider {
    height: 1px;
    background: linear-gradient(90deg, transparent, #e2e8f0, transparent);
    margin: 0 40px;
}

/* 订单信息 */
.order-info {
    padding: 32px 40px;
}

.info-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;
}

.info-item {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.info-label {
    font-size: 14px;
    color: #6b7280;
    font-weight: 500;
}

.info-value {
    font-size: 18px;
    font-weight: 600;
    color: #374151;
}

.info-value.amount {
    color: #667eea;
    font-size: 24px;
    font-weight: 700;
}

/* 支付方式 */
.payment-method {
    padding: 0 40px 32px;
}

.method-header {
    margin-bottom: 20px;
}

.method-label {
    font-size: 16px;
    font-weight: 600;
    color: #374151;
}

.custom-radio-group {
    display: flex;
    gap: 16px;
}

.radio-option {
    flex: 1;
}

.radio-content {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 16px 20px;
    border: 2px solid #e5e7eb;
    border-radius: 12px;
    transition: all 0.3s;
    background: #f9fafb;
}

.radio-content:hover {
    border-color: #667eea;
    background: #fff;
}

.radio-icon {
    font-size: 18px;
    color: #6b7280;
}

:deep(.el-radio.is-checked .radio-content) {
    border-color: #667eea;
    background: linear-gradient(135deg, #667eea10 0%, #764ba210 100%);
}

:deep(.el-radio.is-checked .radio-icon) {
    color: #667eea;
}

/* 密码输入区域 */
.password-section {
    padding: 0 40px 32px;
}

.password-input,
.payment-password {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.input-label {
    font-size: 15px;
    font-weight: 600;
    color: #374151;
}

.custom-input :deep(.el-input__wrapper) {
    border-radius: 12px;
    border: 2px solid #e5e7eb;
    background-color: #f9fafb;
    transition: all 0.3s;
    box-shadow: none;
    padding: 16px;
}

.custom-input :deep(.el-input__wrapper):hover {
    border-color: #d1d5db;
    background-color: #fff;
}

.custom-input.is-focused :deep(.el-input__wrapper) {
    border-color: #667eea;
    background-color: #fff;
    box-shadow: 0 0 0 4px rgba(102, 126, 234, 0.1);
}

/* 支付密码输入框 */
.password-digits {
    display: flex;
    gap: 12px;
    justify-content: center;
}

.digit-input {
    width: 48px;
    height: 48px;
    font-size: 20px;
    text-align: center;
    border: 2px solid #e5e7eb;
    border-radius: 12px;
    outline: none;
    background: #f9fafb;
    transition: all 0.3s;
    font-weight: 600;
}

.digit-input:focus {
    border-color: #667eea;
    background: #fff;
    box-shadow: 0 0 0 4px rgba(102, 126, 234, 0.1);
}

.digit-input:hover {
    border-color: #d1d5db;
    background: #fff;
}

/* 卡片底部 */
.card-footer {
    padding: 32px 40px;
    background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
    border-top: 1px solid rgba(226, 232, 240, 0.5);
}

.action-buttons {
    display: flex;
    justify-content: center;
}

.confirm-button {
    padding: 16px 48px;
    border-radius: 12px;
    font-weight: 600;
    font-size: 16px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    border: none;
    box-shadow: 0 8px 24px rgba(102, 126, 234, 0.3);
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    min-width: 200px;
}

.confirm-button:hover {
    transform: translateY(-2px);
    box-shadow: 0 12px 32px rgba(102, 126, 234, 0.4);
}

/* 支付成功卡片 */
.success-card {
    width: 100%;
    max-width: 500px;
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(20px);
    border-radius: 24px;
    box-shadow: 
        0 20px 40px rgba(0, 0, 0, 0.1),
        0 0 0 1px rgba(255, 255, 255, 0.2);
    overflow: hidden;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.success-card:hover {
    transform: translateY(-4px);
    box-shadow: 
        0 32px 64px rgba(0, 0, 0, 0.15),
        0 0 0 1px rgba(255, 255, 255, 0.3);
}

.success-content {
    padding: 60px 40px;
    text-align: center;
}

.success-icon {
    margin-bottom: 32px;
}

.check-icon {
    font-size: 80px;
    color: #10b981;
    animation: bounceIn 0.6s cubic-bezier(0.68, -0.55, 0.265, 1.55);
}

.success-title {
    font-size: 32px;
    font-weight: 700;
    color: #1a202c;
    margin: 0 0 16px 0;
    letter-spacing: -0.5px;
}

.success-subtitle {
    font-size: 18px;
    color: #64748b;
    margin: 0 0 40px 0;
    font-weight: 400;
}

.success-details {
    display: flex;
    flex-direction: column;
    gap: 16px;
    background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
    padding: 24px;
    border-radius: 16px;
}

.detail-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.detail-label {
    font-size: 15px;
    color: #6b7280;
    font-weight: 500;
}

.detail-value {
    font-size: 16px;
    font-weight: 600;
    color: #374151;
}

/* 动画效果 */
@keyframes bounceIn {
    0% {
        opacity: 0;
        transform: scale(0.3);
    }
    50% {
        opacity: 1;
        transform: scale(1.05);
    }
    70% {
        transform: scale(0.9);
    }
    100% {
        opacity: 1;
        transform: scale(1);
    }
}

@keyframes fadeInUp {
    from {
        opacity: 0;
        transform: translateY(20px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.order-info,
.payment-method,
.password-section {
    animation: fadeInUp 0.6s cubic-bezier(0.4, 0, 0.2, 1) forwards;
}

.order-info { animation-delay: 0.1s; }
.payment-method { animation-delay: 0.2s; }
.password-section { animation-delay: 0.3s; }

/* 响应式设计 */
@media (max-width: 768px) {
    .pay-container {
        padding: 20px 16px;
    }

    .pay-card,
    .success-card {
        border-radius: 20px;
    }

    .card-header {
        flex-direction: column;
        text-align: center;
        padding: 32px 24px 20px;
    }

    .header-icon {
        margin: 20px 0 0 0;
    }

    .icon-circle {
        width: 48px;
        height: 48px;
        font-size: 20px;
    }

    .page-title {
        font-size: 24px;
    }

    .order-info,
    .payment-method,
    .password-section,
    .card-footer {
        padding-left: 24px;
        padding-right: 24px;
    }

    .info-grid {
        grid-template-columns: 1fr;
        gap: 16px;
    }

    .custom-radio-group {
        flex-direction: column;
    }

    .password-digits {
        gap: 8px;
    }

    .digit-input {
        width: 40px;
        height: 40px;
        font-size: 18px;
    }

    .confirm-button {
        width: 100%;
        min-width: auto;
    }

    .success-content {
        padding: 40px 24px;
    }

    .check-icon {
        font-size: 60px;
    }

    .success-title {
        font-size: 24px;
    }
}

@media (max-width: 480px) {
    .pay-container {
        padding: 16px 12px;
    }

    .card-header {
        padding: 24px 20px 16px;
    }

    .page-title {
        font-size: 20px;
    }

    .order-info,
    .payment-method,
    .password-section,
    .card-footer {
        padding-left: 20px;
        padding-right: 20px;
    }

    .divider {
        margin: 0 20px;
    }

    .success-content {
        padding: 32px 20px;
    }
}
</style>
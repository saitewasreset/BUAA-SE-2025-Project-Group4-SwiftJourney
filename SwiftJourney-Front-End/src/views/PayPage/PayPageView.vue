<template>
    <div class="CardContainer">
        <el-card v-if="!successPay" class="PayCard">
            <p class="title">支付订单</p>
            <div class="line2">
                <p>订单编号：{{ transactionId }}</p>
                <p>订单金额：{{ money }}</p>
            </div>
            <div class="line3">
                <el-radio-group v-model="payType">
                    <el-radio value="1" size="large">账户密码</el-radio>
                    <el-tooltip content="未设置支付密码" placement="right" :disabled="user.havePaymentPasswordSet">
                        <el-radio value="2" size="large" :disabled="!user.havePaymentPasswordSet">支付密码</el-radio>
                    </el-tooltip>
                </el-radio-group>
            </div>
            <div v-if="payType == '1'" class="line4">
                <el-input type="password" class="text-input" id="userPassword" v-model="userPassword" maxlength="20" show-password
                placeholder="请输入您的账户密码"/>
            </div>
            <div v-if="payType == '2'" class="line4">
                <input v-for="(digit, index) in digits" :key="index" class="captcha-digit" type="text" 
                maxlength="1" v-model="digits[index]" @input="onInput(index)" @keydown="onKeyDown"/>
            </div>
            <div class="ConfirmButtonContainer">
                <el-button class="ConfirmButton" type="success" @click="confirmPay">确认支付</el-button>
            </div>
        </el-card>
        <el-card v-else class="PayCard">
            <el-icon class="PayedIcon"><SuccessFilled /></el-icon>
            <p class="PayedText">支付成功</p>
            <p class="PayedText">已加入处理队列</p>
        </el-card>
    </div>
</template>

<script setup lang="ts">
import { ElMessage, ElMessageBox } from 'element-plus';
import { ref } from 'vue';
import { useRoute } from 'vue-router';
import { paymentApi } from '@/api/PaymentApi/paymentApi';
import { useUserStore } from '@/stores/user';

const user = useUserStore();

const payType = ref('1')

const route = useRoute();
const transactionId = route.params.transactionId as string;
const money = route.query.money;

const userPassword = ref<string>('');
const digits = ref(new Array(6).fill(''));
const payPassword = ref<string>('');

const successPay = ref<boolean>(false);

function onInput(index: number) {
    if (index < digits.value.length - 1 && digits.value[index].length === 1) {
        (document.querySelectorAll('.captcha-digit')[index + 1] as HTMLElement).focus();
    }
}

function onKeyDown(event: any) {
    if (event.key === 'Backspace' && event.target.value === '') {
        const index = Array.from(document.querySelectorAll('.captcha-digit')).indexOf(event.target);
        if (index > 0) {
            (document.querySelectorAll('.captcha-digit')[index - 1] as HTMLElement).focus();
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
    paymentApi.pay(transactionId, payment)
    .then((res) => {
        if(res.status == 200) {
            //成功支付
            if(res.data.code == 200) {
                successPay.value = true;
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
.CardContainer {
    width: 100%;
    height: 60vh;
    display: flex;
    justify-content: center;
    align-items: center;
}

.PayCard {
    width: 650px;
    display: flex;
    align-items: center;
    flex-direction: column;
}
.PayedIcon {
    width: 500px;
    height: 100px;
    --color: lightgreen;
}
::v-deep(.PayedIcon svg) {
    width: 100px;
    height: 100px;
}
.PayedText {
    font-size: 50px; 
    font-weight: bold;
    color: grey;
    margin-bottom: 0;
    text-align: center;
}

.PayCard .title {
    font-size: 1.75rem;
    font-weight: bold;
    text-align: center;
    margin-bottom: 10px;
}
.PayCard .line2 {
    width: 550px;
    display: flex;
    justify-content: space-between;
}

.PayCard .line2 p {
    font-size: 16px;
    margin-bottom: 10px;
}

::v-deep(.el-radio.el-radio--large .el-radio__label) {
    font-size: 18px;
}

.ConfirmButtonContainer {
    padding-top: 20px;
    display: flex;
    justify-content: flex-end;
}
::v-deep(.ConfirmButton span) {
    font-size: 18px;
}

.PayCard .line3 {
    padding-bottom: 10px;
}

.PayCard .line4 {
    display: flex;
    align-items: center;
    justify-content: center;
}
.text-input {
    font-size: 18px;
    max-width: 400px;
}
.captcha-digit {
  width: 30px;
  height: 30px;
  font-size: 20px;
  text-align: center;
  margin-right: 10px;
  border: 1px solid #ccc;
  border-radius: 4px;
  outline: none;
  -webkit-text-security: disc; /* 使用小黑点隐藏输入字符 */
}

.captcha-digit:focus {
  border-color: #007bff;
}
</style>
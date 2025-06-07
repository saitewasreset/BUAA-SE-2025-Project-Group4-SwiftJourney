<template>
  <div class="root">
    <div class="left-area">
      <div class="brand-section">
        <div class="logo-container">
          <img src="../../assets/railway.svg" class="logo" />
          <div class="logo-glow"></div>
        </div>
        <h1 class="title">风行旅途</h1>
        <p class="subtitle">开启您的专属旅程</p>
      </div>
      <div class="decorative-elements">
        <div class="floating-circle circle-1"></div>
        <div class="floating-circle circle-2"></div>
        <div class="floating-circle circle-3"></div>
      </div>
    </div>
    
    <div class="right-area">
      <a-card class="card">
        <div class="card-header">
          <h2 class="card-title">创建账户</h2>
          <p class="card-subtitle">加入我们，开始您的旅行之旅</p>
        </div>
        
        <div class="form-section">
          <div class="input-group">
            <label class="input-label">手机号</label>
            <a-input
              v-model:value="inputPhone"
              type="string"
              :allowClear="true"
              placeholder="请输入手机号"
              class="input"
              size="large"
              :status="inputPhoneStatus"
              @input="checkPhoneNumber"
              @change="checkPhoneNumber"
            >
              <template #prefix>
                <TabletOutlined class="input-icon" />
              </template>
            </a-input>
            <p class="input-error" v-if="inputPhoneError">{{ phoneErrorMsg }}</p>
          </div>
          
          <div class="input-group">
            <label class="input-label">用户名</label>
            <a-input
              v-model:value="inputNickName"
              type="string"
              :allowClear="true"
              placeholder="请输入用户名"
              class="input"
              size="large"
              :status="inputNickNameStatus"
              @input="checkNickName"
              @change="checkNickName"
            >
              <template #prefix>
                <UserOutlined class="input-icon" />
              </template>
            </a-input>
            <p class="input-error" v-if="inputNickNameError">{{ nickNameErrorMsg }}</p>
          </div>
          
          <div class="input-group">
            <label class="input-label">密码</label>
            <a-tooltip
              title="密码长度应在8-20位之间，至少包含大小写字母、数字或特殊符号中的两种，且不能包含空格"
              placement="right"
            >
              <a-input-password
                v-model:value="inputPassword"
                type="string"
                :allowClear="true"
                placeholder="请输入密码"
                class="input"
                size="large"
                @input="checkInput"
                @change="checkInput"
                :status="inputPasswordStatus"
              >
                <template #prefix>
                  <LockOutlined class="input-icon" />
                </template>
              </a-input-password>
            </a-tooltip>
            <p class="input-error" v-if="inputPasswordError">{{ passwordErrorMsg }}</p>
          </div>
          
          <div class="input-group">
            <label class="input-label">确认密码</label>
            <a-input-password
              v-model:value="inputConfirmPassword"
              type="string"
              :allowClear="true"
              placeholder="请再次输入密码"
              class="input"
              size="large"
              @input="checkInput"
              @change="checkInput"
              :status="inputConfirmPasswordStatus"
            >
              <template #prefix>
                <LockOutlined class="input-icon" />
              </template>
            </a-input-password>
            <p class="input-error" v-if="inputConfirmPasswordError">{{ confirmPasswordErrorMsg }}</p>
          </div>
          
          <div class="input-row">
            <div class="input-group half-width">
              <label class="input-label">真实姓名</label>
              <a-input
                v-model:value="inputName"
                type="string"
                :allowClear="true"
                placeholder="请输入真实姓名"
                class="input"
                size="large"
              >
                <template #prefix>
                  <TagOutlined class="input-icon" />
                </template>
              </a-input>
            </div>
            
            <div class="input-group half-width">
              <label class="input-label">身份证号</label>
              <a-input
                v-model:value="inputIdNumber"
                type="string"
                :allowClear="true"
                placeholder="请输入身份证号"
                class="input"
                size="large"
                :status="inputIdNumberStatus"
                @input="checkIdNumber"
                @change="checkIdNumber"
              >
                <template #prefix>
                  <IdcardOutlined class="input-icon" />
                </template>
              </a-input>
            </div>
          </div>
          <p class="input-error" v-if="inputIdNumberError">{{ idNumberErrorMsg }}</p>
        </div>

        <div class="button-section">
          <a-button
            type="primary"
            @click="postRegisterMsg"
            class="register-button"
            size="large"
            :disabled="disableRegister"
            :loading="isLoading"
          >
            注册
          </a-button>
          
          <div class="login-section">
            <span class="login-text">已有账户？</span>
            <a @click="goToLoginPage" class="login-link">立即登录</a>
          </div>
        </div>
      </a-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import {
  TabletOutlined,
  UserOutlined,
  LockOutlined,
  ArrowLeftOutlined,
  TagOutlined,
  IdcardOutlined,
} from '@ant-design/icons-vue'
import { useRouter } from 'vue-router'
import { userApi } from '@/api/UserApi/userApi'
import { message } from 'ant-design-vue'
import type { UserApiResponseData } from '@/interface/userInterface'

const inputPhone = ref('')
const inputNickName = ref('')
const inputPassword = ref('')
const inputConfirmPassword = ref('')
const inputName = ref('')
const inputIdNumber = ref('')
const isLoading = ref(false)

interface UpdatePersonalInfo {
  // 姓名
  name?: string;
  // 身份证号
  identityCardId: string;
  // 偏好座位位置
  preferredSeatLocation?: "A" | "B" | "C" | "D" | "F";
  // 是否为默认个人资料，即，当前用户的身份
  default?: boolean;
}

// -------------------- 使用 Vue Router 进行路由导航 --------------------

const router = useRouter()

function goToHomePage() {
  router.push({ name: 'homepage' })
}

function goToLoginPage() {
  router.push({ name: 'login' })
}

// -------------------- 处理注册逻辑 --------------------

async function postRegisterMsg() {
  if (!inputPhone.value || !inputPassword.value || !inputConfirmPassword.value || 
      !inputNickName.value || !inputIdNumber.value || !inputName.value) {
    message.warning('请填写完整信息')
    return
  }
  
  isLoading.value = true
  
  try {
    console.log(
      'post register message: ' +
        inputPhone.value +
        ' ' +
        inputPassword.value +
        ' ' +
        inputConfirmPassword.value,
    )
    const params: Object = {
      phone: inputPhone.value,
      username: inputNickName.value,
      // 明文密码
      password: inputPassword.value,
      // 姓名
      name: inputName.value,
      // 身份证号
      identityCardId: inputIdNumber.value,
    };
    const res: UserApiResponseData = (await userApi.userRegister(params)).data;
    if (res.code === 200) {
      message.success('注册成功');
      router.push({ name: 'login' }) 
    } else if (res.code === 13001) {
      message.error('身份证号格式错误')
    } else if (res.code === 15001) {
      message.error('该手机号对应的用户已经存在')
    } else if (res.code === 15003) {
      message.error('用户名格式错误')
    } else if (res.code === 15004) {
      message.error('密码格式错误')
    } else if (res.code === 15005) {
      message.error('姓名格式错误')
    } else {
      message.error('其他错误');
    }
  } catch (error) {
    message.error('网络错误，请稍后重试')
  } finally {
    isLoading.value = false
  }
}

// -------------------- 手机号检查 --------------------

const inputPhoneStatus = ref('')
const inputPhoneError = ref(false)
const phoneErrorMsg = ref('')

function checkPhoneNumber() {
  const validPrefixes = [
    '134',
    '135',
    '136',
    '137',
    '138',
    '139',
    '144',
    '147',
    '148',
    '150',
    '151',
    '152',
    '157',
    '158',
    '159',
    '165',
    '170',
    '172',
    '178',
    '182',
    '183',
    '184',
    '187',
    '188',
    '195',
    '197',
    '198',
    '130',
    '131',
    '132',
    '140',
    '145',
    '146',
    '155',
    '156',
    '166',
    '167',
    '171',
    '175',
    '176',
    '185',
    '186',
    '196',
    '133',
    '141',
    '149',
    '153',
    '162',
    '173',
    '174',
    '177',
    '180',
    '181',
    '189',
    '190',
    '191',
    '193',
    '199',
    '192',
  ]

  inputPhoneStatus.value = ''
  inputPhoneError.value = false
  phoneErrorMsg.value = ''

  if (inputPhone.value === '') {
    return
  }

  if (inputPhone.value.length !== 11) {
    inputPhoneStatus.value = 'error'
    inputPhoneError.value = true
    phoneErrorMsg.value = '手机号长度应为11位'
    return 
  }

  const regex = /^1[3-9]\d{9}$/
  if (!regex.test(inputPhone.value)) {
    inputPhoneStatus.value = 'error'
    inputPhoneError.value = true
    phoneErrorMsg.value = '手机号格式不正确'
    return 
  }

  const prefix = inputPhone.value.substring(0, 3)
  if (!validPrefixes.includes(prefix)) {
    inputPhoneStatus.value = 'error'
    inputPhoneError.value = true
    phoneErrorMsg.value = '手机号无效'
  }
}

// -------------------- 昵称检查 --------------------

const inputNickNameStatus = ref('')
const inputNickNameError = ref(false)
const nickNameErrorMsg = ref('')

function checkNickName() {
  inputNickNameStatus.value = ''
  inputNickNameError.value = false
  nickNameErrorMsg.value = ''

  if (inputNickName.value === '') {
    return
  }

  if (inputNickName.value.length > 16) {
    inputNickNameStatus.value = 'error'
    inputNickNameError.value = true
    nickNameErrorMsg.value = '昵称长度应小于 16'
    return
  }
}

// -------------------- 输入框状态检查 --------------------

const inputPasswordStatus = ref('')
const inputPasswordError = ref(false)
const passwordErrorMsg = ref('')

const inputConfirmPasswordStatus = ref('')
const inputConfirmPasswordError = ref(false)
const confirmPasswordErrorMsg = ref('')

const inputIdNumberStatus = ref('')
const inputIdNumberError = ref(false)
const idNumberErrorMsg = ref('')

function checkInput() {
  initPasswordCheck()
  checkPassword()
  checkConfirmPassword()
}

function initPasswordCheck() {
  inputPasswordStatus.value = ''
  inputConfirmPasswordStatus.value = ''
  inputPasswordError.value = false
  inputConfirmPasswordError.value = false
  passwordErrorMsg.value = ''
  confirmPasswordErrorMsg.value = ''
}

function checkPassword() {
  if (inputPassword.value === '') {
    return
  }
  if (inputPassword.value.includes(' ')) {
    inputPasswordStatus.value = 'error'
    inputPasswordError.value = true
    passwordErrorMsg.value = '密码不能包含空格'
    return
  }
  if (inputPassword.value.length < 8 || inputPassword.value.length > 20) {
    inputPasswordStatus.value = 'error'
    inputPasswordError.value = true
    passwordErrorMsg.value = '密码长度应在 8 - 20 位之间'
    return
  }

  let matchedTypes = 0
  if (/[A-Za-z]/.test(inputPassword.value)) {
    matchedTypes += 1
  }
  // if (/[A-Z]/.test(inputPassword.value)) {
  //   matchedTypes += 1
  // }
  if (/\d/.test(inputPassword.value)) {
    matchedTypes += 1
  }
  // 特殊符号的正则表达式
  if (/[\W_]/.test(inputPassword.value)) {
    matchedTypes += 1
  }
  if (matchedTypes < 2) {
    inputPasswordStatus.value = 'error'
    inputPasswordError.value = true
    passwordErrorMsg.value = '密码必须至少包含大小写字母、数字或特殊符号中的两种'
  }
}

function checkConfirmPassword() {
  if (inputConfirmPassword.value !== '' && inputPassword.value === '') {
    inputConfirmPasswordStatus.value = 'error'
    inputConfirmPasswordError.value = true
    confirmPasswordErrorMsg.value = '请先输入密码'
    return
  }
  if (inputConfirmPassword.value !== '' && inputConfirmPassword.value !== inputPassword.value) {
    inputConfirmPasswordStatus.value = 'error'
    inputConfirmPasswordError.value = true
    confirmPasswordErrorMsg.value = '两次输入的密码不一致'
  }
}

function checkIdNumber() {
  inputIdNumberStatus.value = ''
  inputIdNumberError.value = false
  idNumberErrorMsg.value = ''

  if (inputIdNumber.value === '') {
    return
  }

  const weight = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2]
  const checkCode = ['1', '0', 'X', '9', '8', '7', '6', '5', '4', '3', '2']

  if (inputIdNumber.value.length !== 18) {
    inputIdNumberStatus.value = 'error'
    inputIdNumberError.value = true
    idNumberErrorMsg.value = '身份证号码长度应为18位'
    return
  }

  let sum = 0
  for (let i = 0; i < 17; i++) {
    if (!/\d/.test(inputIdNumber.value[i])) {
      inputIdNumberStatus.value = 'error'
      inputIdNumberError.value = true
      idNumberErrorMsg.value = '身份证号码前17位应全部为数字'
      return
    }
    sum += parseInt(inputIdNumber.value[i], 10) * weight[i]
  }

  // 计算模11后的余数
  const mod = sum % 11

  // 对比最后一位校验码
  const expectedCheckCode = checkCode[mod].toUpperCase()
  const actualCheckCode = inputIdNumber.value[17].toUpperCase()

  if (actualCheckCode !== expectedCheckCode) {
    inputIdNumberStatus.value = 'error'
    inputIdNumberError.value = true
    idNumberErrorMsg.value = `身份证号码校验失败`
  }
}

// -------------------- 注册按钮禁用逻辑 --------------------

const disableRegister = computed(() => {
  return (
    inputPhone.value === '' ||
    inputPassword.value === '' ||
    inputConfirmPassword.value === '' ||
    inputNickName.value === '' ||
    inputIdNumber.value === '' ||
    inputName.value === '' ||
    inputPasswordError.value ||
    inputConfirmPasswordError.value ||
    inputPhoneError.value ||
    inputNickNameError.value ||
    inputIdNumberError.value
  )
})
</script>

<style lang="scss" scoped>
// 渐变色变量
$primary-gradient: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
$secondary-gradient: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
$card-gradient: linear-gradient(145deg, rgba(255, 255, 255, 0.9) 0%, rgba(255, 255, 255, 0.8) 100%);

// 颜色变量
$primary-color: #667eea;
$secondary-color: #764ba2;
$text-primary: #2c3e50;
$text-secondary: #7f8c8d;
$border-color: #e1e8ed;
$success-color: #27ae60;
$error-color: #e74c3c;

.root {
  min-height: 100vh;
  display: grid;
  grid-template-columns: 1fr 1fr;
  background: $primary-gradient;
  position: relative;
  overflow: hidden;
  
  &::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><defs><pattern id="grain" width="100" height="100" patternUnits="userSpaceOnUse"><circle cx="50" cy="50" r="0.5" fill="rgba(255,255,255,0.1)"/></pattern></defs><rect width="100" height="100" fill="url(%23grain)"/></svg>');
    opacity: 0.3;
    pointer-events: none;
  }
  :deep(.ant-card-body) {
    padding: 0; // 去除默认内边距
    overflow: hidden;
  }
}

.left-area {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  padding: 1.5rem; // 从 2rem 减少到 1.5rem
  position: relative;
  z-index: 2;
  
  .brand-section {
    text-align: center;
    
    .logo-container {
      position: relative;
      margin-bottom: 2rem;
      
      .logo {
        width: 120px;
        height: 120px;
        filter: drop-shadow(0 8px 16px rgba(0, 0, 0, 0.2));
        transition: all 0.3s ease;
        
        &:hover {
          transform: scale(1.05);
        }
      }
      
      .logo-glow {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        width: 140px;
        height: 140px;
        background: radial-gradient(circle, rgba(255, 255, 255, 0.3) 0%, transparent 70%);
        border-radius: 50%;
        animation: pulse 3s ease-in-out infinite;
      }
    }
    
    .title {
      font-size: 3.5rem;
      font-weight: 800;
      color: white;
      margin: 0 0 1rem 0;
      text-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
      letter-spacing: 2px;
      background: linear-gradient(45deg, #fff, #f0f8ff);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      background-clip: text;
    }
    
    .subtitle {
      font-size: 1.2rem;
      color: rgba(255, 255, 255, 0.8);
      margin: 0;
      font-weight: 300;
      letter-spacing: 1px;
    }
  }
  
  .decorative-elements {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    pointer-events: none;
    
    .floating-circle {
      position: absolute;
      border-radius: 50%;
      background: rgba(255, 255, 255, 0.1);
      animation: float 6s ease-in-out infinite;
      
      &.circle-1 {
        width: 80px;
        height: 80px;
        top: 20%;
        left: 20%;
        animation-delay: 0s;
      }
      
      &.circle-2 {
        width: 60px;
        height: 60px;
        top: 60%;
        right: 30%;
        animation-delay: 2s;
      }
      
      &.circle-3 {
        width: 40px;
        height: 40px;
        bottom: 20%;
        left: 10%;
        animation-delay: 4s;
      }
    }
  }
}

.right-area {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1.5rem; // 从 2rem 减少到 1.5rem
  position: relative;
  z-index: 2;
}

.card {
  width: 520px;
  max-height: 90vh;
  padding: 2rem 1.5rem; // 左右内边距从 2rem 减少到 1.5rem
  border-radius: 24px;
  background: $card-gradient;
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.2);
  box-shadow: 
    0 32px 64px rgba(0, 0, 0, 0.15),
    0 16px 32px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.4);
  transition: all 0.3s ease;
  
  &:hover {
    transform: translateY(-4px);
    box-shadow: 
      0 40px 80px rgba(0, 0, 0, 0.2),
      0 20px 40px rgba(0, 0, 0, 0.15);
  }
  
  .card-header {
    text-align: center;
    margin-bottom: 2rem;
    
    .card-title {
      font-size: 1.8rem;
      font-weight: 700;
      color: $text-primary;
      margin: 0 0 0.3rem 0;
      background: linear-gradient(135deg, $primary-color, $secondary-color);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      background-clip: text;
    }
    
    .card-subtitle {
      font-size: 0.95rem;
      color: $text-secondary;
      margin: 0;
      font-weight: 400;
    }
  }
  
  .form-section {
    .input-group {
      margin-bottom: 1.2rem;
      
      &.half-width {
        width: calc(50% - 0.5rem);
      }
      
      .input-label {
        display: block;
        font-size: 0.85rem;
        font-weight: 600;
        color: $text-primary;
        margin-bottom: 0.4rem;
        letter-spacing: 0.5px;
      }
      
      .input {
        border-radius: 12px;
        border: 2px solid $border-color;
        transition: all 0.3s ease;
        background: rgba(255, 255, 255, 0.8);
        
        &:hover {
          border-color: rgba($primary-color, 0.5);
          box-shadow: 0 4px 12px rgba($primary-color, 0.1);
        }
        
        &:focus-within {
          border-color: $primary-color;
          box-shadow: 0 0 0 4px rgba($primary-color, 0.1);
          background: white;
        }
        
        .input-icon {
          color: $text-secondary;
          transition: color 0.3s ease;
        }
        
        &:focus-within .input-icon {
          color: $primary-color;
        }
      }
    }
    
    .input-row {
      display: flex;
      gap: 1rem;
      margin-bottom: 1.2rem;
    }
    
    .input-error {
      color: $error-color;
      font-size: 0.75rem;
      margin-top: 0.25rem;
      margin-left: 0.5rem;
      font-weight: 500;
    }
  }
  
  .button-section {
    margin-top: 1.5rem;
    
    .register-button {
      width: 100%;
      height: 48px;
      border-radius: 12px;
      background: $primary-gradient;
      border: none;
      font-size: 1rem;
      font-weight: 600;
      letter-spacing: 1px;
      transition: all 0.3s ease;
      box-shadow: 0 8px 16px rgba($primary-color, 0.3);
      
      &:hover:not(:disabled) {
        transform: translateY(-2px);
        box-shadow: 0 12px 24px rgba($primary_color, 0.4);
        background: linear-gradient(135deg, lighten($primary-color, 5%) 0%, lighten($secondary-color, 5%) 100%);
      }
      
      &:active:not(:disabled) {
        transform: translateY(0);
      }
      
      &:disabled {
        background: linear-gradient(135deg, #ccc 0%, #bbb 100%);
        box-shadow: none;
        cursor: not-allowed;
      }
    }
    
    .login-section {
      text-align: center;
      margin-top: 1.5rem;
      
      .login-text {
        color: $text-secondary;
        font-size: 0.9rem;
        margin-right: 0.5rem;
      }
      
      .login-link {
        color: $primary-color;
        text-decoration: none;
        font-weight: 600;
        font-size: 0.9rem;
        transition: all 0.3s ease;
        cursor: pointer;
        
        &:hover {
          color: $secondary-color;
          text-decoration: underline;
        }
      }
    }
  }
}

// 动画
@keyframes pulse {
  0%, 100% {
    opacity: 0.3;
    transform: translate(-50%, -50%) scale(1);
  }
  50% {
    opacity: 0.6;
    transform: translate(-50%, -50%) scale(1.1);
  }
}

@keyframes float {
  0%, 100% {
    transform: translateY(0) rotate(0deg);
  }
  33% {
    transform: translateY(-20px) rotate(120deg);
  }
  66% {
    transform: translateY(20px) rotate(240deg);
  }
}

// 响应式设计
@media (max-width: 1024px) {
  .root {
    grid-template-columns: 1fr;
    
    .left-area {
      display: none;
    }
    
    .right-area {
      padding: 0.8rem; // 从 1rem 减少到 0.8rem
    }
    
    .card {
      width: 100%;
      max-width: 480px;
    }
  }
}

@media (max-width: 480px) {
  .card {
    padding: 2rem 1rem; // 左右内边距从 1.5rem 减少到 1rem
    border-radius: 16px;
    
    .card-header .card-title {
      font-size: 1.5rem;
    }
    
    .form-section .input-row {
      flex-direction: column;
      gap: 0;
      
      .input-group.half-width {
        width: 100%;
      }
    }
  }
}
</style>

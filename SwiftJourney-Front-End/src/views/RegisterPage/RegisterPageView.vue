<template>
  <div class="root">
    <div class="left-area">
      <img src="../../assets/railway.svg" class="logo" />
      <p class="title">风行旅途</p>
    </div>
    <a-card class="card">
      <div class="card-title">
        <a-tooltip title="返回首页">
          <ArrowLeftOutlined @click="goToHomePage" class="card-title icon" />
        </a-tooltip>
        <p class="card-title text">注册</p>
      </div>
      <div>
        <a-input
          v-model:value="inputPhone"
          type="string"
          :allowClear="true"
          placeholder="手机号"
          class="input"
          :status="inputPhoneStatus"
          @input="checkPhoneNumber"
          @change="checkPhoneNumber"
        >
          <template #prefix>
            <TabletOutlined class="icon" />
          </template>
        </a-input>
        <p class="input-error" v-if="inputPhoneError">{{ phoneErrorMsg }}</p>
        <a-input
          v-model:value="inputNickName"
          type="string"
          :allowClear="true"
          placeholder="用户名"
          class="input"
          :status="inputNickNameStatus"
          @input="checkNickName"
          @change="checkNickName"
        >
          <template #prefix>
            <UserOutlined class="icon" />
          </template>
        </a-input>
        <p class="input-error" v-if="inputNickNameError">{{ nickNameErrorMsg }}</p>
        <a-tooltip
          title="密码长度应在8-20位之间，至少包含大小写字母、数字或特殊符号中的两种，且不能包含空格"
          placement="right"
        >
          <a-input-password
            v-model:value="inputPassword"
            type="string"
            :allowClear="true"
            placeholder="密码"
            class="input"
            @input="checkInput"
            @change="checkInput"
            :status="inputPasswordStatus"
          >
            <template #prefix>
              <LockOutlined class="icon" />
            </template>
          </a-input-password>
        </a-tooltip>
        <p class="input-error" v-if="inputPasswordError">{{ passwordErrorMsg }}</p>
        <a-input-password
          v-model:value="inputConfirmPassword"
          type="string"
          :allowClear="true"
          placeholder="确认密码"
          class="input"
          @input="checkInput"
          @change="checkInput"
          :status="inputConfirmPasswordStatus"
        >
          <template #prefix>
            <LockOutlined class="icon" />
          </template>
        </a-input-password>
        <p class="input-error" v-if="inputConfirmPasswordError">{{ confirmPasswordErrorMsg }}</p>
        <a-input
          v-model:value="inputName"
          type="string"
          :allowClear="true"
          placeholder="真实姓名"
          class="input"
        >
          <template #prefix>
            <TagOutlined class="icon" />
          </template>
        </a-input>
        <a-input
          v-model:value="inputIdNumber"
          type="string"
          :allowClear="true"
          placeholder="身份证号"
          class="input"
          :status="inputIdNumberStatus"
          @input="checkIdNumber"
          @change="checkIdNumber"
        >
          <template #prefix>
            <IdcardOutlined class="icon" />
          </template>
        </a-input>
        <p class="input-error" v-if="inputIdNumberError">{{ idNumberErrorMsg }}</p>
      </div>

      <div>
        <a-button @click="goToLoginPage" class="button left">登录</a-button>
        <a-button
          type="primary"
          @click="postRegisterMsg"
          class="button right"
          :disabled="disableRegister"
          >注册</a-button
        >
      </div>
    </a-card>
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

const inputPhone = ref('')
const inputNickName = ref('')
const inputPassword = ref('')
const inputConfirmPassword = ref('')
const inputName = ref('')
const inputIdNumber = ref('')

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
  console.log(
    'post register message: ' +
      inputPhone.value +
      ' ' +
      inputPassword.value +
      ' ' +
      inputConfirmPassword.value,
  )
  await userApi.userRegister({ phone: inputPhone.value, password: inputPassword.value }).then((res) => {
    if (res.status === 200) {
      message.success('注册成功')
      router.push({ name: 'login' })
    } else if (res.status === 13001) {
      message.error('身份证号格式错误')
    } else if (res.status === 15001) {
      message.error('该手机号对应的用户已经存在')
    } else if (res.status === 15003) {
      message.error('用户名格式错误')
    } else if (res.status === 15004) {
      message.error('密码格式错误')
    } else if (res.status === 15005) {
      message.error('姓名格式错误')
    } 
  })
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

<style lang="css" scoped>
/* 整体背景颜色 */
body {
  background-color: #f0f2f5;
}

.root {
  display: grid;
  grid-template-columns: 1fr 1fr;
}

/* 左侧元素样式 */
.left-area {
  align-self: center;
  margin-left: 40%;
}

.logo {
  width: 100px;
  height: 100px;
}

.title {
  font-size: 48px;
  font-weight: bold;
  color: #333;
  margin-bottom: 20px;
}

/* 卡片容器的样式 */
.card {
  width: 400px;
  padding: 20px;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  background-color: white;
  margin-left: 5%;
}

.card-title {
  display: flex;
  align-items: center;
  margin-bottom: 16px;
}

.card-title.icon {
  font-size: 16px;
  margin-right: 12px;
}

.card-title.text {
  font-size: 24px;
  font-weight: bold;
  color: #333;
}

/* 输入框和标签的样式 */
.input {
  width: 100%;
  height: 40px;
  font-size: 16px;
  margin-bottom: 20px;
  .icon {
    margin-right: 8px;
  }
}

.input-error {
  color: #ff4d4f;
  font-size: 10px;
  margin-top: -16px;
  margin-left: 10px;
}

/* 设置按钮样式 */
.button {
  width: 45%;
  height: 40px;
  font-size: 16px;
  margin-top: 16px;
}
.button.left {
  margin-right: 5%;
}
.button.right {
  margin-left: 5%;
}
</style>

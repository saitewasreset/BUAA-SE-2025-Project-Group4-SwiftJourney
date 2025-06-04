<template>
  <div class="root">
    <div class="left-area">
      <img src="../../assets/railway.svg" class="logo" />
      <p class="title">风行旅途</p>
    </div>
    <a-card class="card">
      <div class="card-title">
        <!-- <a-tooltip title="返回首页">
          <ArrowLeftOutlined @click="goToHomePage" class="card-title icon" />
        </a-tooltip> -->
        <p class="card-title text">登录</p>
      </div>
      <div>
        <a-input
          v-model:value="inputPhone"
          type="string"
          :allowClear="true"
          placeholder="手机号"
          class="input"
        >
          <template #prefix>
            <UserOutlined class="icon" />
          </template>
        </a-input>
        <a-input-password
          v-model:value="inputPassword"
          type="string"
          :allowClear="true"
          placeholder="密码"
          class="input"
        >
          <template #prefix>
            <LockOutlined class="icon" />
          </template>
        </a-input-password>
      </div>

      <div>
        <a-button @click="goToRegisterPage" class="button left">注册</a-button>
        <a-button type="primary" @click="postLoginMsg" class="button right">登录</a-button>
      </div>
    </a-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { UserOutlined, LockOutlined, ArrowLeftOutlined } from '@ant-design/icons-vue'
import { useRouter } from 'vue-router';
import { message } from 'ant-design-vue';
import { userApi } from '@/api/UserApi/userApi';
import { useUserStore } from '@/stores/user';
import type { AxiosResponse } from 'axios';
import type { UserApiResponseData } from '@/interface/userInterface';

const inputPhone = ref('')
const inputPassword = ref('')

// -------------------- 路由相关 --------------------

const router = useRouter()

function goToHomePage() {
  router.push({ name: 'homepage' })
}

function goToRegisterPage() {
  router.push({ name: 'register' })
}

// -------------------- 处理登录逻辑 --------------------

async function postLoginMsg() {
  
  const params: Object = {
    phone: inputPhone.value,
    password: inputPassword.value
  };

  const res: UserApiResponseData = (await userApi.userLogin(params)).data;
  if (res.code === 200) {
    const nowUser = useUserStore();
    await nowUser.restoreUserFromCookie(router);
    message.success('登录成功');
    goToHomePage();
  } else  {
    message.error('登录失败，请检查手机号和密码是否正确');
  }
}

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

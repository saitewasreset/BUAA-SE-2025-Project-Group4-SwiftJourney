<template>
  <div class="root">
    <div class="left-area">
      <div class="brand-section">
        <div class="logo-container">
          <img src="../../assets/railway.svg" class="logo" />
          <div class="logo-glow"></div>
        </div>
        <h1 class="title">风行旅途</h1>
        <p class="subtitle">探索未知，享受旅程</p>
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
          <h2 class="card-title">欢迎回来</h2>
          <p class="card-subtitle">请登录您的账户</p>
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
            >
              <template #prefix>
                <UserOutlined class="input-icon" />
              </template>
            </a-input>
          </div>
          
          <div class="input-group">
            <label class="input-label">密码</label>
            <a-input-password
              v-model:value="inputPassword"
              type="string"
              :allowClear="true"
              placeholder="请输入密码"
              class="input"
              size="large"
            >
              <template #prefix>
                <LockOutlined class="input-icon" />
              </template>
            </a-input-password>
          </div>
        </div>

        <div class="button-section">
          <a-button 
            type="primary" 
            @click="postLoginMsg" 
            class="login-button"
            size="large"
            :loading="isLoading"
          >
            登录
          </a-button>
          
          <div class="register-section">
            <span class="register-text">还没有账户？</span>
            <a @click="goToRegisterPage" class="register-link">立即注册</a>
          </div>
        </div>
      </a-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { UserOutlined, LockOutlined, ArrowLeftOutlined } from '@ant-design/icons-vue'
import { useRouter } from 'vue-router'
import { message } from 'ant-design-vue'
import { userApi } from '@/api/UserApi/userApi'
import { useUserStore } from '@/stores/user'
import type { AxiosResponse } from 'axios'
import type { UserApiResponseData } from '@/interface/userInterface'

const inputPhone = ref('')
const inputPassword = ref('')
const isLoading = ref(false)

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
  if (!inputPhone.value || !inputPassword.value) {
    message.warning('请填写完整信息')
    return
  }
  
  isLoading.value = true
  
  try {
    const params: Object = {
      phone: inputPhone.value,
      password: inputPassword.value,
    }

    const res: UserApiResponseData = (await userApi.userLogin(params)).data
    if (res.code === 200) {
      const nowUser = useUserStore()
      await nowUser.restoreUserFromCookie(router).then(() => {
        message.success('登录成功')
        goToHomePage()
      })
    } else {
      message.error('登录失败，请检查手机号和密码是否正确')
    }
  } catch (error) {
    message.error('网络错误，请稍后重试')
  } finally {
    isLoading.value = false
  }
}
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
}

.left-area {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  padding: 1.5rem; // 从 2rem 改为 1.5rem，与注册页保持一致
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
  padding: 1.5rem; // 从 2rem 改为 1.5rem，与注册页保持一致
  position: relative;
  z-index: 2;
}

.card {
  width: 480px;
  padding: 3rem 1.5rem; // 左右内边距从 2.5rem 改为 1.5rem，与注册页保持一致
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
    transform: translateY(-8px);
    box-shadow: 
      0 40px 80px rgba(0, 0, 0, 0.2),
      0 20px 40px rgba(0, 0, 0, 0.15);
  }
  
  .card-header {
    text-align: center;
    margin-bottom: 2.5rem;
    
    .card-title {
      font-size: 2rem;
      font-weight: 700;
      color: $text-primary;
      margin: 0 0 0.5rem 0;
      background: linear-gradient(135deg, $primary-color, $secondary-color);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      background-clip: text;
    }
    
    .card-subtitle {
      font-size: 1rem;
      color: $text-secondary;
      margin: 0;
      font-weight: 400;
    }
  }
  
  .form-section {
    .input-group {
      margin-bottom: 1.5rem;
      
      .input-label {
        display: block;
        font-size: 0.9rem;
        font-weight: 600;
        color: $text-primary;
        margin-bottom: 0.5rem;
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
    
    .forgot-password {
      text-align: right;
      margin-bottom: 2rem;
      
      .forgot-link {
        color: $primary-color;
        text-decoration: none;
        font-size: 0.9rem;
        font-weight: 500;
        transition: all 0.3s ease;
        
        &:hover {
          color: $secondary-color;
          text-decoration: underline;
        }
      }
    }
  }
  
  .button-section {
    .login-button {
      width: 100%;
      height: 52px;
      border-radius: 12px;
      background: $primary-gradient;
      border: none;
      font-size: 1.1rem;
      font-weight: 600;
      letter-spacing: 1px;
      transition: all 0.3s ease;
      box-shadow: 0 8px 16px rgba($primary-color, 0.3);
      
      &:hover {
        transform: translateY(-2px);
        box-shadow: 0 8px 25px rgba(102, 126, 234, 0.4);
        background: linear-gradient(135deg, #7c8ef0 0%, #8557a8 100%);
      }
      
      &:active {
        transform: translateY(0);
      }
    }
    
    .register-section {
      text-align: center;
      margin-top: 2rem;
      
      .register-text {
        color: $text-secondary;
        font-size: 0.95rem;
        margin-right: 0.5rem;
      }
      
      .register-link {
        color: $primary-color;
        text-decoration: none;
        font-weight: 600;
        font-size: 0.95rem;
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
      padding: 0.8rem; // 从 1rem 改为 0.8rem，与注册页保持一致
    }
    
    .card {
      width: 100%;
      max-width: 400px;
    }
  }
}

@media (max-width: 480px) {
  .card {
    padding: 2rem 1rem; // 左右内边距从 1.5rem 改为 1rem，与注册页保持一致
    border-radius: 16px;
    
    .card-header .card-title {
      font-size: 1.6rem;
    }
  }
}
</style>

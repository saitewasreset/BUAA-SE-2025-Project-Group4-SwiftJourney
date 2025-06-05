<template>
  <div class="security-container">
    <div class="security-card">
      <!-- 头部标题区域 -->
      <div class="card-header">
        <div class="header-content">
          <h1 class="page-title">账户安全</h1>
          <p class="page-subtitle">保护您的账户和支付安全</p>
        </div>
      </div>

      <!-- 分割线 -->
      <div class="divider"></div>

      <!-- 表单内容区域 -->
      <div class="form-content">
        <div class="security-sections">
          
          <!-- 登录密码区域 -->
          <div 
            class="security-section"
            :class="{ 'section-hidden': isSetPayPassword }"
            v-show="!isSetPayPassword"
          >
            <div class="section-header">
              <div class="section-title">
                <i class="el-icon-key section-icon"></i>
                <span>登录密码</span>
              </div>
            </div>
            
            <div v-if="!isSetPassword" class="security-item">
              <div class="item-info">
                <label class="item-label">密码</label>
                <div class="item-value">••••••••</div>
              </div>
              <el-button class="edit-btn" type="primary" @click="setPassword">
                修改密码
              </el-button>
            </div>
            
            <div v-else class="edit-form">
              <div class="form-item">
                <label class="form-label">原密码</label>
                <el-input 
                  type="password" 
                  v-model="passwordFormData.originPassword" 
                  maxlength="20" 
                  show-password
                  placeholder="请输入原密码"
                />
              </div>
              
              <div class="form-item">
                <label class="form-label">新密码</label>
                <el-input 
                  type="password" 
                  v-model="passwordFormData.newPassword" 
                  maxlength="20" 
                  show-password
                  placeholder="请输入新密码"
                />
              </div>
              
              <div class="form-item">
                <label class="form-label">确认密码</label>
                <el-input 
                  type="password" 
                  v-model="passwordFormData.confirmPassword" 
                  maxlength="20" 
                  show-password
                  placeholder="请再次输入新密码"
                />
              </div>
              
              <div class="form-hint">
                <i class="el-icon-info"></i>
                密码长度8-20位，需包含大小写字母、数字或特殊符号至少三种
              </div>
              
              <div class="form-actions">
                <el-button class="cancel-btn" @click="isSetPasswordCancel">取消</el-button>
                <el-button class="save-btn" type="primary" @click="isSetPasswordSave">保存</el-button>
              </div>
            </div>
          </div>

          <!-- 支付密码区域 -->
          <div 
            class="security-section"
            :class="{ 'section-hidden': isSetPassword }"
            v-show="!isSetPassword"
          >
            <div class="section-header">
              <div class="section-title">
                <i class="el-icon-wallet section-icon"></i>
                <span>支付密码</span>
              </div>
            </div>
            
            <div v-if="!isSetPayPassword" class="security-item">
              <div class="item-info">
                <label class="item-label">支付密码</label>
                <div class="item-value">{{ setedPayPassword }}</div>
              </div>
              <el-button class="edit-btn" type="primary" @click="setPayPassword">
                {{ setedPayPassword === '已设置' ? '修改密码' : '设置密码' }}
              </el-button>
            </div>
            
            <div v-else class="edit-form">
              <div class="form-item">
                <label class="form-label">账户密码</label>
                <el-input 
                  type="password" 
                  v-model="payPasswordFormData.password" 
                  maxlength="20" 
                  show-password
                  placeholder="请输入账户密码"
                />
              </div>
              
              <div class="form-item">
                <label class="form-label">支付密码</label>
                <div class="pin-input-container">
                  <div class="pin-input-group">
                    <input 
                      v-for="(digit, index) in digits" 
                      :key="`pin-${index}`"
                      class="pin-digit" 
                      type="password"
                      inputmode="numeric"
                      maxlength="1" 
                      v-model="digits[index]" 
                      @input="onInput(index)" 
                      @keydown="onKeyDown($event, index)"
                      @focus="onFocus()"
                    />
                  </div>
                </div>
              </div>
              
              <div class="form-item">
                <label class="form-label">确认密码</label>
                <div class="pin-input-container">
                  <div class="pin-input-group">
                    <input 
                      v-for="(digit, index) in confirmDigits" 
                      :key="`confirm-${index}`"
                      class="pin-digit confirm-digit" 
                      type="password"
                      inputmode="numeric"
                      maxlength="1" 
                      v-model="confirmDigits[index]" 
                      @input="confirmOnInput(index)" 
                      @keydown="confirmOnKeyDown($event, index)"
                      @focus="confirmOnFocus()"
                    />
                  </div>
                </div>
              </div>
              
              <div class="form-hint">
                <i class="el-icon-info"></i>
                支付密码为6位数字
              </div>
              
              <div class="form-actions">
                <el-button class="cancel-btn" @click="isSetPayPasswordCancel">取消</el-button>
                <el-button class="save-btn" type="primary" @click="isSetPayPasswordSave">保存</el-button>
              </div>
            </div>
          </div>

        </div>
      </div>
    </div>
  </div>
</template>
  
<script>
import { ElMessage } from 'element-plus';
import { paymentApi } from '@/api/PaymentApi/paymentApi'
import { userApi } from '@/api/UserApi/userApi'
import { useUserStore } from '@/stores/user';

const user = useUserStore();

export default{
    data() {
        return {
            passwordFormData: {
                originPassword: "",
                newPassword: "",
                confirmPassword: "",
            },
            payPasswordFormData: {
                password: "",
                newPayPassword: "",
            },
            setedPayPassword: user.havePaymentPasswordSet ? '已设置' : '未设置',
            isSetPassword: false,
            isSetPayPassword: false,
            digits: new Array(6).fill(''),
            confirmDigits:  new Array(6).fill(''),
        }
    },
    methods: {
        setPassword() {
            this.isSetPassword = !this.isSetPassword;
        },
        setPayPassword() {
            this.isSetPayPassword = !this.isSetPayPassword;
        },
        isSetPasswordCancel() {
            this.isSetPassword = false;
            this.passwordFormData.originPassword = "";
            this.passwordFormData.newPassword = "";
            this.passwordFormData.confirmPassword = "";
        },
        isSetPasswordSave(){
            if(!this.checkPassword()){
                return;
            }
            this.updatePassword();
        },
        async updatePassword() {
            await userApi.updatePassword({originPassword: this.passwordFormData.originPassword, newPassword: this.passwordFormData.newPassword})
            .then((res) => {
                if(res.status == 200) {
                    if(res.data.code == 200)  {
                        this.successUpdatePassword();
                    }
                    else if (res.data.code == 403) {
                        ElMessage.error('会话无效');
                    } else if (res.data.code == 15002) {
                        ElMessage.error('密码错误');
                    }
                }
            }) .catch ((error) => {
                ElMessage.error(error);
            })
        },
        successUpdatePassword() {
            this.isSetPassword = false;
            this.passwordFormData.originPassword = "";
            this.passwordFormData.newPassword = "";
            this.passwordFormData.confirmPassword = "";
        },
        checkPassword(){
            if(this.passwordFormData.newPassword.includes(' ')){
                ElMessage.error('密码不能包含空格');
                return false;
            }
            if(this.passwordFormData.newPassword.length < 8 || this.passwordFormData.newPassword.length > 20){
                ElMessage.error('密码长度应在 8 - 20 位之间');
                return false;
            }
            let matchedTypes = 0
            if (/[a-z]/.test(this.passwordFormData.newPassword)) {
                matchedTypes += 1
            }
            if (/[A-Z]/.test(this.passwordFormData.newPassword)) {
                matchedTypes += 1
            }
            if (/\d/.test(this.passwordFormData.newPassword)) {
                matchedTypes += 1
            }
            // 特殊符号的正则表达式
            if (/[\W_]/.test(this.passwordFormData.newPassword)) {
                matchedTypes += 1
             }
            if (matchedTypes < 3) {
                ElMessage.error('密码必须包含大小写字母、数字或特殊符号至少三种');
                return false;
            }
            if (this.passwordFormData.newPassword != this.passwordFormData.confirmPassword) {
                ElMessage.error('两次输入的密码不一致');
                return false;
            }
            return true;
        },
        isSetPayPasswordCancel() {
            this.isSetPayPassword = false;
            this.payPasswordFormData.password = "";
            this.payPasswordFormData.newPayPassword = "";
            for(let i = 0; i < 6; i++){
                this.digits[i] = "";
                this.confirmDigits[i] = "";
            }
        }, 
        async isSetPayPasswordSave() {
            if(!this.checkPayPassword()){
                return;
            }
            this.payPasswordFormData.newPayPassword = this.digits.join('');

            await paymentApi.setPaymentPassword({userPassword: this.payPasswordFormData.password, paymentPassword: this.payPasswordFormData.newPayPassword})
            .then((res) => {
                if (res.status == 200) {
                    if(res.data.code == 200) {
                        ElMessage.success('支付密码设置成功');
                        this.isSetPayPassword = false;
                        this.payPasswordFormData.password = "";
                        this.payPasswordFormData.newPayPassword = "";
                        for(let i = 0; i < 6; i++){
                            this.digits[i] = "";
                            this.confirmDigits[i] = "";
                        }
                        this.setedPayPassword = "已设置";
                    } else if(res.data.code == 403) {
                        ElMessage.error('会话无效');
                    } else if(res.data.code == 11002) {
                        ElMessage.error('用户密码错误');
                    } else if(res.data.code == 11007) {
                        ElMessage.error('支付密码格式错误');
                    }
                }
            })
            .catch((error) => {
                ElMessage.error(error);
            })
        },
        checkPayPassword() {
            for(let i = 0; i < 6; i++){
                if(!/^\d$/.test(this.digits[i])){
                    ElMessage.error('支付密码只能是数字');
                    return false;
                }
            }
            for(let i = 0; i < 6; i++){
                if(this.digits[i] != this.confirmDigits[i]){
                    ElMessage.error('两次输入的支付密码不同');
                    return false;
                }
            }
            return true;
        },
        // 支付密码输入框焦点控制
        onFocus(index) {
            // 找到第一个为空的输入框位置
            const firstEmptyIndex = this.digits.findIndex(digit => digit === '');
            
            // 如果点击的不是第一个空位，则强制聚焦到第一个空位
            if (firstEmptyIndex !== -1 && index !== firstEmptyIndex) {
                this.$nextTick(() => {
                    const inputs = document.querySelectorAll('.pin-digit:not(.confirm-digit)');
                    if (inputs[firstEmptyIndex]) {
                        inputs[firstEmptyIndex].focus();
                    }
                });
            }
        },
        onInput(index) {
          // 确保输入的是数字
          const value = this.digits[index];
          if (value && !/^\d$/.test(value)) {
            this.digits[index] = '';
            return;
          }

          // 输入完成后自动跳转到下一格
          if (index < this.digits.length - 1 && value.length === 1) {
            this.$nextTick(() => {
              const inputs = document.querySelectorAll('.pin-digit:not(.confirm-digit)');
              inputs[index + 1].focus();
            });
          }
        },
        onKeyDown(event, index) {
          // 处理退格键
          if (event.key === 'Backspace') {
            if (this.digits[index] === '' && index > 0) {
              // 如果当前格为空且不是第一格，则删除前一格的内容并聚焦到前一格
              this.digits[index - 1] = '';
              this.$nextTick(() => {
                const inputs = document.querySelectorAll('.pin-digit:not(.confirm-digit)');
                inputs[index - 1].focus();
              });
            }
          }
          // 阻止非数字键的输入
          else if (!/\d/.test(event.key) && !['Backspace', 'Delete', 'Tab', 'ArrowLeft', 'ArrowRight'].includes(event.key)) {
            event.preventDefault();
          }
        },
        // 确认支付密码输入框焦点控制
        confirmOnFocus(index) {
          // 找到第一个为空的确认密码输入框位置
          const firstEmptyIndex = this.confirmDigits.findIndex(digit => digit === '');
          
          // 如果点击的不是第一个空位，则强制聚焦到第一个空位
          if (firstEmptyIndex !== -1 && index !== firstEmptyIndex) {
            this.$nextTick(() => {
              const inputs = document.querySelectorAll('.confirm-digit');
              if (inputs[firstEmptyIndex]) {
                inputs[firstEmptyIndex].focus();
              }
            });
          }
        },
        confirmOnInput(index) {
          // 确保输入的是数字
          const value = this.confirmDigits[index];
          if (value && !/^\d$/.test(value)) {
            this.confirmDigits[index] = '';
            return;
          }

          // 输入完成后自动跳转到下一格
          if (index < this.confirmDigits.length - 1 && value.length === 1) {
            this.$nextTick(() => {
              const inputs = document.querySelectorAll('.confirm-digit');
              inputs[index + 1].focus();
            });
          }
        },
        confirmOnKeyDown(event, index) {
          // 处理退格键
          if (event.key === 'Backspace') {
            if (this.confirmDigits[index] === '' && index > 0) {
              // 如果当前格为空且不是第一格，则删除前一格的内容并聚焦到前一格
              this.confirmDigits[index - 1] = '';
              this.$nextTick(() => {
                const inputs = document.querySelectorAll('.confirm-digit');
                inputs[index - 1].focus();
              });
            }
          }
          // 阻止非数字键的输入
          else if (!/\d/.test(event.key) && !['Backspace', 'Delete', 'Tab', 'ArrowLeft', 'ArrowRight'].includes(event.key)) {
            event.preventDefault();
          }
        },
    }
}
</script>
  
<style scoped>
/* 容器样式 */
.security-container {
  min-height: 50vh;
  padding: 20px;
  display: flex;
  justify-content: center;
}

/* 主卡片 */
.security-card {
  width: 100%;
  max-width: 900px;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border-radius: 24px;
  box-shadow: 
    0 20px 40px rgba(0, 0, 0, 0.1),
    0 0 0 1px rgba(255, 255, 255, 0.2);
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.security-card:hover {
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
  background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
}

.header-content {
  flex: 1;
}

.page-title {
  font-size: 32px;
  font-weight: 700;
  color: #1a202c;
  margin: 0 0 8px 0;
  letter-spacing: -0.5px;
}

.page-subtitle {
  font-size: 16px;
  color: #64748b;
  margin: 0;
  font-weight: 400;
}

.header-icon {
  margin-left: 24px;
}

.icon-circle {
  width: 80px;
  height: 80px;
  background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
  border-radius: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 32px;
  box-shadow: 0 8px 24px rgba(245, 158, 11, 0.3);
}

/* 分割线 */
.divider {
  height: 1px;
  background: linear-gradient(90deg, transparent, #e2e8f0, transparent);
  margin: 0 40px;
}

/* 表单内容 */
.form-content {
  padding: 40px;
}

.security-sections {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

/* 隐藏状态的区域 */
.section-hidden {
  opacity: 0.3;
  pointer-events: none;
  transform: scale(0.95);
}

/* 安全区域 */
.security-section {
  background: #f8fafc;
  border-radius: 16px;
  padding: 24px;
  border: 1px solid #e2e8f0;
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

.security-section:hover:not(.section-hidden) {
  border-color: #cbd5e1;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
}

.section-header {
  margin-bottom: 20px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 18px;
  font-weight: 600;
  color: #374151;
}

.section-icon {
  font-size: 20px;
  color: #667eea;
}

/* 安全项目 */
.security-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: white;
  border-radius: 12px;
  border: 1px solid #e5e7eb;
}

.item-info {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 16px;
}

.item-label {
  font-size: 15px;
  font-weight: 600;
  color: #374151;
  min-width: 80px;
}

.item-value {
  font-size: 15px;
  color: #6b7280;
  font-family: monospace;
}

.edit-btn {
  padding: 8px 20px;
  border-radius: 10px;
  font-weight: 500;
  font-size: 14px;
  transition: all 0.2s;
}

.edit-btn:hover {
  transform: translateY(-1px);
}

/* 编辑表单 */
.edit-form {
  background: white;
  border-radius: 12px;
  padding: 24px;
  border: 1px solid #e5e7eb;
}

.form-item {
  margin-bottom: 20px;
}

.form-label {
  display: block;
  font-size: 15px;
  font-weight: 600;
  color: #374151;
  margin-bottom: 8px;
}

/* 输入框样式 */
.form-item .el-input :deep(.el-input__wrapper) {
  border-radius: 10px;
  border: 2px solid #e5e7eb;
  background-color: #f9fafb;
  transition: all 0.3s;
  box-shadow: none;
}

.form-item .el-input:not(.is-disabled) :deep(.el-input__wrapper):hover {
  border-color: #d1d5db;
  background-color: #fff;
}

.form-item .el-input.is-focused :deep(.el-input__wrapper) {
  border-color: #667eea;
  background-color: #fff;
  box-shadow: 0 0 0 4px rgba(102, 126, 234, 0.1);
}

.form-item .el-input :deep(.el-input__inner) {
  font-size: 15px;
  color: #374151;
  font-weight: 500;
  padding: 12px 16px;
}

/* PIN 输入框 */
.pin-input-container {
  display: flex;
  justify-content: flex-start;
}

.pin-input-group {
  display: flex;
  gap: 12px;
}

.pin-digit {
  width: 48px;
  height: 48px;
  font-size: 18px;
  font-weight: 600;
  text-align: center;
  border: 2px solid #e5e7eb;
  border-radius: 10px;
  outline: none;
  background-color: #f9fafb;
  transition: all 0.3s;
  color: #374151;
}

.pin-digit:focus {
  border-color: #667eea;
  background-color: #fff;
  box-shadow: 0 0 0 4px rgba(102, 126, 234, 0.1);
}

.pin-digit:hover {
  border-color: #d1d5db;
  background-color: #fff;
}

/* 提示信息 */
.form-hint {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #6b7280;
  font-size: 14px;
  background: #f3f4f6;
  padding: 12px 16px;
  border-radius: 8px;
  margin-bottom: 24px;
}

.form-hint i {
  color: #9ca3af;
}

/* 操作按钮 */
.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.cancel-btn,
.save-btn {
  padding: 10px 24px;
  border-radius: 10px;
  font-weight: 600;
  font-size: 14px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.cancel-btn {
  background: #f3f4f6;
  color: #6b7280;
  border: 1px solid #e5e7eb;
}

.cancel-btn:hover {
  background: #e5e7eb;
  color: #374151;
  transform: translateY(-1px);
}

.save-btn {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
}

.save-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .security-container {
    padding: 20px 16px;
  }

  .security-card {
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
    width: 64px;
    height: 64px;
    font-size: 24px;
  }

  .page-title {
    font-size: 28px;
  }

  .form-content {
    padding: 32px 24px;
  }

  .security-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
  }

  .item-info {
    width: 100%;
  }

  .edit-btn {
    align-self: flex-start;
  }

  .form-actions {
    flex-direction: column-reverse;
    gap: 12px;
  }

  .cancel-btn,
  .save-btn {
    width: 100%;
    justify-content: center;
  }

  .pin-input-group {
    gap: 8px;
  }

  .pin-digit {
    width: 40px;
    height: 40px;
    font-size: 16px;
  }
}

@media (max-width: 480px) {
  .security-container {
    padding: 16px 12px;
  }

  .card-header {
    padding: 24px 20px 16px;
  }

  .page-title {
    font-size: 24px;
  }

  .form-content {
    padding: 24px 20px;
  }

  .security-section {
    padding: 20px 16px;
  }

  .edit-form {
    padding: 20px 16px;
  }

  .divider {
    margin: 0 20px;
  }
}

/* 动画效果 */
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

.security-section {
  animation: fadeInUp 0.6s cubic-bezier(0.4, 0, 0.2, 1) forwards;
}

.security-section:nth-child(1) { animation-delay: 0.1s; }
.security-section:nth-child(2) { animation-delay: 0.2s; }

/* 编辑状态过渡动画 */
.edit-form {
  animation: fadeInUp 0.4s ease-out;
}
</style>
<template>
    <div class="form-container">
      <div v-if="!isSetPassword" class="form-row">
        <label for="password">密码</label>
        <el-input type="text" class="text-input" id="password" placeholder="********" disabled />
        <el-button class="form-row-el-button" type="primary" plain @click="setPassword">修改密码</el-button>
      </div>
      <div v-else>
        <div class="form-row">
            <label for="originPassword">原密码</label>
            <el-input type="password" class="text-input" id="originPassword" v-model="passwordFormData.originPassword" maxlength="20" show-password/>
        </div>
        <div class="form-row">
            <label for="newPassword">新密码</label>
            <el-input type="password" class="text-input" id="newPassword" v-model="passwordFormData.newPassword" maxlength="20" show-password/>
        </div>
        <div class="form-row">
            <label for="confirmPassword">确认密码</label>
            <el-input type="password" class="text-input" id="confirmPassword" v-model="passwordFormData.confirmPassword" maxlength="20" show-password/>
        </div>
        <div class="button-row">
            <el-button class="final-el-button" type="primary" plain @click="isSetPasswordCancel">取消</el-button>
            <el-button class="final-el-button" type="primary" @click="isSetPasswordSave">保存</el-button>
        </div>
      </div>

      <div v-if="!isSetPayPassword" class="form-row">
        <label for="payPassword">支付密码</label>
        <el-input type="text" class="text-input" id="payPassword" v-model="setedPayPassword" disabled />
        <el-button class="form-row-el-button" type="primary" plain @click="setPayPassword">设置密码</el-button>
      </div>
      <div v-else>
        <div class="form-row">
            <label for="password">账户密码</label>
            <el-input type="password" class="text-input" id="password" v-model="payPasswordFormData.password" maxlength="20" show-password/>
        </div>
        <div class="form-row">
            <label for="newPassword">支付密码</label>
            <input v-for="(digit, index) in digits" :key="index" class="captcha-digit" type="text" maxlength="1" v-model="digits[index]" @input="onInput(index)" @keydown="onKeyDown"/>
        </div>
        <div class="form-row">
            <label for="confirmPassword">确认密码</label>
            <input v-for="(digit, index) in digits" :key="index" class="captcha-confirm-digit" type="text" maxlength="1" v-model="confirmDigits[index]" @input="confirmOnInput(index)" @keydown="confirmOnKeyDown"/>
        </div>
        <div class="button-row">
            <el-button class="final-el-button" type="primary" plain @click="isSetPayPasswordCancel">取消</el-button>
            <el-button class="final-el-button" type="primary" @click="isSetPayPasswordSave">保存</el-button>
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
                    this.successUpdatePassword();
                } else if (res.status == 403) {
                    ElMessage.error('会话无效');
                } else if (res.status == 15002) {
                    ElMessage.error('密码错误');
                } else {
                    throw new Error(res.statusText);
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
                if(res.status == 403) {
                    ElMessage.error('账户密码错误');
                    return;
                } else if (res.status == 200) {
                    ElMessage.success('支付密码设置成功');
                    this.isSetPayPassword = false;
                    this.payPasswordFormData.password = "";
                    this.payPasswordFormData.newPayPassword = "";
                    for(let i = 0; i < 6; i++){
                        this.digits[i] = "";
                        this.confirmDigits[i] = "";
                    }
                    this.setedPayPassword = "已设置";
                } else {
                    ElMessage.error(res.status + res.data)
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
        onInput(index) {
            if (index < this.digits.length - 1 && this.digits[index].length === 1) {
                document.querySelectorAll('.captcha-digit')[index + 1].focus();
            }
        },
        onKeyDown(event) {
            if (event.key === 'Backspace' && event.target.value === '') {
                const index = Array.from(document.querySelectorAll('.captcha-digit')).indexOf(event.target);
                if (index > 0) {
                    document.querySelectorAll('.captcha-digit')[index - 1].focus();
                }
            }
        },
        confirmOnInput(index) {
            if (index < this.confirmDigits.length - 1 && this.confirmDigits[index].length === 1) {
                document.querySelectorAll('.captcha-confirm-digit')[index + 1].focus();
            }
        },
        confirmOnKeyDown(event) {
            if (event.key === 'Backspace' && event.target.value === '') {
                const index = Array.from(document.querySelectorAll('.captcha-confirm-digit')).indexOf(event.target);
                if (index > 0) {
                    document.querySelectorAll('.captcha-confirm-digit')[index - 1].focus();
                }
            }
        },
    }
}
</script>
  
  <style scoped>
  /* 全局样式 */
  .form-container {
    max-width: 500px;
    margin-top: 1%;
    margin-left: 3%;
    padding: 20px;
  }
  
  /* 行布局 */
  .form-row {
    display: flex;
    margin-bottom: 15px;
    align-items: center;
    justify-content: flex-start;
    gap: 15px;
  }
  
  .form-row label {
    width: 100px;
    font-size: 16px; /* 增加字体大小 */
    color: #333;
    text-align: center; /* 文字居中对齐 */
  }
  
  .text-input {
    font-size: 16px;
    max-width: 200px;
  }


  .form-row-el-button {
    font-size: 16px;
  }
  
  .button-row {
    margin-top: 20px;
    margin-bottom: 20px;
    display: flex;
    justify-content: flex-start;
    align-items: center;
    padding-left: 175px;
  }

  .final-el-button {
    font-size: 16px;
  }
  
  /* 响应式布局 */
  @media (max-width: 600px) {
    .form-row {
      flex-direction: column;
    }
    .form-row label {
      margin-right: 0;
      margin-bottom: 5px;
      text-align: left; /* 在小屏幕上文字靠左对齐 */
    }
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

.captcha-confirm-digit {
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

.captcha-confirm-digit:focus {
  border-color: #007bff;
}
  </style>
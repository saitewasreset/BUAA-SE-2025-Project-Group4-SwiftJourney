<template>
  <div class="personal-data-container">
    <div class="profile-card">
      <!-- 头部标题区域 -->
      <div class="card-header">
        <div class="header-content">
          <h1 class="page-title">个人资料</h1>
          <p class="page-subtitle">管理您的个人信息和账户设置</p>
        </div>
      </div>

      <!-- 分割线 -->
      <div class="divider"></div>

      <!-- 表单内容区域 -->
      <div class="form-content">
        <div class="form-grid">
          <!-- 用户名 -->
          <div class="form-item">
            <div class="item-header">
              <label class="item-label">用户名</label>
              <el-button 
                class="edit-btn" 
                :type="isSetUsername ? 'warning' : ''" 
                :icon="isSetUsername ? 'Close' : 'Edit'"
                size="small" 
                text 
                @click="setUsername"
              >
                {{ setUsernameButtonText }}
              </el-button>
            </div>
            <div class="item-content">
              <el-input 
                v-model="username" 
                :disabled="!isSetUsername" 
                maxlength="16"
                :class="{ 'editing': isSetUsername }"
                placeholder="请输入用户名"
              />
            </div>
          </div>

          <!-- 姓名 -->
          <div class="form-item">
            <div class="item-header">
              <label class="item-label">姓名</label>
              <span class="readonly-tag">只读</span>
            </div>
            <div class="item-content">
              <el-input v-model="desensitizeName" disabled />
            </div>
          </div>

          <!-- 性别 -->
          <div class="form-item">
            <div class="item-header">
              <label class="item-label">性别</label>
              <el-button 
                class="edit-btn" 
                :type="isSetGender ? 'warning' : ''" 
                :icon="isSetGender ? 'Close' : 'Edit'"
                size="small" 
                text 
                @click="setGender"
              >
                {{ setGenderButtonText }}
              </el-button>
            </div>
            <div class="item-content">
              <el-input 
                v-model="gender" 
                :disabled="!isSetGender" 
                maxlength="1"
                :class="{ 'editing': isSetGender }"
                placeholder="请输入性别（男/女）"
              />
            </div>
          </div>

          <!-- 年龄 -->
          <div class="form-item">
            <div class="item-header">
              <label class="item-label">年龄</label>
              <el-button 
                class="edit-btn" 
                :type="isSetAge ? 'warning' : ''" 
                :icon="isSetAge ? 'Close' : 'Edit'"
                size="small" 
                text 
                @click="setAge"
              >
                {{ setAgeButtonText }}
              </el-button>
            </div>
            <div class="item-content">
              <el-input 
                v-model="age" 
                :disabled="!isSetAge" 
                maxlength="3"
                :class="{ 'editing': isSetAge }"
                placeholder="请输入年龄"
              />
            </div>
          </div>

          <!-- 身份证号 -->
          <div class="form-item">
            <div class="item-header">
              <label class="item-label">身份证号</label>
              <span class="readonly-tag">只读</span>
            </div>
            <div class="item-content">
              <el-input v-model="desensitizeIdentityCardId" disabled />
            </div>
          </div>

          <!-- 手机号 -->
          <div class="form-item">
            <div class="item-header">
              <label class="item-label">手机号</label>
              <span class="readonly-tag">只读</span>
            </div>
            <div class="item-content">
              <el-input v-model="desensitizePhone" disabled />
            </div>
          </div>

          <!-- 邮箱 -->
          <div class="form-item full-width">
            <div class="item-header">
              <label class="item-label">邮箱地址</label>
              <el-button 
                class="edit-btn" 
                :type="isSetEmail ? 'warning' : ''" 
                :icon="isSetEmail ? 'Close' : 'Edit'"
                size="small" 
                text 
                @click="setEmail"
              >
                {{ setEmailButtonText }}
              </el-button>
            </div>
            <div class="item-content">
              <el-input 
                v-model="email" 
                :disabled="!isSetEmail"
                :class="{ 'editing': isSetEmail }"
                placeholder="请输入邮箱地址"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- 操作按钮区域 -->
      <div class="card-footer" v-if="isSetAge || isSetEmail || isSetUsername || isSetGender">
        <div class="action-buttons">
          <el-button class="cancel-btn" @click="cancel">
            <i class="el-icon-close"></i>
            取消修改
          </el-button>
          <el-button class="save-btn" type="primary" @click="save">
            <i class="el-icon-check"></i>
            保存更改
          </el-button>
        </div>
      </div>
    </div>
  </div>
</template>


<script lang="ts">
import { ElMessage } from 'element-plus';
import { useUserStore } from '@/stores/user';
import { useDebugUserStore } from '@/stores/user';
import { userApi } from '@/api/UserApi/userApi'
import validator from 'validator';

interface UserUpdateInfo {
  username: string;
  gender?: "male" | "female";
  age?: number;
  email: string;
}

const user = useUserStore();
//const debugUser = useDebugUserStore();

  export default {
    data() {
      return {
        formTepData: {
          username: "",
          gender: "",
          age: "",
          email: "",
        },
        isSetUsername: false,
        isSetGender: false,
        isSetAge: false,
        isSetEmail: false,
        setUsernameButtonText: "设置",
        setGenderButtonText: "设置",
        setAgeButtonText: "设置",
        setEmailButtonText: "设置",
      };
    },
    computed: {
      desensitizeName() {
        const prefix = user.name.substring(0, 1);
        // return `${prefix}**`;
        return user.name;
      },
      desensitizeEmail() {
        if(user.email == null) {
          return '未设置';
        }
        const prefix = user.email.substring(0, 3);
        const atIndex = user.email.indexOf('@');
        if(atIndex != -1) {
          const suffix = user.email.substring(atIndex);
          return `${prefix} **** ${suffix}`;
        }
        return `***`
      },
      username: {
        get(){
          return this.isSetUsername ? this.formTepData.username : user.username;
        },
        set(value: string) {
          this.formTepData.username = value;
        }
      },
      gender: {
        get(){
          let gender;
          if(user.gender == 'female') {
            gender = '女';
          } else if (user.gender == 'male') {
            gender = '男';
          } else {
            gender = '未设置';
          }
          return this.isSetGender ? this.formTepData.gender : gender;
        },
        set(value: string) {
          this.formTepData.gender = value;
        }
      },
      age: {
        get(){
          let age = user.age ? user.age : '未设置';
          return this.isSetAge ? this.formTepData.age : age;
        },
        set(value: string) {
          this.formTepData.age = value;
        }
      },
      email: {
        get(){
          return this.isSetEmail ? this.formTepData.email : this.desensitizeEmail;
        },
        set(value: string) {
          this.formTepData.email = value;
        }
      },
      desensitizeIdentityCardId() {
        const prefix = user.identityCardId.substring(0, 6);
        const suffix = user.identityCardId.substring(user.identityCardId.length - 4);
        return `${prefix} ******** ${suffix}`;
      },
      desensitizePhone() {
        const prefix = user.phone.substring(0, 3);
        const suffix = user.phone.substring(user.phone.length - 2);
        return `${prefix} **** **${suffix}`;
      },
    },
    methods: {
      setUsername() {
        this.isSetUsername = !this.isSetUsername;
        this.setUsernameButtonText = this.isSetUsername ? "取消" : "设置";
        this.formTepData.username = "";
      },
      setGender() {
        this.isSetGender = !this.isSetGender;
        this.setGenderButtonText = this.isSetGender ? "取消" : "设置";
        this.formTepData.gender = "";
      },
      setAge() {
        this.isSetAge = !this.isSetAge;
        this.setAgeButtonText = this.isSetAge ? "取消" : "设置";
        this.formTepData.age = "";
      },
      setEmail() {
        this.isSetEmail = !this.isSetEmail;
        this.setEmailButtonText = this.isSetEmail ? "取消" : "设置";
        this.formTepData.email = "";
      },
      cancel() {
        if(this.isSetUsername) {
          this.setUsername();
        }
        if(this.isSetGender) {
          this.setGender();
        }
        if(this.isSetAge) {
          this.setAge();
        }
        if(this.isSetEmail){
          this.setEmail();
        }
      },
      async save() {
        if(!this.checkAll()){
          return;
        }

        this.postData();
      },
      checkAll(){
        if(this.isSetUsername && this.formTepData.username.trim() == "") {
          ElMessage.error('用户名不能为空');
          return false;
        }
        if(this.isSetGender && this.formTepData.gender != "") {
          if(this.formTepData.gender != "男" && this.formTepData.gender != "女"){
            ElMessage.error('性别只能为“男”或“女”')
            return false;
          }
        }
        if(this.isSetAge && this.formTepData.age != "") {
          if (!/^\d+$/.test(this.formTepData.age)) {
            ElMessage.error('请输入有效的数字作为年龄');
            return false;
          }
          const age = parseInt(this.formTepData.age, 10);
          if (age < 1 || age > 200) {
              ElMessage.error('年龄只能为1~200的数字');
              return false;
          }
        }
        if(this.isSetEmail && this.formTepData.email != ""){
          if(!validator.isEmail(this.formTepData.email)){
            ElMessage.error('请填入合法的邮箱地址');
            return false;
          }
        }
        if(user.email == null && !this.isSetEmail) {
          ElMessage.error('当前未设置邮箱，需先设置邮箱');
          return false;
        }
        return true;
      },
      async postData(){
        const formPostData: UserUpdateInfo = {
          username: this.isSetUsername ? this.formTepData.username : user.username,
          email: this.isSetEmail ? this.formTepData.email : user.email,
        }
        if(this.isSetAge) {
          formPostData.age = parseInt(this.formTepData.age);
        } else {
          formPostData.age = user.age;
        }
        if(this.isSetGender) {
          formPostData.gender = this.formTepData.gender == '男' ? 'male' : 'female';
        } else {
          formPostData.gender = user.gender;
        }

        await userApi.setUserInfo(formPostData)
          .then((res) =>{
            if(res.status == 200) {
              if(res.data.code == 200) {
                ElMessage.success('设置成功');

                if(this.isSetUsername) {
                  this.setUsername();
                  user.username = formPostData.username;
                }
                if(this.isSetGender) {
                  this.setGender();
                  user.gender = formPostData.gender;
                }
                if(this.isSetAge) {
                  this.setAge();
                  user.age = formPostData.age;
                }
                if(this.isSetEmail){
                  this.setEmail();
                  user.email = formPostData.email;
                }
              }  else if (res.data.code == 403) {
                ElMessage.error('会话无效');
              } else if (res.data.code == 15003) {
                ElMessage.error('用户名格式错误');
              } else if (res.data.code == 15006) {
                ElMessage.error('年龄格式错误');
              } else if (res.data.code == 15007) {
                ElMessage.error('邮箱格式错误');
              }
            }
          })
          .catch((error) => {
            ElMessage.error(error);
          })
      },
    },
  };
</script>

<style scoped>
/* 容器样式 */
.personal-data-container {
  min-height: 50vh;
  /* background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); */
  padding: 20px 20px;
  display: flex;
  /* align-items: center; */
  justify-content: center;
}

/* 主卡片 */
.profile-card {
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

.profile-card:hover {
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

.header-avatar {
  margin-left: 24px;
}

.avatar-circle {
  width: 80px;
  height: 80px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 32px;
  box-shadow: 0 8px 24px rgba(102, 126, 234, 0.3);
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

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 32px;
}

.form-item {
  position: relative;
}

.form-item.full-width {
  grid-column: 1 / -1;
}

.item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.item-label {
  font-size: 15px;
  font-weight: 600;
  color: #374151;
  letter-spacing: 0.025em;
}

.readonly-tag {
  font-size: 12px;
  color: #6b7280;
  background: #f3f4f6;
  padding: 4px 8px;
  border-radius: 8px;
  font-weight: 500;
}

.edit-btn {
  font-size: 12px;
  padding: 4px 12px;
  border-radius: 8px;
  font-weight: 500;
  transition: all 0.2s;
}

.edit-btn:hover {
  transform: translateY(-1px);
}

/* 输入框样式 */
.item-content .el-input {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.item-content .el-input :deep(.el-input__wrapper) {
  border-radius: 12px;
  border: 2px solid #e5e7eb;
  background-color: #f9fafb;
  transition: all 0.3s;
  box-shadow: none;
}

.item-content .el-input:not(.is-disabled) :deep(.el-input__wrapper):hover {
  border-color: #d1d5db;
  background-color: #fff;
}

.item-content .el-input.is-focused :deep(.el-input__wrapper) {
  border-color: #667eea;
  background-color: #fff;
  box-shadow: 0 0 0 4px rgba(102, 126, 234, 0.1);
}

.item-content .el-input.editing :deep(.el-input__wrapper) {
  border-color: #f59e0b;
  background-color: #fffbeb;
  box-shadow: 0 0 0 4px rgba(245, 158, 11, 0.1);
}

.item-content .el-input.is-disabled :deep(.el-input__wrapper) {
  background-color: #f8fafc;
  border-color: #e2e8f0;
  opacity: 0.8;
}

.item-content .el-input :deep(.el-input__inner) {
  font-size: 15px;
  color: #374151;
  font-weight: 500;
  padding: 12px 16px;
}

/* 卡片底部 */
.card-footer {
  padding: 32px 40px;
  background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
  border-top: 1px solid rgba(226, 232, 240, 0.5);
}

.action-buttons {
  display: flex;
  justify-content: flex-end;
  gap: 16px;
}

.cancel-btn,
.save-btn {
  padding: 12px 24px;
  border-radius: 12px;
  font-weight: 600;
  font-size: 15px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  border: 2px solid transparent;
}

.cancel-btn {
  background: #f3f4f6;
  color: #6b7280;
  border-color: #e5e7eb;
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
  .personal-data-container {
    padding: 20px 16px;
  }

  .profile-card {
    border-radius: 20px;
  }

  .card-header {
    flex-direction: column;
    text-align: center;
    padding: 32px 24px 20px;
  }

  .header-avatar {
    margin: 20px 0 0 0;
  }

  .avatar-circle {
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

  .form-grid {
    grid-template-columns: 1fr;
    gap: 24px;
  }

  .card-footer {
    padding: 24px;
  }

  .action-buttons {
    flex-direction: column-reverse;
    gap: 12px;
  }

  .cancel-btn,
  .save-btn {
    width: 100%;
    justify-content: center;
  }
}

@media (max-width: 480px) {
  .personal-data-container {
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

  .divider {
    margin: 0 20px;
  }

  .card-footer {
    padding: 20px;
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

.form-item {
  animation: fadeInUp 0.6s cubic-bezier(0.4, 0, 0.2, 1) forwards;
}

.form-item:nth-child(1) { animation-delay: 0.1s; }
.form-item:nth-child(2) { animation-delay: 0.2s; }
.form-item:nth-child(3) { animation-delay: 0.3s; }
.form-item:nth-child(4) { animation-delay: 0.4s; }
.form-item:nth-child(5) { animation-delay: 0.5s; }
.form-item:nth-child(6) { animation-delay: 0.6s; }
.form-item:nth-child(7) { animation-delay: 0.7s; }
</style>
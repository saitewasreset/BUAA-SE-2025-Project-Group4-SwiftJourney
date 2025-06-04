<template>
    <div class="form-container">
      <div class="form-row">
        <label for="username">用户名</label>
        <el-input type="text" id="username" v-model="username" maxlength="16" :disabled="!isSetUsername" />
        <el-button class="form-row-el-button" type="primary" plain @click="setUsername">{{ setUsernameButtonText }}</el-button>
      </div>

      <div class="form-row">
        <label for="name">姓名</label>
        <el-input type="text" id="name" v-model="desensitizeName" disabled />
      </div>

      <div class="form-row">
        <label for="gender">性别</label>
        <el-input type="text" id="gender" v-model="gender" maxlength="1" :disabled="!isSetGender" />
        <el-button class="form-row-el-button" type="primary" plain @click="setGender">{{ setGenderButtonText }}</el-button>
      </div>

      <div class="form-row">
        <label for="age">年龄</label>
        <el-input type="text" id="age" v-model="age" maxlength="3" :disabled="!isSetAge" />
        <el-button class="form-row-el-button" type="primary" plain @click="setAge">{{ setAgeButtonText }}</el-button>
      </div>
  
      <div class="form-row">
        <label for="identityCardId">身份证号</label>
        <el-input type="text" id="identityCardId" v-model="desensitizeIdentityCardId" disabled />
      </div>
  
      <div class="form-row">
        <label for="phone">手机号</label>
        <el-input type="text" id="phoneNumber" v-model="desensitizePhone" disabled />
      </div>
  
      <div class="form-row">
        <label for="email">邮箱</label>
        <el-input type="text" id="email" v-model="email" :disabled="!isSetEmail"/>
        <el-button class="form-row-el-button" type="primary" plain @click="setEmail">{{ setEmailButtonText }}</el-button>
      </div>
  
      <div class="button-row" v-if="isSetAge | isSetEmail | isSetUsername | isSetGender">
        <el-button class="final-el-button" type="primary" plain @click="cancel">取消</el-button>
        <el-button class="final-el-button" type="primary" @click="save">保存</el-button>
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
        return `${prefix}**`;
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
        }
        if(this.isSetGender) {
          formPostData.gender = this.formTepData.gender == '男' ? 'male' : 'female';
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
  
  .el-input {
    font-size: 16px;
    max-width: 240px;
  }

  .form-row-el-button {
    font-size: 14px;
  }
  
  /* 按钮样式 */
  .button-row {
    margin-top: 20px;
    display: flex;
    justify-content: center;
    align-items: center;
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
  </style>
  
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
  
<script>
import { ElMessage } from 'element-plus';
import { useUserStore } from '@/stores/user';
import { useDebugUserStore } from '@/stores/user';
import validator from 'validator';

const user = useUserStore();
const debugUser = useDebugUserStore();

  export default {
    data() {
      return {
        formData: {
          name: "", //姓名
          username: "",
          identityCardId: "", //身份证号
          phone: "",
          gender: "",
          email: "",
          age: "",
        },
        formTepData: {
          username: "",
          gender: "",
          age: "",
          email: "",
        },
        formPostData: {
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
    created() {
      this.initFormData();
    },
    computed: {
      desensitizeName() {
        const prefix = this.formData.name.substring(0, 1);
        return `${prefix}**`;
      },
      desensitizeEmail() {
        const prefix = this.formData.email.substring(0, 3);
        const atIndex = this.formData.email.indexOf('@');
        if(atIndex != -1) {
          const suffix = this.formData.email.substring(atIndex);
          return `${prefix} **** ${suffix}`;
        }
        return `***`
      },
      username: {
        get(){
          return this.isSetUsername ? this.formTepData.username : this.formData.username;
        },
        set(value) {
          this.formTepData.username = value;
        }
      },
      gender: {
        get(){
          return this.isSetGender ? this.formTepData.gender : this.formData.gender;
        },
        set(value) {
          this.formTepData.gender = value;
        }
      },
      age: {
        get(){
          return this.isSetAge ? this.formTepData.age : this.formData.age;
        },
        set(value) {
          this.formTepData.age = value;
        }
      },
      email: {
        get(){
          return this.isSetEmail ? this.formTepData.email : this.desensitizeEmail;
        },
        set(value) {
          this.formTepData.email = value;
        }
      },
      desensitizeIdentityCardId() {
        const prefix = this.formData.identityCardId.substring(0, 6);
        const suffix = this.formData.identityCardId.substring(this.formData.identityCardId.length - 4);
        return `${prefix} ******** ${suffix}`;
      },
      desensitizePhone() {
        const prefix = this.formData.phone.substring(0, 3);
        const suffix = this.formData.phone.substring(this.formData.phone.length - 2);
        return `${prefix} **** **${suffix}`;
      },
    },
    methods: {
      initFormData() {
        this.formData.name = debugUser.name;
        this.formData.username = debugUser.username;
        this.formData.identityCardId = debugUser.identityCardId;
        this.formData.phone = debugUser.phone;
        this.formData.gender = debugUser.gender === 'male' ? '男' : '女';
        this.formData.email = debugUser.email;
        this.formData.age = debugUser.age;
      },
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
      save() {
        if(!this.checkAll()){
          return;
        }

        if(!this.postData()){
          return;
        }

        this.formData.age = this.formPostData.age;
        this.formData.email = this.formPostData.email;
        this.formData.gender = this.formPostData.gender;
        this.formData.username = this.formPostData.username;

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
      checkAll(){
        if(this.isSetGender && this.formTepData.gender != "") {
          if(this.formTepData.gender != "男" && this.formTepData.gender != "女"){
            ElMessage.error('性别只能为“男”或“女”')
            return false;
          }
        }
        if(this.isSetAge && this.formTepData.age != "") {
          if(this.formTepData.age < 1 || this.formTepData.age > 200){
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
        return true;
      },
      postData(){
        this.formPostData.age = this.formData.age;
        this.formPostData.email = this.formData.email;
        this.formPostData.gender = this.formData.gender;
        this.formPostData.username = this.formData.username;
        if(this.formTepData.username != "") {
          this.formPostData.username = this.formTepData.username;
        }
        if(this.formTepData.gender != "") {
          this.formPostData.gender = this.formTepData.gender;
        }
        if(this.formTepData.age != "") {
          this.formPostData.age = this.formTepData.age;
        }
        if(this.formTepData.email != "") {
          this.formPostData.email = this.formTepData.email;
        }

        //-----------TODO------------//
        //添加将fromPostData发送的逻辑//

        return true;
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
  
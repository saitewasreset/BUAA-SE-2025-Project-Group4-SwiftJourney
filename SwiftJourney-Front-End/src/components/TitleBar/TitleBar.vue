<template>
    <div class="TitleBar">
        <el-menu
        :default-active="activeIndex"
        class="el-menu"
        mode="horizontal"
        :ellipsis="false"
        router
        >
            <div class="webside_title">
                <img src="../../assets/railway.svg" width="30" height="30" />
                <p style="font-size: larger">风行旅途</p>
            </div>
            <div style="border-left: 1px solid #ccc; height: 24px; margin: auto 10px;"></div>
            <el-menu-item index="homepage">首页</el-menu-item>
            <el-menu-item index="trainTicket">车票</el-menu-item>
            <el-menu-item index="hotel">酒店</el-menu-item>
            <el-menu-item index="trainmeal">火车餐</el-menu-item>
        </el-menu>
        <div class="TitleBarButton">
            <el-button @click="goToLoginPage" type="primary" plain>登录</el-button>
            <el-button @click="goToRegisterPage" class="TailButton" type="success" plain>注册</el-button>
        </div>
    </div>
</template>
  
<script lang="ts" setup>
    import { ref } from 'vue';
    import { RouterLink, RouterView } from 'vue-router';
    import { useRouter } from 'vue-router';
    const router = useRouter();

    const activeIndex = ref('homepage');

    function goToLoginPage() {
        router.push({ name: 'login' });
    }

    function goToRegisterPage() {
        router.push({ name: 'register' });
    }
</script>

<style lang="css" scoped>

.TitleBar {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    z-index: 1000;
    background-color: #fff;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.el-menu {
    display: flex;
}
.webside_title {
    margin: auto 10px auto 30px;
    display: flex;
    align-items: center;
    gap: 10px;
}

.webside_title img {
    display: inline-block;
}

.webside_title p {
    margin: 0;
    font-size: larger;
    font-weight: bold;
    color: #333;
}

.TitleBarButton {
    display: flex;
    align-items: center;
    margin-right: 10px;
}
</style>
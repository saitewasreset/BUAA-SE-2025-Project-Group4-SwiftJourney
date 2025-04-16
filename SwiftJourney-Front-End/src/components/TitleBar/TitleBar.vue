<template>
    <div class="TitleBar">
        <el-menu
        :default-active="activeIndex"
        class="el-menu-demo"
        mode="horizontal"
        :ellipsis="false"
        @select="handleSelect"
        >
            <el-menu-item index="0">
            <div class="webside_title">
                <img src="./logo/railway.svg" width="30" height="30" />
                <p style="font-size: larger">风行旅途</p>
            </div>
            </el-menu-item>
            <div style="border-left: 1px solid #ccc; height: 24px; margin: auto 10px;"></div>
            <el-menu-item index="homepage">首页</el-menu-item>
            <el-menu-item index="traintacket">车票</el-menu-item>
            <el-menu-item index="hotel">酒店</el-menu-item>
            <el-menu-item index="trainmeal">火车餐</el-menu-item>
            <div class="TitleBarButton">
                <el-menu-item index="5" style="padding: 0;">
                    <el-button type="primary" plain>登录</el-button>
                </el-menu-item>
                <el-menu-item index="6" style="padding: 10px;">
                    <el-button type="success" plain>注册</el-button>
                </el-menu-item>
            </div>
        </el-menu>
    </div>
</template>
  
<script lang="ts" setup>
    import { ref } from 'vue'
    import { RouterLink, RouterView } from 'vue-router'
    import { useRouter } from 'vue-router'
    const router = useRouter()

    const activeIndex = ref('homepage')
    const handleSelect = (key: string, keyPath: string[]) => {
        router.push({ name: key })
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
}

.webside_title {
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
    gap: 5px;
    padding: 0;
}

.el-menu--horizontal > .el-menu-item:nth-child(6) {
    margin-right: auto;
}
</style>
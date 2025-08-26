<template>
    <div class="hotel-page">
        <div class="content-wrapper">
            <HotelSearch />
        </div>
    </div>
</template>

<script setup lang="ts">
    import HotelSearch from '@/components/HotelSearch/HotelSearch.vue';
</script>

<style lang="scss" scoped>
.hotel-page {
    height: calc(100vh - 60px); // TitleBar 的确切高度是 60px
    background: linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 25%, #fef3c7 75%, #fef7cd 100%);
    position: relative;
    overflow: hidden; // 完全禁用滚动条
    
    &::before {
        content: '';
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: 
            radial-gradient(circle at 20% 80%, rgba(120, 119, 198, 0.1) 0%, transparent 50%),
            radial-gradient(circle at 80% 20%, rgba(255, 119, 198, 0.1) 0%, transparent 50%),
            radial-gradient(circle at 40% 40%, rgba(120, 219, 226, 0.1) 0%, transparent 50%);
        pointer-events: none;
    }
}

.content-wrapper {
    position: relative;
    z-index: 1;
    display: flex;
    justify-content: center;
    padding: 20px;
    height: 100%;
    box-sizing: border-box; // 确保 padding 包含在总高度内
    overflow: hidden;
}

// 响应式设计
@media (max-width: 768px) {
    .content-wrapper {
        padding: 16px;
    }
}
</style>
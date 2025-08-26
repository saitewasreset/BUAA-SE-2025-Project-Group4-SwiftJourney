<template>
  <div class="meal-page">
    <div class="content-wrapper">
      <MealSearch />
    </div>
  </div>
</template>

<script setup lang="ts">
import MealSearch from '@/components/MealSearch/MealSearch.vue'
</script>

<style lang="scss" scoped>
.meal-page {
  min-height: 100vh; // 改为 min-height 确保至少覆盖整个视口
  background: linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 25%, #fef3c7 75%, #fef7cd 100%);
  background-attachment: fixed; // 固定背景，滚动时保持不变
  position: relative;

  &::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    min-height: 100vh; // 确保伪元素也覆盖全部区域
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
  box-sizing: border-box;
  min-height: calc(100vh - 60px); // 确保内容包装器也有足够高度
}

// 为 body 添加全局背景保证
:global(body) {
  background: linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 25%, #fef3c7 75%, #fef7cd 100%);
  background-attachment: fixed;
}

// 响应式设计
@media (max-width: 768px) {
  .content-wrapper {
    padding: 16px;
  }
}
</style>

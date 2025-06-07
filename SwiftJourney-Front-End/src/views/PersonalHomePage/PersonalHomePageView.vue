<template>
  <div class="page-container">
    <div class="sidebar-container">
      <div class="sidebar-header">
        <div class="avatar-section">
          <div class="user-avatar">
            <TeamOutlined />
          </div>
          <div class="user-info">
            <h3 class="user-name">个人中心</h3>
            <p class="user-desc">管理您的账户信息</p>
          </div>
        </div>
      </div>
      
      <div class="menu-wrapper">
        <a-menu
          class="custom-menu"
          mode="inline"
          :items="items"
          v-model:selectedKeys="selectedKeys"
          @click="handleClick" />
      </div>
    </div>
    
    <div class="content-container">
      <component :is="pageRef" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, VueElement, reactive, h, type Component } from 'vue';
import { watch } from 'vue';
import type { ItemType } from 'ant-design-vue';
import { useRoute, useRouter } from 'vue-router';
import { TeamOutlined, LockOutlined, EditOutlined, MoneyCollectOutlined, AuditOutlined } from '@ant-design/icons-vue';

import AccountSecurity from '@/components/PersonalHomeElMenu/AccountSecurity.vue';
import PersonalData from '@/components/PersonalHomeElMenu/PersonalData.vue';
import PrefilledInformation from '@/components/PersonalHomeElMenu/PrefilledInformation.vue';
import TransactionRecord from '@/components/PersonalHomeElMenu/TransactionRecord.vue';
import TravelPlan from '@/components/PersonalHomeElMenu/TravelPlan.vue';

const route = useRoute();
const router = useRouter();
const selectedKeys = ref([route.params.activeIndex]);

function getItem(
  label: VueElement | string,
  key: string,
  icon?: any,
  children?: ItemType[],
  type?: 'group',
): ItemType {
  return {
    key,
    icon,
    children,
    label,
    type,
  } as ItemType;
}

const items: ItemType[] = reactive([
  getItem('个人资料', 'personaldata',h(TeamOutlined)),
  getItem('账户安全', 'accountsecurity', h(LockOutlined)),
  getItem('预填信息', 'prefilledinformation', h(EditOutlined)),
  getItem('行程', 'travelplan', h(MoneyCollectOutlined)),
  getItem('交易记录', 'transactionrecord', h(AuditOutlined))
]);

const pageRefs: Array<{ label: string; ref: Component }> = [
  {label: 'personaldata', ref: PersonalData},
  {label: 'accountsecurity', ref: AccountSecurity},
  {label: 'prefilledinformation', ref: PrefilledInformation},
  {label: 'travelplan', ref: TravelPlan},
  {label: 'transactionrecord', ref: TransactionRecord}
];

const pageRef = ref<Component>(pageRefs.find((page) => page.label ===  route.params.activeIndex)?.ref || PersonalData);

function handleClick(e: { key: string }) {
  const pageKey = e.key;
  const foundPage = pageRefs.find((page) => page.label === pageKey);
  if (foundPage) {
    pageRef.value = foundPage.ref;
    activeIndex.value = pageKey;
  }
}

const activeIndex = computed({
  get() {
    return route.params.activeIndex;
  },
  set(newIndex) {
    if (newIndex !== route.params.activeIndex) {
      router.push({ name: route.name, params: { activeIndex: newIndex } });
    }
  },
});

const validTabs = [
  'personaldata',
  'accountsecurity',
  'prefilledinformation',
  'travelplan',
  'transactionrecord'
];

watch(
  () => activeIndex.value,
  (newVal) => {
    let tab = Array.isArray(newVal) ? newVal[0] : newVal;
    if (!validTabs.includes(tab)) {
      router.replace(`/${route.params.activeIndex}`);
    }
  },
  { immediate: true }
);

</script>

<style scoped>
.page-container {
  display: flex;
  height: calc(100vh - 60px);
  background: linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 25%, #fef3c7 75%, #fef7cd 100%);
  position: relative;
}

.page-container::before {
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

/* 侧边栏容器 */
.sidebar-container {
  width: 220px;
  height: 100%;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border-right: 1px solid rgba(226, 232, 240, 0.6);
  box-shadow: 
    4px 0 20px rgba(0, 0, 0, 0.06),
    0 0 0 1px rgba(255, 255, 255, 0.3);
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
  z-index: 1;
}

.sidebar-container::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 4px;
  background: linear-gradient(90deg, #3b82f6 0%, #1e40af 100%);
}

/* 侧边栏头部 */
.sidebar-header {
  padding: 32px 24px 24px;
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.05) 0%, rgba(30, 64, 175, 0.05) 100%);
  border-bottom: 1px solid rgba(226, 232, 240, 0.5);
}

.avatar-section {
  display: flex;
  align-items: center;
  gap: 16px;
}

.user-avatar {
  width: 48px;
  height: 48px;
  background: linear-gradient(135deg, #3b82f6 0%, #1e40af 100%);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 20px;
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.user-avatar:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(59, 130, 246, 0.4);
}

.user-info {
  flex: 1;
}

.user-name {
  font-size: 18px;
  font-weight: 700;
  color: #1a202c;
  margin: 0 0 4px 0;
  line-height: 1.3;
  letter-spacing: -0.2px;
}

.user-desc {
  font-size: 13px;
  color: #64748b;
  margin: 0;
  font-weight: 500;
  line-height: 1.4;
}

/* 菜单包装器 */
.menu-wrapper {
  flex: 1;
  padding: 16px 0;
  overflow-y: auto;
}

/* 自定义菜单样式 */
.custom-menu {
  background: transparent !important;
  border: none !important;
  padding: 0 16px;
}

.custom-menu :deep(.ant-menu-item) {
  height: 48px !important;
  line-height: 48px !important;
  margin: 6px 0 !important;
  border-radius: 12px !important;
  padding: 0 16px !important;
  border: 2px solid transparent !important;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1) !important;
  position: relative;
  overflow: hidden;
  font-weight: 500;
  color: #64748b !important;
}

.custom-menu :deep(.ant-menu-item::before) {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(59, 130, 246, 0.1), transparent);
  transition: left 0.6s;
}

.custom-menu :deep(.ant-menu-item:hover::before) {
  left: 100%;
}

.custom-menu :deep(.ant-menu-item:hover) {
  background: rgba(59, 130, 246, 0.08) !important;
  border-color: rgba(59, 130, 246, 0.2) !important;
  transform: translateX(4px);
  color: #3b82f6 !important;
}

.custom-menu :deep(.ant-menu-item-selected) {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.15) 0%, rgba(30, 64, 175, 0.15) 100%) !important;
  border-color: #3b82f6 !important;
  color: #3b82f6 !important;
  font-weight: 600 !important;
  box-shadow: 0 2px 8px rgba(59, 130, 246, 0.2);
}

.custom-menu :deep(.ant-menu-item-selected::after) {
  content: '';
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 4px;
  height: 20px;
  background: linear-gradient(135deg, #3b82f6 0%, #1e40af 100%);
  border-radius: 2px;
}

.custom-menu :deep(.ant-menu-item .anticon) {
  font-size: 16px !important;
  margin-right: 12px !important;
  transition: all 0.3s !important;
}

.custom-menu :deep(.ant-menu-item-selected .anticon) {
  color: #3b82f6 !important;
  transform: scale(1.1);
}

.custom-menu :deep(.ant-menu-item span) {
  font-size: 14px !important;
  font-weight: inherit !important;
  transition: all 0.3s !important;
}

/* 内容容器 */
.content-container {
  flex: 1;
  height: 100%;
  overflow-y: auto;
  background: transparent;
  position: relative;
  z-index: 1;
  animation: fadeInUp 0.6s cubic-bezier(0.4, 0, 0.2, 1);
}

/* 滚动条样式 */
.menu-wrapper::-webkit-scrollbar,
.content-container::-webkit-scrollbar {
  width: 6px;
}

.menu-wrapper::-webkit-scrollbar-track,
.content-container::-webkit-scrollbar-track {
  background: rgba(226, 232, 240, 0.3);
  border-radius: 3px;
}

.menu-wrapper::-webkit-scrollbar-thumb,
.content-container::-webkit-scrollbar-thumb {
  background: rgba(59, 130, 246, 0.3);
  border-radius: 3px;
  transition: background 0.3s;
}

.menu-wrapper::-webkit-scrollbar-thumb:hover,
.content-container::-webkit-scrollbar-thumb:hover {
  background: rgba(59, 130, 246, 0.5);
}

/* 响应式设计 */
@media (max-width: 1024px) {
  .sidebar-container {
    width: 260px;
  }
  
  .sidebar-header {
    padding: 24px 20px 20px;
  }
  
  .user-avatar {
    width: 44px;
    height: 44px;
    font-size: 18px;
  }
  
  .user-name {
    font-size: 16px;
  }
  
  .user-desc {
    font-size: 12px;
  }
  
  .custom-menu {
    padding: 0 12px;
  }
  
  .custom-menu :deep(.ant-menu-item) {
    height: 44px !important;
    line-height: 44px !important;
    padding: 0 12px !important;
  }
}

@media (max-width: 768px) {
  .page-container {
    flex-direction: column;
  }
  
  .sidebar-container {
    width: 100%;
    height: auto;
    max-height: 200px;
    order: 2;
  }
  
  .content-container {
    order: 1;
    height: calc(100vh - 200px);
  }
  
  .sidebar-header {
    padding: 16px 20px;
  }
  
  .avatar-section {
    gap: 12px;
  }
  
  .user-avatar {
    width: 40px;
    height: 40px;
    font-size: 16px;
  }
  
  .menu-wrapper {
    padding: 8px 0;
  }
  
  .custom-menu {
    padding: 0 16px;
  }
  
  .custom-menu :deep(.ant-menu-item) {
    height: 40px !important;
    line-height: 40px !important;
    margin: 4px 0 !important;
  }
}

/* 动画效果 */
@keyframes fadeInUp {
  from {
    transform: translateY(20px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

/* 深色模式支持 */
@media (prefers-color-scheme: dark) {
  .page-container {
    background: linear-gradient(135deg, #1a202c 0%, #2d3748 100%);
  }
  
  .page-container::before {
    background: 
      radial-gradient(circle at 20% 80%, rgba(59, 130, 246, 0.1) 0%, transparent 50%),
      radial-gradient(circle at 80% 20%, rgba(30, 64, 175, 0.1) 0%, transparent 50%),
      radial-gradient(circle at 40% 40%, rgba(96, 165, 250, 0.1) 0%, transparent 50%);
  }
  
  .sidebar-container {
    background: rgba(26, 32, 44, 0.95);
    border-right-color: rgba(74, 85, 104, 0.6);
  }
  
  .user-name {
    color: #f7fafc;
  }
  
  .user-desc {
    color: #a0aec0;
  }
  
  .custom-menu :deep(.ant-menu-item) {
    color: #a0aec0 !important;
  }
  
  .custom-menu :deep(.ant-menu-item:hover) {
    background: rgba(59, 130, 246, 0.15) !important;
    color: #93c5fd !important;
  }
  
  .custom-menu :deep(.ant-menu-item-selected) {
    background: linear-gradient(135deg, rgba(59, 130, 246, 0.25) 0%, rgba(30, 64, 175, 0.25) 100%) !important;
    color: #93c5fd !important;
  }
}
</style>
<template>
  <div class="tabs-container">
    <a-menu
      style="width: 144px; height: 100%;"
      mode="inline"
      :items="items"
      v-model:selectedKeys="selectedKeys"
      @click="handleClick" />
  </div>
  <div class="content-container">
    <component :is="pageRef" />
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
  getItem('交易记录', 'transactionrecord', h(AuditOutlined))
]);

const pageRefs: Array<{ label: string; ref: Component }> = [
  {label: 'personaldata', ref: PersonalData},
  {label: 'accountsecurity', ref: AccountSecurity},
  {label: 'prefilledinformation', ref: PrefilledInformation},
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
.tabs-container {
  position: absolute;
  top: 10%;
  left: 20px;
  width: 95%; /* 根据需要调整宽度 */
  height: 90%; /* 根据需要调整高度 */
  background: none;
}

.content-container {
  position: absolute;
  top: 10%;
  left: 200px; /* 与 tabs-container 的宽度相匹配 */
  width: calc(100% - 200px); /* 减去 tabs-container 的宽度 */
  height: 90%; /* 根据需要调整高度 */
  overflow-y: auto; /* 如果内容超出容器，添加滚动条 */
}

</style>
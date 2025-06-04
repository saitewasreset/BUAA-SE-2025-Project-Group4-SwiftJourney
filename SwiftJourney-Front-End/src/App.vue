<template>
  <div style="width: 100%; height: 100%; align-items: center; justify-content: center; display: flex;">
    <div v-if="shouldTitleBarDisplay">
      <TitleBar v-if="shouldTitleBarDisplay" />
      <RouterView />
    </div>
    <div v-else>
      <RouterView />
    </div>
  </div>
</template>

<script setup lang="ts">
import TitleBar from './components/TitleBar/TitleBar.vue'
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'
import { computed, onMounted } from 'vue';
import { useUserStore } from './stores/user';

const route = useRoute()

const undisplayTitleBar: string[] = ['login', 'register']

const shouldTitleBarDisplay = computed(() => {
  if (undisplayTitleBar.includes(route.name as string)) {
    return false
  }
  return true
})

onMounted(async () => {
  const nowUser = useUserStore();
  await nowUser.restoreUserFromCookie(useRouter());
})
</script>

<style scoped>
.root {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.title-bar {
  flex-shrink: 0; /* 防止标题栏被压缩 */
}

.router-view-container {
  padding: 20px;
}

.container-margin {
  margin-top: calc(var(--el-menu-horizontal-height));
}

#app {
  height: 100%;
  width: 100%;
  padding: 0%;
  margin: 0%;
}

/* header {
  line-height: 1.5;
  max-height: 100vh;
}

.logo {
  display: block;
  margin: 0 auto 2rem;
}

@media (min-width: 1024px) {
  header {
    display: flex;
    place-items: center;
    padding-right: calc(var(--section-gap) / 2);
  }

  .logo {
    margin: 0 2rem 0 0;
  }

  header .wrapper {
    display: flex;
    place-items: flex-start;
    flex-wrap: wrap;
  }
} */
</style>

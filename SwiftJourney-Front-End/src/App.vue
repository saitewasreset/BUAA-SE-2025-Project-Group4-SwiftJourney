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
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'
import TitleBar from './components/TitleBar/TitleBar.vue';
import { computed, onMounted } from 'vue';
import { useUserStore } from './stores/user';

const route = useRoute();

const undisplayTitleBar : string[] = ["login", "register"];

const shouldTitleBarDisplay = computed(() => {
  if (undisplayTitleBar.includes(route.name as string)) {
    return false;
  }
  return true;
});

onMounted(async () => {
  const nowUser = useUserStore();
  await nowUser.restoreUserFromCookie(useRouter());
})

</script>

<style scoped>
header {
  line-height: 1.5;
  max-height: 100vh;
}

.logo {
  display: block;
  margin: 0 auto 2rem;
}

nav {
  width: 100%;
  font-size: 12px;
  text-align: center;
  margin-top: 2rem;
}

nav a.router-link-exact-active {
  color: var(--color-text);
}

nav a.router-link-exact-active:hover {
  background-color: transparent;
}

nav a {
  display: inline-block;
  padding: 0 1rem;
  border-left: 1px solid var(--color-border);
}

nav a:first-of-type {
  border: 0;
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

  nav {
    text-align: left;
    margin-left: -1rem;
    font-size: 1rem;

    padding: 1rem 0;
    margin-top: 1rem;
  }
}
</style>

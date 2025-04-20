<template>
  <div style="width: 100%; height: 100%; padding: 0%; margin: 0%;">
    <TitleBar v-if="shouldTitleBarDisplay" />
    <RouterView />
  </div>
</template>

<script setup lang="ts">
import { RouterLink, RouterView, useRoute } from 'vue-router'
import TitleBar from './components/TitleBar/TitleBar.vue';
import { computed } from 'vue';

const route = useRoute();

const undisplayTitleBar : string[] = ["login", "register"];

const shouldTitleBarDisplay = computed(() => {
  if (undisplayTitleBar.includes(route.name as string)) {
    return false;
  }
  return true;
});
</script>

<style scoped>
#app {
  height: 100%;
  width: 100%;
  padding: 0%;
  margin: 0%;
}
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

# SwiftJourney-Front-End

Areka: when you write one new vue text, you should use this template.

```vue
<template>
    
</template>

<script setup lang="ts">
</script>

<style lang="css">
</style>
```

Areka: when you create one new view, please create one new directory in @/views/*
    the router just like:
```js
    {
      path: '/homepage',
      name: 'homepage',
      component: () => import('../views/HomePage/HomePageView.vue'),
    }
```
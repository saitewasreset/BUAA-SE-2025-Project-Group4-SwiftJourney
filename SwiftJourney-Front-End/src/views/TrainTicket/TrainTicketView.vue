<template>
    <a-space direction="vertical">
        <a-space warp>
          <a-tooltip title="search">
            <a-button type="primary" shape="circle" :icon="h(SearchOutlined)" />
          </a-tooltip>
          <a-button type="primary" shape="circle">A</a-button>
          <a-button type="primary" :icon="h(SearchOutlined)">Search</a-button>
          <a-tooltip title="search">
            <a-button shape="circle" :icon="h(SearchOutlined)" />
          </a-tooltip>
          <a-button :icon="h(SearchOutlined)">Search</a-button>
        </a-space>
        <a-space warp>
          <a-tooltip title="search">
            <a-button shape="circle" :icon="h(SearchOutlined)" />
          </a-tooltip>
          <a-button :icon="h(SearchOutlined)">Search</a-button>
          <a-tooltip title="search">
            <a-button type="dashed" shape="circle" :icon="h(SearchOutlined)" />
          </a-tooltip>
          <a-button type="dashed" :icon="h(SearchOutlined)">Search</a-button>
          <a-button :icon="h(SearchOutlined)" href="https://www.google.com" />
        </a-space>
      </a-space>
</template>

<script setup lang="ts">
    import { h } from 'vue';
    import { SearchOutlined } from '@ant-design/icons-vue'; 
</script>

<style lang="css">
</style>
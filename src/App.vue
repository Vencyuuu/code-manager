<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useProjectStore } from './stores/projectStore'

const router = useRouter()
const route = useRoute()
const { loadProjects, refreshAllProjects } = useProjectStore()

const navigate = (path: string) => {
  router.push(path)
}

onMounted(async () => {
  // 全局加载项目数据
  await loadProjects()
  // 应用启动时自动刷新
  await refreshAllProjects()
})
</script>

<template>
  <div class="app-container">
    <aside class="sidebar">
      <div class="logo">
        <h2>代码管理器</h2>
      </div>
      <nav class="nav-menu">
        <a
          href="#"
          class="nav-item"
          :class="{ active: route.path === '/' }"
          @click.prevent="navigate('/')"
        >
          项目列表
        </a>
        <a
          href="#"
          class="nav-item"
          :class="{ active: route.path === '/settings' }"
          @click.prevent="navigate('/settings')"
        >
          设置
        </a>
      </nav>
    </aside>
    <main class="main-content">
      <header class="header">
        <h1>{{ route.path === '/' ? '项目列表' : '设置' }}</h1>
      </header>
      <div class="content">
        <router-view />
      </div>
    </main>
  </div>
</template>

<style scoped>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

.app-container {
  display: flex;
  height: 100vh;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.sidebar {
  width: 220px;
  background: #1e1e1e;
  color: #fff;
  display: flex;
  flex-direction: column;
}

.logo {
  padding: 20px;
  border-bottom: 1px solid #333;
}

.logo h2 {
  font-size: 18px;
  font-weight: 600;
}

.nav-menu {
  padding: 10px 0;
}

.nav-item {
  display: block;
  padding: 12px 20px;
  color: #aaa;
  text-decoration: none;
  transition: all 0.2s;
}

.nav-item:hover,
.nav-item.active {
  background: #2d2d2d;
  color: #fff;
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: #f5f5f5;
}

.header {
  padding: 20px 30px;
  background: #fff;
  border-bottom: 1px solid #e0e0e0;
}

.header h1 {
  font-size: 20px;
  font-weight: 600;
  color: #333;
}

.content {
  flex: 1;
  padding: 30px;
  overflow-y: auto;
}
</style>
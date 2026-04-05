<template>
  <el-config-provider :locale="zhCn">
    <div id="app" class="app-container">
      <el-container class="app-layout">
        <!-- 顶栏导航 -->
        <el-header class="app-header">
          <div class="header-content">
            <div class="header-left">
              <h1 class="app-title">敏感信息扫描工具</h1>
            </div>
            <div class="header-nav">
                 <el-radio-group v-model="activeTab" @change="handleTabChange" size="large">
                <el-radio-button value="scan">开始扫描</el-radio-button>
                <el-radio-button value="results">扫描结果</el-radio-button>
              </el-radio-group>
            </div>
          </div>
        </el-header>

        <!-- 主要容载区 -->
        <el-main class="app-main">
          <div class="content-container">
            <router-view />
          </div>
        </el-main>
      </el-container>
    </div>
  </el-config-provider>
</template>

<script setup lang="ts">
/// <reference path="./element-plus-locale.d.ts" />
import { ref, onMounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import zhCn from 'element-plus/dist/locale/zh-cn.mjs'

const router = useRouter()
const route = useRoute()
const activeTab = ref('scan')

onMounted(() => {
  const path = route.path.replace('/', '')
  if (path) activeTab.value = path
})

watch(() => route.path, (newPath) => {
  activeTab.value = newPath.replace('/', '') || 'scan'
})

const handleTabChange = (tab: string) => {
  router.push(`/${tab}`)
}
</script>

<style scoped lang="css">
.app-container {
  width: 100%;
  height: 100vh;
  background-color: #f5f7fa;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

.app-layout {
  height: 100%;
}

/* 导航栏样式配置 */
.app-header {
  background-color: #fff;
  border-bottom: 1px solid #e8eaf6;
  padding: 0 40px;
  display: flex;
  align-items: center;
  height: 70px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.header-content {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.app-title {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  color: #1a1a1a;
  letter-spacing: -0.5px;
}

/* 主容器层面板 */
.app-main {
  overflow-y: auto;
  padding: 40px;
  background-color: #f8f9fa;
}

.content-container {
  max-width: 1000px;
  margin: 0 auto;
  min-height: calc(100vh - 150px);
}
</style>
